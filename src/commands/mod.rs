use serenity::model::application::interaction::application_command::CommandDataOption;

pub mod exp;
pub mod experience;
pub mod mvp;
pub mod register_player;
pub mod resolve_mvp;
pub mod roll;
pub mod schedule;
pub mod test;

pub trait PathfinderBotCommand {
    fn run(&self, options: &[CommandDataOption]) -> String;
}
