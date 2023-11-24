use poise::serenity_prelude as serenity;

use crate::{Context, Error};

/// Wraps the cache and http from the serenity context.
/// The result is an `impl CacheHttp`.
macro_rules! cache_http {
    ($ctx:expr) => {{
        let ctx = $ctx.serenity_context();
        (&ctx.cache.clone(), ctx.http.as_ref())
    }};
}

/// Gets a user by id from Discord.
pub(crate) async fn get_user(ctx: Context<'_>, id: &i64) -> Result<serenity::User, Error> {
    log::debug!("Getting name for user {id}");
    serenity::UserId(*id as u64)
        .to_user(cache_http!(ctx))
        .await
        .map_err(|e| e.into())
}

/// Gets a user's nickname for the current guild, or defaults to name, from Discord.
pub(crate) async fn get_nick_or_name(ctx: Context<'_>, user: serenity::User) -> String {
    if let Some(guild_id) = ctx.guild_id() {
        if log::log_enabled!(log::Level::Debug) {
            if let Some(guild) = guild_id.to_guild_cached(cache_http!(ctx)) {
                log::debug!(
                    "Getting nickname for {user} in Guild {guild}",
                    user = user.name,
                    guild = guild.name
                );
            }
        }
        user.nick_in(cache_http!(ctx), guild_id)
            .await
            .unwrap_or(user.name)
    } else {
        user.name
    }
}
