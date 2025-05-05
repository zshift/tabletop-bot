use std::{
    fmt::Display,
    sync::{Mutex, RwLock},
};

use poise::serenity_prelude::{self as serenity, CacheHttp};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use timer::{Guard, Timer};
use tokio::runtime::Handle;

use crate::db::{self, ScheduledMessage};

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub(crate) enum Error {
    Db(db::Error),
    R2d2(r2d2::Error),
}

impl From<db::Error> for Error {
    fn from(e: db::Error) -> Self {
        Error::Db(e)
    }
}

impl From<r2d2::Error> for Error {
    fn from(e: r2d2::Error) -> Self {
        Error::R2d2(e)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Db(e) => write!(f, "Database error: {}", e),
            Error::R2d2(e) => write!(f, "R2D2 error: {}", e),
        }
    }
}

impl std::error::Error for Error {}

pub(crate) struct Scheduler<T>
where
    T: AsRef<serenity::Http> + Clone + Send + Sync + 'static,
{
    timer: Mutex<timer::Timer>,
    pool: Pool<SqliteConnectionManager>,
    guard: RwLock<Option<Guard>>,
    ctx: T,
}

impl<T: AsRef<serenity::Http> + CacheHttp + Clone + Send + Sync + 'static> Scheduler<T> {
    pub(crate) fn new(pool: Pool<SqliteConnectionManager>, ctx: T) -> Self {
        Self {
            timer: Mutex::new(Timer::new()),
            pool,
            guard: RwLock::new(None),
            ctx,
        }
    }

    pub(crate) fn sync_schedule(&mut self) -> Result<()> {
        log::info!("Syncing schedule");
        let conn = self.pool.clone().get()?;

        match db::get_schedule(&conn)? {
            Some(sch) => {
                log::info!("Found schedule: `{:?}`. Starting timer.", sch);
                self.inner_schedule(&sch)
            }
            None => {
                log::info!("No schedule found.");
                Ok(())
            }
        }
    }

    pub(crate) fn schedule(&mut self, sch: &ScheduledMessage) -> Result<()> {
        let conn = self.pool.clone().get()?;

        db::create_schedule(&conn, sch)?;
        self.inner_schedule(sch)
    }

    fn inner_schedule(&mut self, sch: &ScheduledMessage) -> Result<()> {
        let sch = sch.clone();
        let handle = Handle::current();

        let ctx = self.ctx.clone();
        let pool = self.pool.clone();

        let guard = self
            .timer
            .lock()
            .expect("Unable to lock timer")
            .schedule_with_date(sch.on, move || {
                Self::send_msg(ctx.clone(), &pool, handle.clone(), &sch)
            });

        let old_guard = self
            .guard
            .write()
            .expect("Unable to get mut guard")
            .replace(guard);

        drop(old_guard);

        Ok(())
    }

    fn send_msg(
        ctx: T,
        pool: &Pool<SqliteConnectionManager>,
        handle: Handle,
        sch: &ScheduledMessage,
    ) {
        handle.block_on(async {
            log::info!("Sending scheduled message");

            match serenity::ChannelId::from(sch.channel_id)
                .say(&ctx, &sch.msg)
                .await
            {
                Ok(msg) => {
                    log::info!("Scheduled message sent: {}", msg.content);
                    pool.get()
                        .map(|conn| {
                            db::delete_schedule(&conn).unwrap_or_else(|e| {
                                log::error!("Error deleting schedule: {}", e);
                            })
                        })
                        .unwrap_or_else(|e| {
                            log::error!("Error getting connection: {}", e);
                        })
                }
                Err(e) => log::error!("Error sending scheduled message: {}", e),
            }
        });
    }
}
