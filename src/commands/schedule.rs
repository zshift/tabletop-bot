use serenity::{
    builder::CreateApplicationCommand,
    model::application::interaction::application_command::CommandDataOption,
};

use super::PathfinderBotCommand;

pub struct Schedule {
    cron: (),
}

impl Schedule {
    pub fn new() -> Self {
        Self { cron: () }
    }
}

impl PathfinderBotCommand for Schedule {
    fn run(&self, options: &[CommandDataOption]) -> String {
        todo!()
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    todo!()
}
