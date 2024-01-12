use std::sync::Mutex;

use crate::{database, runtime_err::RunTimeError};
use rusqlite::{Connection, Result};

use crate::models::{MaterialData, MaterialDetails, MaterialResponseData};

pub fn save(conn: &Mutex<Connection>, materials: Vec<MaterialData>) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    for m in materials {
        conn.execute(
            "INSERT INTO material (name, md5, group_id) VALUES (?1, ?2, ?3)",
            rusqlite::params![m.name, m.md5, m.group_id],
        )?;
    }
    Ok(())
}
pub fn update(conn: &Mutex<Connection>, name: String, used: i32) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    conn.execute(
        "UPDATE material SET used = ?1 WHERE name = ?2",
        rusqlite::params![used, name],
    )?;
    Ok(())
}
pub fn count(used: Option<i32>, group_id: Option<i32>) -> Result<i32, RunTimeError> {
    let conn = database::get_conn()?;
    let mut query = "
    SELECT count(*) FROM material where 1=1
    "
    .to_string();
    let mut params: Vec<rusqlite::types::Value> = Vec::new();
    if let Some(used_value) = used {
        query.push_str(" AND used = ?1");
        params.push(used_value.into());
    }
    if let Some(group_id_value) = group_id {
        query.push_str(" AND group_id = ?2");
        params.push(group_id_value.into());
    }
    let mut stmt = conn.prepare(&query)?;
    let mut count = 0;
    let material_iter =
        stmt.query_map(rusqlite::params_from_iter(params), |row| Ok(row.get(0)?))?;
    for material in material_iter {
        count = material?;
    }
    Ok(count)
}
pub fn list(
    used: Option<i32>,
    group_id: Option<i32>,
) -> Result<MaterialResponseData, RunTimeError> {
    let conn = database::get_conn()?;
    let mut query = "
    SELECT id,name, md5, used, group_id FROM material
    "
    .to_string();
    let mut params: Vec<rusqlite::types::Value> = Vec::new();
    if let Some(used_value) = used {
        query.push_str(" WHERE used = ?1");
        params.push(used_value.into());
    }
    if let Some(group_id_value) = group_id {
        query.push_str(" WHERE group_id = ?2");
        params.push(group_id_value.into());
    }
    query.push_str(" ORDER BY id DESC");
    let mut stmt = conn.prepare(&query)?;
    let mut data = Vec::new();
    let material_iter = stmt.query_map(rusqlite::params_from_iter(params), |row| {
        Ok(MaterialDetails {
            id: row.get(0)?,
            name: row.get(1)?,
            md5: row.get(2)?,
            used: row.get(3)?,
            group_id: row.get(4)?,
        })
    })?;
    for material in material_iter {
        data.push(material?);
    }
    Ok(MaterialResponseData { data })
}
pub fn del(id: i32) -> Result<(), RunTimeError> {
    let conn = database::get_conn()?;
    conn.execute("DELETE FROM material WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}
pub fn get_and_use_one(
    conn: &Mutex<Connection>,
    group_id: i32,
) -> Result<MaterialDetails, RunTimeError> {
    let query_conn = database::get_conn()?;
    let mut stmt = query_conn.prepare(
        "
    SELECT id,name, md5, used, group_id FROM material
    WHERE used = 0 AND group_id = ?1
    ORDER BY id ASC LIMIT 1
    ",
    )?;
    let mut material_iter = stmt.query_map(rusqlite::params![group_id], |row| {
        Ok(MaterialDetails {
            id: row.get(0)?,
            name: row.get(1)?,
            md5: row.get(2)?,
            used: row.get(3)?,
            group_id: row.get(4)?,
        })
    })?;
    if let Some(material) = material_iter.next() {
        let material = material?;
        update(&conn, material.name.clone(), 1)?;
        return Ok(material);
    }
    Err(RunTimeError::NotFound)
}
