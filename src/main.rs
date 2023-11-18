mod commands;

use commands::{
    exp::Exp, experience::Experience, mvp::Mvp, register_player::RegisterPlayer,
    resolve_mvp::ResolveMvp, roll::Roll, schedule::Schedule, PathfinderBotCommand,
};
use dotenv::dotenv;
use serenity::{
    async_trait,
    model::{
        application::{
            command::Command,
            interaction::{Interaction, InteractionResponseType},
        },
        gateway::Ready,
        id::GuildId,
    },
    prelude::{Context, EventHandler, GatewayIntents},
    Client,
};
use std::{
    collections::HashMap,
    env,
    sync::{Arc, RwLock},
};

#[tokio::main]
async fn main() {
    // Load .env values, if available
    dotenv().ok();

    // Login with a bot token from the env
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let connection =
        sqlite::Connection::open_thread_safe("database.db").expect("Failed to open database");
    let handler = Handler::new(Arc::new(connection));

    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(handler)
        .await
        .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponentiontal backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("An error occurred while starting the client: {:?}", why);
    }
}

struct Handler {
    handlers: HashMap<&'static str, Arc<RwLock<dyn PathfinderBotCommand + Send + Sync>>>,
}

impl Handler {
    pub fn new(conn: Arc<sqlite::ConnectionThreadSafe>) -> Self {
        let mut handlers: HashMap<&str, Arc<RwLock<dyn PathfinderBotCommand + Send + Sync>>> =
            HashMap::new();
        handlers.insert("exp", Arc::new(RwLock::new(Exp::new(conn.clone()))));
        handlers.insert(
            "experience",
            Arc::new(RwLock::new(Experience::new(conn.clone()))),
        );
        handlers.insert("mvp", Arc::new(RwLock::new(Mvp::new(conn.clone()))));
        handlers.insert(
            "registerplayer",
            Arc::new(RwLock::new(RegisterPlayer::new(conn.clone()))),
        );
        handlers.insert(
            "resolve-mvp",
            Arc::new(RwLock::new(ResolveMvp::new(conn.clone()))),
        );
        handlers.insert("roll", Arc::new(RwLock::new(Roll)));
        handlers.insert("schedule", Arc::new(RwLock::new(Schedule::new())));

        Self { handlers }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command);

            let content = self
                .handlers
                .get(command.data.name.as_str())
                .expect("Unknown command")
                .read()
                .expect("Unable to read")
                .run(&command.data.options);

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in the environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| commands::exp::register(command))
                .create_application_command(|command| commands::experience::register(command))
                .create_application_command(|command| commands::mvp::register(command))
                .create_application_command(|command| commands::register_player::register(command))
                .create_application_command(|command| commands::resolve_mvp::register(command))
                .create_application_command(|command| commands::roll::register(command))
                .create_application_command(|command| commands::schedule::register(command))
        })
        .await;

        println!(
            "I now have the following guild slash commands: {:#?}",
            commands
        );

        // TODO: What is this??
        let guild_command = Command::create_global_application_command(&ctx.http, |command| {
            commands::test::register(command)
        })
        .await;

        println!(
            "I created the following global slash command: {:#?}",
            guild_command
        );
    }
}
