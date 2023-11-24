use rusqlite::{named_params, Connection, Result};

// Get the xp of a single player.
pub(crate) fn get_xp(conn: &Connection, player_id: i64) -> Result<i64> {
    conn.query_row(
        "SELECT experience FROM players WHERE players.id = :id",
        named_params! { ":id": player_id },
        |row| row.get(0),
    )
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

pub(crate) fn get_all_xp(conn: &Connection) -> Result<Vec<(i64, i64)>> {
    let mut stmt = conn.prepare("SELECT id, experience FROM players")?;
    stmt.query_map((), |row| {
        let id = row.get(0)?;
        let xp = row.get(1)?;
        Ok((id, xp))
    })
    .map(|iter| {
        iter.filter(|r| r.is_ok())
            .map(|x| x.unwrap())
            .collect::<Vec<_>>()
    })
}

pub(crate) fn create_player(conn: &Connection, player_id: i64) -> Result<()> {
    let mut stmt = conn.prepare("INSERT INTO players (id) VALUES (:id)")?;
    stmt.execute(named_params! { ":id": player_id })?;
    Ok(())
}

pub(crate) fn setup(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS players (
        id INTEGER PRIMARY KEY,
        experience INTEGER NOT NULL DEFAULT 0
    )",
        (),
    )?;
    Ok(())
}
