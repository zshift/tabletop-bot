mod db;
mod discord;
mod roll;

use dotenv::dotenv;
use futures::future;
use poise::{
    command,
    serenity_prelude::{self as serenity, GuildId},
};
use r2d2_sqlite::SqliteConnectionManager;
use std::env;

// User data, which is stored and accessible in all command invocations
struct Data {
    conn_pool: r2d2::Pool<SqliteConnectionManager>,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
type Result<T> = core::result::Result<T, Error>;

#[tokio::main]
async fn main() {
    // Load values from .env, if available.
    dotenv().ok();
    pretty_env_logger::init();

    // Login with a bot token from the env.
    let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in the environment");
    let db_path = env::var("DATABASE_PATH").expect("Expected DATABASE_PATH in the environment");
    let guild_id: u64 = env::var("GUILD_ID")
        .expect("Expected GUILD_ID in the environment")
        .parse()
        .expect("GUILD_ID must be a number");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                exp(),
                experience(),
                mvp(),
                register_player(),
                resolve_mvp(),
                roll(),
                schedule(),
                connections(),
            ],
            ..Default::default()
        })
        .token(token)
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                log::info!("Connected to Discord!");
                let mgr = SqliteConnectionManager::file(db_path);
                let pool = r2d2::Pool::new(mgr).expect("Failed to create connection pool");

                let connection = pool.get().expect("Failed to get connection from pool");

                db::setup(&connection).expect("Failed to setup database");

                poise::builtins::register_in_guild(
                    ctx.http.clone(),
                    &framework.options().commands,
                    GuildId(guild_id),
                )
                .await?;
                // Uncomment to register globally.
                // poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { conn_pool: pool })
            })
        });

    log::info!("Connecting to Discord..");
    framework.run().await.expect("Failed to start framework");
}

// Adds experience to a player
#[command(slash_command)]
async fn exp(
    ctx: Context<'_>,
    #[description = "Player"] player: serenity::Member,
    #[description = "Experience"] experience: u32,
) -> Result<()> {
    let conn = ctx.data().conn_pool.clone().get()?;

    let player_id = *player.user.id.as_u64() as i64;
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
async fn experience(ctx: Context<'_>) -> Result<()> {
    let conn = ctx.data().conn_pool.clone().get()?;

    let id_xp = db::get_all_xp(&conn)?;
    let user_xp_futures = id_xp
        .iter()
        .map(|(id, xp)| async move {
            let user = discord::get_user(ctx, id).await?;
            let nick = discord::get_nick_or_name(ctx, user).await;
            Ok::<_, Error>(format!("{}: {}", nick, xp))
        })
        .collect::<Vec<_>>();

    let user_xp = future::try_join_all(user_xp_futures).await?.join("\n");

    ctx.say(user_xp).await?;
    Ok(())
}

// Nominates a player as the MVP
#[command(slash_command)]
async fn mvp(ctx: Context<'_>) -> Result<()> {
    let response = "MVP";
    ctx.say(response).await?;
    Ok(())
}

// Registers a player
#[command(slash_command, rename = "registerplayer")]
async fn register_player(
    ctx: Context<'_>,
    #[description = "Player"] player: serenity::Member,
) -> Result<()> {
    let conn = ctx.data().conn_pool.clone().get()?;

    db::create_player(&conn, player.user.id.0 as i64)?;
    ctx.say(format!("Created {} with 0 experience.", player.user.name))
        .await?;
    Ok(())
}

// Resolves the MVP
#[command(slash_command, rename = "resolve-mvp")]
async fn resolve_mvp(ctx: Context<'_>) -> Result<()> {
    let response = "Resolve MVP";
    ctx.say(response).await?;
    Ok(())
}

// Rolls dice
#[command(slash_command)]
async fn roll(ctx: Context<'_>, #[description = "Dice"] dice: String) -> Result<()> {
    match roll::eval(&dice).map_err(|e| e.to_string()) {
        Ok(results) => {
            ctx.say(format!("{}", results)).await?;
        }
        Err(e) => {
            ctx.say(format!("Error: {}", e)).await?;
        }
    }
    Ok(())
}

// Schedules a game
#[command(slash_command)]
async fn schedule(ctx: Context<'_>) -> Result<()> {
    let response = "Schedule";
    ctx.say(response).await?;
    Ok(())
}

#[command(slash_command)]
async fn connections(ctx: Context<'_>) -> Result<()> {
    let pool = ctx.data().conn_pool.clone();
    ctx.say(format!(
        "Connections: {}, Idle connections: {}",
        pool.state().connections,
        pool.state().idle_connections
    ))
    .await?;
    Ok(())
}
