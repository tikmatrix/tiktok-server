use std::sync::Mutex;

use crate::models::{MusicData, MusicDetails, MusicResponseData};
use crate::{database, runtime_err::RunTimeError};
use rusqlite::{Connection, Result};

pub fn save(conn: &Mutex<Connection>, data: MusicData) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    conn.execute(
        "INSERT INTO `music` (release_name, artist_name) VALUES (?1, ?2)",
        rusqlite::params![data.release_name, data.artist_name,],
    )?;
    Ok(())
}
pub fn update(conn: &Mutex<Connection>, data: MusicData) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    conn.execute(
        "UPDATE `music` SET release_name = ?1, artist_name = ?2 WHERE id = ?3",
        rusqlite::params![data.release_name, data.artist_name, data.id],
    )?;
    Ok(())
}
pub fn list_all() -> Result<MusicResponseData, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt =
        conn.prepare("SELECT id, release_name, artist_name FROM `music` ORDER BY id ASC")?;
    let mut data = Vec::new();
    let iter = stmt.query_map((), |row| {
        Ok(MusicDetails {
            id: row.get(0)?,
            release_name: row.get(1)?,
            artist_name: row.get(2)?,
        })
    })?;
    for item in iter {
        data.push(item?);
    }
    Ok(MusicResponseData { data })
}

pub fn del(id: i32) -> Result<(), RunTimeError> {
    let conn = database::get_conn()?;
    conn.execute("DELETE FROM `music` WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}
pub fn random_one() -> Result<MusicDetails, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt = conn
        .prepare("SELECT id, release_name, artist_name FROM `music` ORDER BY RANDOM() LIMIT 1")?;
    let mut data = Vec::new();
    let iter = stmt.query_map((), |row| {
        Ok(MusicDetails {
            id: row.get(0)?,
            release_name: row.get(1)?,
            artist_name: row.get(2)?,
        })
    })?;
    for item in iter {
        data.push(item?);
    }
    if data.is_empty() {
        return Err(RunTimeError::NotFound);
    }
    Ok(data[0].clone())
}
