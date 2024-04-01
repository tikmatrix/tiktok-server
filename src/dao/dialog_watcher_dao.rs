use std::sync::Mutex;

use rusqlite::Connection;

use crate::{
    database,
    models::{DialogWatcherData, DialogWatcherDetails, DialogWatcherResponseData},
    runtime_err::RunTimeError,
};
pub fn gen_name(conditions: String) -> String {
    let digest = md5::compute(conditions);
    format!("{:x}", digest)
}
pub fn save(conn: &Mutex<Connection>, data: DialogWatcherData) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    let name: Option<String> = Some(gen_name(data.conditions.clone().unwrap()));
    conn.execute(
        "INSERT INTO dialog_watcher (name, conditions, action, status) VALUES (?, ?, ?, ?)",
        rusqlite::params![name, data.conditions, data.action, data.status,],
    )?;
    Ok(())
}
pub fn update(conn: &Mutex<Connection>, data: DialogWatcherData) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    let name: Option<String> = Some(gen_name(data.conditions.clone().unwrap()));
    conn.execute(
        "UPDATE dialog_watcher SET name=?, conditions=?, action=?, status=? WHERE id=?",
        rusqlite::params![name, data.conditions, data.action, data.status, data.id,],
    )?;
    Ok(())
}
pub fn delete(conn: &Mutex<Connection>, id: i32) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    conn.execute(
        "DELETE FROM dialog_watcher WHERE id=?",
        rusqlite::params![id],
    )?;
    Ok(())
}
pub fn list_all() -> Result<DialogWatcherResponseData, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt =
        conn.prepare("SELECT id, name, conditions, action, status FROM dialog_watcher")?;
    let rows = stmt.query_map([], |row| {
        Ok(DialogWatcherDetails {
            id: row.get(0)?,
            name: row.get(1)?,
            conditions: row.get(2)?,
            action: row.get(3)?,
            status: row.get(4)?,
        })
    })?;
    let mut data = Vec::new();
    for row in rows {
        data.push(row?);
    }
    Ok(DialogWatcherResponseData { data })
}
