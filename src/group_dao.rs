use std::sync::Mutex;

use crate::models::{GroupData, GroupDetails, GroupResponseData};
use crate::{database, runtime_err::RunTimeError};
use rusqlite::{Connection, Result};

pub fn save(conn: &Mutex<Connection>, data: GroupData) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    conn.execute(
        "INSERT INTO `group` (name, title, tags, auto_publish, auto_train, publish_start_time,train_start_time,publish_type,product_link) VALUES (?1, ?2, ?3, ?4, ?5, ?6,?7,?8,?9)",
        rusqlite::params![
            data.name,
            data.title,
            data.tags,
            data.auto_publish,
            data.auto_train,
            data.publish_start_time,
            data.train_start_time,
            data.publish_type,
            data.product_link,
        ],
    )?;
    Ok(())
}
pub fn update(conn: &Mutex<Connection>, data: GroupData) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    conn.execute(
        "UPDATE `group` SET name = ?1, title = ?2, tags = ?3, auto_publish = ?4, auto_train = ?5, publish_start_time = ?6, train_start_time = ?7, publish_type = ?8, product_link = ?9 WHERE id = ?10",
        rusqlite::params![
            data.name,
            data.title,
            data.tags,
            data.auto_publish,
            data.auto_train,
            data.publish_start_time,
            data.train_start_time,
            data.publish_type,
            data.product_link,
            data.id,
        ],
    )?;
    Ok(())
}
pub fn list_all() -> Result<GroupResponseData, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt = conn.prepare("SELECT id, name, title, tags, auto_publish, auto_train, publish_start_time,train_start_time,publish_type,product_link FROM `group` ORDER BY id ASC")?;
    let mut data = Vec::new();
    let group_iter = stmt.query_map((), |row| {
        Ok(GroupDetails {
            id: row.get(0)?,
            name: row.get(1)?,
            title: row.get(2)?,
            tags: row.get(3)?,
            auto_publish: row.get(4)?,
            auto_train: row.get(5)?,
            publish_start_time: row.get(6)?,
            train_start_time: row.get(7)?,
            publish_type: row.get(8)?,
            product_link: row.get(9)?,
        })
    })?;
    for group in group_iter {
        data.push(group?);
    }
    Ok(GroupResponseData { data })
}
pub fn list_all_auto_publish() -> Result<GroupResponseData, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt = conn.prepare("SELECT id, name, title, tags, auto_publish, auto_train, publish_start_time,train_start_time,publish_type,product_link FROM `group` WHERE auto_publish = 1 ORDER BY id ASC")?;
    let mut data = Vec::new();
    let group_iter = stmt.query_map((), |row| {
        Ok(GroupDetails {
            id: row.get(0)?,
            name: row.get(1)?,
            title: row.get(2)?,
            tags: row.get(3)?,
            auto_publish: row.get(4)?,
            auto_train: row.get(5)?,
            publish_start_time: row.get(6)?,
            train_start_time: row.get(7)?,
            publish_type: row.get(8)?,
            product_link: row.get(9)?,
        })
    })?;
    for group in group_iter {
        data.push(group?);
    }
    Ok(GroupResponseData { data })
}
pub fn del(id: i32) -> Result<(), RunTimeError> {
    let conn = database::get_conn()?;
    conn.execute("DELETE FROM `group` WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}
pub fn list_all_auto_train() -> Result<GroupResponseData, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt = conn.prepare("SELECT id, name, title, tags, auto_publish, auto_train, publish_start_time,train_start_time,publish_type,product_link FROM `group` WHERE auto_train = 1 ORDER BY id ASC")?;
    let mut data = Vec::new();
    let group_iter = stmt.query_map((), |row| {
        Ok(GroupDetails {
            id: row.get(0)?,
            name: row.get(1)?,
            title: row.get(2)?,
            tags: row.get(3)?,
            auto_publish: row.get(4)?,
            auto_train: row.get(5)?,
            publish_start_time: row.get(6)?,
            train_start_time: row.get(7)?,
            publish_type: row.get(8)?,
            product_link: row.get(9)?,
        })
    })?;
    for group in group_iter {
        data.push(group?);
    }
    Ok(GroupResponseData { data })
}
