use std::sync::Arc;

use serenity::{
    builder::CreateApplicationCommand,
    model::application::interaction::application_command::CommandDataOption,
};

use super::PathfinderBotCommand;

pub struct Mvp {
    connection: Arc<sqlite::ConnectionThreadSafe>,
}

impl Mvp {
    pub fn new(connection: Arc<sqlite::ConnectionThreadSafe>) -> Self {
        Self { connection }
    }
}

impl PathfinderBotCommand for Mvp {
    fn run(&self, options: &[CommandDataOption]) -> String {
        todo!()
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    todo!()
}
