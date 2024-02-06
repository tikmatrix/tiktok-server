use std::sync::Mutex;

use crate::{database, runtime_err::RunTimeError};
use rusqlite::{Connection, Result};

use crate::models::{AvatarData, AvatarDetails, AvatarResponseData};

pub fn save(conn: &Mutex<Connection>, avatars: Vec<AvatarData>) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    for m in avatars {
        conn.execute(
            "INSERT INTO avatar (name) VALUES (?1)",
            rusqlite::params![m.name],
        )?;
    }
    Ok(())
}
pub fn list_all() -> Result<AvatarResponseData, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt = conn.prepare("SELECT id, name FROM avatar LIMIT 200")?;
    let avatars = stmt
        .query_map([], |row| {
            Ok(AvatarDetails {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?
        .collect::<Result<Vec<AvatarDetails>, _>>()?;
    Ok(AvatarResponseData { data: avatars })
}
pub fn delete(conn: &Mutex<Connection>, id: i32) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    conn.execute("DELETE FROM avatar WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}
pub fn random_one() -> Result<AvatarDetails, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt = conn.prepare("SELECT id, name FROM avatar ORDER BY RANDOM() LIMIT 1")?;
    let avatar = stmt.query_row([], |row| {
        Ok(AvatarDetails {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;
    Ok(avatar)
}
