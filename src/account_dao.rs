use std::sync::Mutex;

use crate::{database, models::AccountData, runtime_err::RunTimeError};
use rusqlite::{Connection, Result};

use crate::models::{AccountDetails, AccountResponseData};

pub fn save(conn: &Mutex<Connection>, data: AccountData) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    conn.execute(
        "INSERT INTO account (email, pwd, fans, shop_creator, device,group_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            data.email,
            data.pwd,
            data.fans,
            data.shop_creator,
            data.device.unwrap(),
            data.group_id.unwrap(),
        ],
    )?;
    Ok(())
}
pub fn update(conn: &Mutex<Connection>, data: AccountData) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    conn.execute(
        "UPDATE account SET email = ?1, pwd = ?2, fans = ?3, shop_creator = ?4, device = ?5, group_id = ?6 WHERE id = ?7",
        rusqlite::params![
            data.email,
            data.pwd,
            data.fans,
            data.shop_creator,
            data.device.unwrap(),
            data.group_id.unwrap(),
            data.id.unwrap(),
        ],
    )?;
    Ok(())
}
pub fn del(email: String) -> Result<(), RunTimeError> {
    let conn = database::get_conn()?;
    conn.execute(
        "DELETE FROM account WHERE email = ?1",
        rusqlite::params![email],
    )?;
    Ok(())
}
pub fn list_all() -> Result<AccountResponseData, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt = conn.prepare(
        "
    SELECT id,email, pwd, fans, shop_creator, device,group_id FROM account
        ORDER BY id ASC;",
    )?;
    let account_iter = stmt.query_map((), |row| {
        Ok(AccountDetails {
            id: row.get(0)?,
            email: row.get(1)?,
            pwd: row.get(2)?,
            fans: row.get(3)?,
            shop_creator: row.get(4)?,
            device: row.get(5)?,
            group_id: row.get(6)?,
        })
    })?;

    let mut data = Vec::new();
    for account_result in account_iter {
        data.push(account_result?);
    }

    Ok(AccountResponseData { data })
}

pub fn list_account_by_device(device: String) -> Result<AccountResponseData, RunTimeError> {
    let conn = database::get_conn()?;

    let mut stmt = conn.prepare(
        "
    SELECT id,email, pwd, fans, shop_creator, device, group_id FROM account
        WHERE device = ?1
        ORDER BY id ASC;",
    )?;
    let mut data = Vec::new();
    let account_iter = stmt.query_map(rusqlite::params![device], |row| {
        Ok(AccountDetails {
            id: row.get(0)?,
            email: row.get(1)?,
            pwd: row.get(2)?,
            fans: row.get(3)?,
            shop_creator: row.get(4)?,
            device: row.get(5)?,
            group_id: row.get(6)?,
        })
    })?;
    for account_result in account_iter {
        data.push(account_result?);
    }
    Ok(AccountResponseData { data })
}
pub fn list_account_by_group_id(group_id: i32) -> Result<AccountResponseData, RunTimeError> {
    let conn = database::get_conn()?;

    let mut stmt = conn.prepare(
        "
    SELECT id,email, pwd, fans, shop_creator, device, group_id FROM account
        WHERE group_id = ?1
        ORDER BY id ASC;",
    )?;
    let mut data = Vec::new();
    let account_iter = stmt.query_map(rusqlite::params![group_id], |row| {
        Ok(AccountDetails {
            id: row.get(0)?,
            email: row.get(1)?,
            pwd: row.get(2)?,
            fans: row.get(3)?,
            shop_creator: row.get(4)?,
            device: row.get(5)?,
            group_id: row.get(6)?,
        })
    })?;
    for account_result in account_iter {
        data.push(account_result?);
    }
    Ok(AccountResponseData { data })
}
pub fn list_auto_train_account_by_agent_ip(
    agent_ip: String,
) -> Result<AccountResponseData, RunTimeError> {
    let conn = database::get_conn()?;

    let mut stmt = conn.prepare(
        "
    SELECT account.id,account.email, account.pwd, account.fans, account.shop_creator, account.device, account.group_id FROM account
        left join device on account.device = device.serial
        left join `group` on account.group_id = `group`.id
        WHERE device.agent_ip = ?1 AND `group`.auto_train = 1 and device.online = 1
        ORDER BY account.id ASC;",
    )?;
    let mut data = Vec::new();
    let account_iter = stmt.query_map(rusqlite::params![agent_ip], |row| {
        Ok(AccountDetails {
            id: row.get(0)?,
            email: row.get(1)?,
            pwd: row.get(2)?,
            fans: row.get(3)?,
            shop_creator: row.get(4)?,
            device: row.get(5)?,
            group_id: row.get(6)?,
        })
    })?;
    for account_result in account_iter {
        data.push(account_result?);
    }
    Ok(AccountResponseData { data })
}
