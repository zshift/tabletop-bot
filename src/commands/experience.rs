use std::sync::Arc;

use serenity::{
    builder::CreateApplicationCommand,
    model::application::interaction::application_command::CommandDataOption,
};

use super::PathfinderBotCommand;

pub struct Experience {
    conn: Arc<sqlite::ConnectionThreadSafe>,
}

impl Experience {
    pub(crate) fn new(conn: Arc<sqlite::ConnectionThreadSafe>) -> Self {
        Self { conn }
    }
}

impl PathfinderBotCommand for Experience {
    fn run(&self, options: &[CommandDataOption]) -> String {
        todo!()
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    todo!()
}
