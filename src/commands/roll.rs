use serenity::{
    builder::CreateApplicationCommand,
    model::application::interaction::application_command::CommandDataOption,
};

use super::PathfinderBotCommand;

pub struct Roll;

impl PathfinderBotCommand for Roll {
    fn run(&self, options: &[CommandDataOption]) -> String {
        todo!()
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    todo!()
}
