use std::sync::Arc;

use serenity::{
    builder::CreateApplicationCommand,
    model::application::interaction::application_command::CommandDataOption,
};

use super::PathfinderBotCommand;

pub struct RegisterPlayer {
    connection: Arc<sqlite::ConnectionThreadSafe>,
}

impl RegisterPlayer {
    pub(crate) fn new(connection: Arc<sqlite::ConnectionThreadSafe>) -> Self {
        Self { connection }
    }
}

impl PathfinderBotCommand for RegisterPlayer {
    fn run(&self, options: &[CommandDataOption]) -> String {
        todo!()
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    todo!()
}
