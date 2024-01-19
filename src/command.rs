use crate::{db, discord, Context, Error, Result};
use futures::future;
use poise::{command, serenity_prelude as serenity};

// Adds experience to a player
#[command(slash_command)]
pub async fn exp(
    ctx: Context<'_>,
    #[description = "Player"] player: serenity::Member,
    #[description = "Experience"] experience: u32,
) -> Result<()> {
    let conn = ctx.data().pool.clone().get()?;

    let player_id = player.user.id.get() as i64;
    let curr_xp = db::get_xp(&conn, player_id)?;
    let new_xp = curr_xp + experience as i64;

    db::set_xp(&conn, player_id, new_xp)?;

    let response = format!(
        "Updated {}'s account from {}xp to {}xp.",
        player.user.name, curr_xp, new_xp
    );
    ctx.say(response).await?;
    Ok(())
}

// Returns the experience of all players.
#[command(slash_command)]
pub async fn experience(ctx: Context<'_>) -> Result<()> {
    log::debug!("Getting experience");
    let conn = ctx.data().pool.clone().get()?;

    let id_xp = db::get_all_xp(&conn)?;
    if id_xp.is_empty() {
        ctx.say("No experience yet").await?;
        return Ok(());
    }

    let user_xp_futures = id_xp
        .iter()
        .map(|(id, xp)| async move {
            let user = discord::get_user(ctx, id).await?;
            let nick = discord::get_nick_or_name(ctx, user).await;
            Ok::<_, Error>(format!("{}: {}", nick, xp))
        })
        .collect::<Vec<_>>();

    let user_xp = future::try_join_all(user_xp_futures).await?.join("\n");
    let user_xp = user_xp.trim();

    if user_xp.trim().is_empty() {
        ctx.say("No experience yet").await?;
        return Ok(());
    }

    log::debug!("Sending experience: {}", user_xp);
    ctx.say(user_xp).await?;

    log::debug!("Done sending experience");
    Ok(())
}

// Nominates a player as the MVP
#[command(slash_command)]
pub async fn mvp(ctx: Context<'_>, #[description = "MVP"] mvp: serenity::Member) -> Result<()> {
    let conn = ctx.data().pool.clone().get()?;

    let player_id = ctx.author().id.get() as i64;
    let mvp_id = mvp.user.id.get() as i64;

    let result = db::vote_for_mvp(&conn, player_id, mvp_id);
    match result {
        Ok(_) => {
            let nick = discord::get_nick_or_name(ctx, mvp.user).await;
            ctx.say(format!("Your vote for {} was registered", nick))
                .await?;
        }

        Err(e) => {
            ctx.say(format!("Error voting for MVP: {}", e)).await?;
            return Ok(());
        }
    }
    Ok(())
}

// Registers a player
#[command(slash_command, rename = "registerplayer")]
pub async fn register_player(
    ctx: Context<'_>,
    #[description = "Player"] player: serenity::Member,
) -> Result<()> {
    let conn = ctx.data().pool.clone().get()?;
    let player_id = player.user.id.get() as i64;

    db::create_player(&conn, player_id)?;
    ctx.say(format!("Created {} with 0 experience.", player.user.name))
        .await?;
    Ok(())
}

// Resolves the MVP
#[command(slash_command, rename = "resolve-mvp")]
pub async fn resolve_mvp(ctx: Context<'_>) -> Result<()> {
    let mut conn = ctx.data().pool.clone().get()?;

    match db::resolve_mvp(&mut conn) {
        Ok(mvp_id) => {
            let mvp = discord::get_user(ctx, &mvp_id).await?;
            let nick = discord::get_nick_or_name(ctx, mvp).await;

            ctx.say(format!("The MVP is {}!", nick)).await?;
        }

        Err(e) => match e {
            db::Error::MissingVotes => {
                ctx.say("Not everyone has voted").await?;
            }
            db::Error::Chrono(e) => {
                ctx.say(format!("Error parsing datetime: {}", e)).await?;
            }
            db::Error::Sqlite(e) => {
                ctx.say(format!("Error querying database: {}", e)).await?;
            }
        },
    }

    Ok(())
}

// Rolls dice
#[command(slash_command)]
pub async fn roll(ctx: Context<'_>, #[description = "Dice"] dice: String) -> Result<()> {
    let mut rng = ctx.data().rng.clone();

    match evaluroll::eval(&mut rng, &dice).map_err(|e| e.to_string()) {
        Ok(results) => {
            ctx.say(format!(
                "Rolled **{}** = {}",
                dice,
                discord::Output(&results)
            ))
            .await?;
        }

        Err(e) => {
            ctx.say(format!("Error: {}", e)).await?;
        }
    }
    Ok(())
}

// Schedules a game
#[command(slash_command)]
pub async fn schedule(
    ctx: Context<'_>,
    #[description = "Channel"] channel: serenity::Channel,
    #[description = "Message"] msg: String,
    #[description = "On"] on: serenity::Timestamp,
) -> Result<()> {
    log::info!("Scheduling message: {} on {}", msg, on);

    let channel_id = channel.id().get();

    let sch = db::ScheduledMessage {
        channel_id,
        msg,
        on: (*on).into(),
    };

    {
        let mut scheduler = ctx
            .data()
            .scheduler
            .write()
            .expect("Unable to get mut scheduler");

        log::info!("Scheduling message");
        scheduler.schedule(&sch)?;
        log::info!("Scheduled message");
    }

    ctx.say("Message scheduled!").await?;

    Ok(())
}

#[command(slash_command)]
pub async fn connections(ctx: Context<'_>) -> Result<()> {
    let pool = ctx.data().pool.clone();
    ctx.say(format!(
        "Connections: {}, Idle connections: {}",
        pool.state().connections,
        pool.state().idle_connections
    ))
    .await?;
    Ok(())
}
