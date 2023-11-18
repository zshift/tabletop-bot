use serenity::{
    builder::CreateApplicationCommand,
    model::application::interaction::application_command::CommandDataOption,
};

pub fn run(options: &[CommandDataOption]) -> String {
    "".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
}
