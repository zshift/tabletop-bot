use std::fmt::Display;

use chrono::{DateTime, Local};
use rusqlite::{named_params, Connection};

#[derive(Debug)]
pub(crate) enum Error {
    MissingVotes,
    Sqlite(rusqlite::Error),
    Chrono(chrono::ParseError),
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Error::Sqlite(e)
    }
}

impl From<chrono::ParseError> for Error {
    fn from(e: chrono::ParseError) -> Self {
        Error::Chrono(e)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {:?}", self)
    }
}

impl std::error::Error for Error {}

type Result<T, E = Error> = std::result::Result<T, E>;

// Get the xp of a single player.
pub(crate) fn get_xp(conn: &Connection, player_id: i64) -> Result<i64> {
    let xp = conn.query_row(
        "SELECT experience FROM players WHERE players.id = :id",
        named_params! { ":id": player_id },
        |row| row.get(0),
    )?;

    Ok(xp)
}

pub(crate) fn set_xp(conn: &Connection, player_id: i64, xp: i64) -> Result<()> {
    let query = "UPDATE players SET experience = :xp WHERE players.id = :id";
    conn.execute(
        query,
        named_params! {
            ":id": player_id,
            ":xp": xp
        },
    )?;

    Ok(())
}

pub(crate) fn vote_for_mvp(conn: &Connection, player_id: i64, mvp_id: i64) -> Result<()> {
    // Perform an upsert, which allows players to update their votes.
    let query = "INSERT INTO mvp (playerid, mvpid) VALUES (:playerid, :mvpid)
    ON CONFLICT(playerid) DO UPDATE SET mvpid = :mvpid";
    conn.execute(
        query,
        named_params! {
            ":playerid": player_id,
            ":mvpid": mvp_id
        },
    )?;

    Ok(())
}

pub(crate) fn resolve_mvp(conn: &mut Connection) -> Result<i64> {
    let tx = conn.transaction()?;

    let query =
        "SELECT (SELECT COUNT(*) FROM mvp)=(SELECT COUNT(*) FROM players) as RowCountResult";
    let has_everyone_voted: bool = tx.query_row(query, [], |row| row.get(0))?;
    if !has_everyone_voted {
        tx.rollback()?;

        return Err(Error::MissingVotes);
    }

    let query = "SELECT mvpid, COUNT(*) FROM mvp GROUP BY mvpid ORDER BY COUNT(*) DESC LIMIT 1";
    let mvp = tx.query_row(query, [], |row| row.get(0))?;

    tx.execute("DELETE FROM mvp", [])?;

    tx.commit()?;

    Ok(mvp)
}

pub(crate) fn get_all_xp(conn: &Connection) -> Result<Vec<(i64, i64)>> {
    let mut stmt = conn.prepare("SELECT id, experience FROM players")?;

    let all_xp = stmt
        .query_map((), |row| {
            let id = row.get(0)?;
            let xp = row.get(1)?;
            Ok((id, xp))
        })
        .map(|iter| {
            iter.filter_map(|x| x.ok())
                .collect::<Vec<_>>()
        })?;

    Ok(all_xp)
}

pub(crate) fn create_player(conn: &Connection, player_id: i64) -> Result<()> {
    let mut stmt = conn.prepare("INSERT INTO players (id) VALUES (:id)")?;
    stmt.execute(named_params! { ":id": player_id })?;
    Ok(())
}

#[derive(Clone, Debug)]
pub struct ScheduledMessage {
    pub channel_id: u64,
    pub msg: String,
    pub on: DateTime<Local>,
}

pub(crate) fn create_schedule(conn: &Connection, sch: &ScheduledMessage) -> Result<()> {
    let mut stmt = conn.prepare(
        "INSERT INTO schedule (id, channel_id, scheduled, msg) VALUES (1, :channel_id, :scheduled, :msg)
    ON CONFLICT (id) DO UPDATE SET
        channel_id = excluded.channel_id,
        scheduled = excluded.scheduled,
        msg = excluded.msg",
    )?;
    stmt.execute(named_params! {
        ":channel_id": sch.channel_id,
        ":scheduled": sch.on.to_rfc3339(),
        ":msg": sch.msg
    })?;
    Ok(())
}

pub(crate) fn get_schedule(conn: &Connection) -> Result<Option<ScheduledMessage>> {
    let query = "SELECT channel_id, scheduled, msg FROM schedule";

    let query_results = conn.query_row(query, [], |row| {
        let channel_id = row.get(0)?;
        let on = row.get(1)?;
        let msg = row.get(2)?;
        Ok(Some((channel_id, on, msg)))
    });

    let scheduled_message: Option<(u64, String, String)> = {
        match query_results {
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            sch @ Ok(_) => sch,
            e @ Err(_) => e,
        }
    }?;

    match scheduled_message {
        Some((channel_id, on, msg)) => Ok(Some(ScheduledMessage {
            channel_id,
            on: parse_schedule(on)?,
            msg,
        })),
        None => Ok(None),
    }
}

pub(crate) fn delete_schedule(conn: &Connection) -> Result<()> {
    let query = "DELETE FROM schedule";
    conn.execute(query, [])?;
    Ok(())
}

fn parse_schedule(on: String) -> Result<DateTime<Local>> {
    match DateTime::parse_from_rfc3339(&on) {
        Ok(on) => Ok(on.into()),
        Err(e) => {
            log::error!("Error parsing datetime: {}", e);
            Err(e.into())
        }
    }
}

// TODO: Move this to a migration.
pub(crate) fn setup(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "BEGIN;
    CREATE TABLE IF NOT EXISTS players (
        id INTEGER PRIMARY KEY,
        experience INTEGER NOT NULL DEFAULT 0
    );

    CREATE TABLE IF NOT EXISTS mvp (
        id INTEGER PRIMARY KEY,
        playerid INTEGER NOT NULL UNIQUE,
        mvpid INTEGER NOT NULL,
        FOREIGN KEY(playerid) REFERENCES players(id),
        FOREIGN KEY(mvpid) REFERENCES players(id)
    );

    CREATE TABLE IF NOT EXISTS schedule (
        id INTEGER PRIMARY KEY,
        channel_id INTEGER NOT NULL,
        scheduled TEXT NOT NULL,
        msg TEXT NOT NULL
    );

    COMMIT;",
    )?;

    Ok(())
}
