mod command;
mod db;
mod discord;
mod exit_on_err;
mod scheduler;

use dotenvy::dotenv;
use exit_on_err::MapOrExit;
use poise::{
    serenity_prelude::{self as serenity, GuildId},
    FrameworkError,
};
use r2d2_sqlite::SqliteConnectionManager;
use rand::{Rng, SeedableRng};
use rand_hc::Hc128Rng;
use scheduler::Scheduler;
use std::{
    env,
    sync::{Arc, RwLock},
};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data<serenity::Context, Hc128Rng>, Error>;
type Result<T> = core::result::Result<T, Error>;

// User data, which is stored and accessible in all command invocations
struct Data<T, R>
where
    T: AsRef<serenity::Http> + Clone + Send + Sync + 'static,
    R: Rng + ?Sized,
{
    pool: r2d2::Pool<SqliteConnectionManager>,
    scheduler: Arc<RwLock<Scheduler<T>>>,
    rng: R,
}

async fn handle_error<T>(error: FrameworkError<'_, T, Error>) {
    log::error!("Error: {}", error);

    if let Some(ctx) = error.ctx() {
        if let Err(e) = ctx.say(format!("Error: {}", error)).await {
            log::error!("Error sending error message: {}", e);
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load values from .env, if available.
    dotenv().ok();
    pretty_env_logger::init();

    // Login with a bot token from the env.
    let token = env::var("DISCORD_TOKEN").map_or_exit("Expected DISCORD_TOKEN in the environment");
    let db_path =
        env::var("DATABASE_PATH").map_or_exit("Expected DATABASE_PATH in the environment");
    let guild_id: u64 = env::var("GUILD_ID")
        .map_or_exit("Expected GUILD_ID in the environment")
        .parse()
        .expect("GUILD_ID must be a number");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                command::exp(),
                command::experience(),
                command::mvp(),
                command::register_player(),
                command::resolve_mvp(),
                command::roll(),
                command::schedule(),
                command::connections(),
            ],
            on_error: |error| Box::pin(handle_error(error)),
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            Box::pin(async move {
                log::info!("Connected to Discord as {}!", ready.user.name);
                let mgr = SqliteConnectionManager::file(db_path);
                let pool = r2d2::Pool::new(mgr).expect("Failed to create connection pool");

                let connection = pool.get().expect("Failed to get connection from pool");

                db::setup(&connection).expect("Failed to setup database");
                poise::builtins::register_in_guild(
                    &ctx,
                    &framework.options().commands,
                    GuildId::new(guild_id),
                )
                .await?;
                // Uncomment to register globally.
                // poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                let mut scheduler = Scheduler::new(pool.clone(), ctx.clone());
                scheduler.sync_schedule()?;

                Ok(Data {
                    pool,
                    scheduler: Arc::new(RwLock::new(scheduler)),
                    rng: Hc128Rng::from_os_rng(),
                })
            })
        })
        .build();

    let mut client =
        serenity::ClientBuilder::new(token, serenity::GatewayIntents::non_privileged())
            .framework(framework)
            .await?;

    log::info!("Connecting to Discord...");
    client.start().await?;

    Ok(())
}
