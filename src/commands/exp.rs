use std::sync::Arc;

use serenity::{
    builder::CreateApplicationCommand,
    model::{
        application::{
            command::CommandOptionType, interaction::application_command::CommandDataOptionValue,
        },
        prelude::interaction::application_command::CommandDataOption,
    },
};

use super::PathfinderBotCommand;

pub struct Exp {
    connection: Arc<sqlite::ConnectionThreadSafe>,
}

impl Exp {
    pub fn new(connection: Arc<sqlite::ConnectionThreadSafe>) -> Self {
        Self { connection }
    }
}

impl PathfinderBotCommand for Exp {
    fn run(&self, options: &[CommandDataOption]) -> String {
        let option = options
            .get(0)
            .expect("Expected User option")
            .resolved
            .as_ref()
            .expect("Expected User option");

        if let CommandDataOptionValue::User(user, partial) = option {
            format!(
                "User is {}#{} ({:#?})",
                user.name, user.discriminator, partial
            )
        } else {
            "".to_string()
        }
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("exp")
        .description("Add Experience to a Player")
        .create_option(|option| {
            option
                .name("Player")
                .description("The player to add experience to")
                .kind(CommandOptionType::User)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("Experience")
                .description("The amount of experience to add to the player's experience pool.")
                .kind(CommandOptionType::Integer)
                .min_int_value(1)
                .required(true)
        })
}
