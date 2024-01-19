use poise::serenity_prelude as serenity;

use crate::{Context, Error};

/// Gets a user by id from Discord.
pub(crate) async fn get_user(ctx: Context<'_>, id: &i64) -> Result<serenity::User, Error> {
    log::debug!("Getting name for user {id}");
    serenity::UserId::from(*id as u64)
        .to_user(&ctx.serenity_context())
        .await
        .map_err(|e| e.into())
}

/// Gets a user's nickname for the current guild, or defaults to name, from Discord.
pub(crate) async fn get_nick_or_name(ctx: Context<'_>, user: serenity::User) -> String {
    if let Some(guild_id) = ctx.guild_id() {
        if log::log_enabled!(log::Level::Debug) {
            if let Some(guild) = guild_id.to_guild_cached(&ctx) {
                log::debug!(
                    "Getting nickname for {user} in Guild {guild}",
                    user = user.name,
                    guild = guild.name
                );
            }
        }
        user.nick_in(&ctx.serenity_context(), guild_id)
            .await
            .unwrap_or(user.name)
    } else {
        user.name
    }
}

use std::fmt::Display;

pub(crate) struct RollDisplay<'a>(pub &'a evaluroll::ast::Roll);

impl<'a> Display for RollDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let roll = self.0;
        if roll.keep {
            write!(f, "**{}**", roll.result)
        } else {
            write!(f, "{}", roll.result)
        }
    }
}

pub(crate) struct Output<'a>(pub &'a evaluroll::ast::Output);

impl<'a> Display for Output<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} [{}]",
            self.0.total,
            self.0
                .rolls
                .iter()
                .map(RollDisplay)
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        )
    }
}
