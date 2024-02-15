use std::sync::Mutex;

use crate::models::{GroupData, GroupDetails, GroupResponseData};
use crate::{database, runtime_err::RunTimeError};
use rusqlite::{Connection, Result};

pub fn save(conn: &Mutex<Connection>, data: GroupData) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    conn.execute(
        "INSERT INTO `group` (name, title,  auto_publish, auto_train, publish_start_time,train_start_time,publish_type,product_link, floow_probable, like_probable, collect_probable) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6,?7,?8,?9,?10,?11)",
        rusqlite::params![
            data.name,
            data.title,
            data.auto_publish,
            data.auto_train,
            data.publish_start_time,
            data.train_start_time,
            data.publish_type,
            data.product_link,
            data.floow_probable,
            data.like_probable,
            data.collect_probable,
        ],
    )?;
    Ok(())
}
pub fn update(conn: &Mutex<Connection>, data: GroupData) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    conn.execute(
        "UPDATE `group` SET name = ?1, title = ?2,auto_publish = ?3, auto_train = ?4, 
        publish_start_time = ?5, train_start_time = ?6, publish_type = ?7, product_link = ?8, floow_probable = ?9, like_probable = ?10, collect_probable = ?11 WHERE id = ?12",
        rusqlite::params![
            data.name,
            data.title,
            data.auto_publish,
            data.auto_train,
            data.publish_start_time,
            data.train_start_time,
            data.publish_type,
            data.product_link,
            data.floow_probable,
            data.like_probable,
            data.collect_probable,
            data.id,
        ],
    )?;
    Ok(())
}
pub fn list_all() -> Result<GroupResponseData, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt = conn.prepare("SELECT id, name, title,  auto_publish, auto_train, publish_start_time,train_start_time,publish_type,product_link, floow_probable, like_probable, collect_probable
     FROM `group` ORDER BY id ASC")?;
    let mut data = Vec::new();
    let group_iter = stmt.query_map((), |row| {
        Ok(GroupDetails {
            id: row.get(0)?,
            name: row.get(1)?,
            title: row.get(2)?,
            auto_publish: row.get(3)?,
            auto_train: row.get(4)?,
            publish_start_time: row.get(5)?,
            train_start_time: row.get(6)?,
            publish_type: row.get(7)?,
            product_link: row.get(8)?,
            floow_probable: row.get(9)?,
            like_probable: row.get(10)?,
            collect_probable: row.get(11)?,
        })
    })?;
    for group in group_iter {
        data.push(group?);
    }
    Ok(GroupResponseData { data })
}
pub fn list_all_auto_publish() -> Result<GroupResponseData, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt = conn.prepare("SELECT id, name, title, auto_publish, auto_train, publish_start_time,train_start_time,publish_type,product_link, floow_probable, like_probable, collect_probable
     FROM `group` WHERE auto_publish = 1 ORDER BY id ASC")?;
    let mut data = Vec::new();
    let group_iter = stmt.query_map((), |row| {
        Ok(GroupDetails {
            id: row.get(0)?,
            name: row.get(1)?,
            title: row.get(2)?,
            auto_publish: row.get(3)?,
            auto_train: row.get(4)?,
            publish_start_time: row.get(5)?,
            train_start_time: row.get(6)?,
            publish_type: row.get(7)?,
            product_link: row.get(8)?,
            floow_probable: row.get(9)?,
            like_probable: row.get(10)?,
            collect_probable: row.get(11)?,
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
    let mut stmt = conn.prepare("SELECT id, name, title,  auto_publish, auto_train, publish_start_time,train_start_time,publish_type,product_link, floow_probable, like_probable, collect_probable
     FROM `group` WHERE auto_train = 1 ORDER BY id ASC")?;
    let mut data = Vec::new();
    let group_iter = stmt.query_map((), |row| {
        Ok(GroupDetails {
            id: row.get(0)?,
            name: row.get(1)?,
            title: row.get(2)?,
            auto_publish: row.get(3)?,
            auto_train: row.get(4)?,
            publish_start_time: row.get(5)?,
            train_start_time: row.get(6)?,
            publish_type: row.get(7)?,
            product_link: row.get(8)?,
            floow_probable: row.get(9)?,
            like_probable: row.get(10)?,
            collect_probable: row.get(11)?,
        })
    })?;
    for group in group_iter {
        data.push(group?);
    }
    Ok(GroupResponseData { data })
}
