use std::sync::Mutex;

use crate::{database, models::AccountData, runtime_err::RunTimeError};
use rusqlite::{Connection, Result};

use crate::models::{AccountDetails, AccountResponseData};

pub fn save(conn: &Mutex<Connection>, data: AccountData) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    conn.execute(
        "INSERT INTO account (email, pwd, fans, device,group_id,username) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            data.email,
            data.pwd,
            data.fans,
            data.device.unwrap_or_default(),
            data.group_id.unwrap_or_default(),
            data.username.unwrap_or_default(),
        ],
    )?;
    Ok(())
}
pub fn update(conn: &Mutex<Connection>, data: AccountData) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    //get by id
    let mut stmt = conn.prepare(
        "select device, email, fans, group_id, id, pwd, username from account where id = ?1",
    )?;
    let mut account_iter = stmt.query_map(rusqlite::params![data.id.unwrap()], |row| {
        Ok(AccountDetails {
            device: row.get(0)?,
            email: row.get(1)?,
            fans: row.get(2)?,
            group_id: row.get(3)?,
            id: row.get(4)?,
            pwd: row.get(5)?,
            username: row.get(6)?,
        })
    })?;
    let mut account = account_iter.next().unwrap().unwrap();
    if data.email != "" {
        account.email = data.email;
    }
    if data.pwd != "" {
        account.pwd = data.pwd;
    }
    if data.fans != 0 {
        account.fans = data.fans;
    }

    if data.device != None {
        account.device = data.device;
    }
    if data.group_id != None {
        account.group_id = data.group_id;
    }
    if data.username != None {
        account.username = data.username;
    }

    conn.execute(
        "UPDATE account SET device = ?1, email = ?2, fans = ?3, 
         group_id = ?4, id = ?5, pwd = ?6, username = ?7 
         WHERE id = ?5",
        rusqlite::params![
            account.device.unwrap_or_default(),
            account.email,
            account.fans,
            account.group_id.unwrap_or_default(),
            account.id,
            account.pwd,
            account.username.unwrap_or_default(),
        ],
    )?;
    Ok(())
}
pub fn del(id: String) -> Result<(), RunTimeError> {
    let conn = database::get_conn()?;
    conn.execute("DELETE FROM account WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}
pub fn list_all() -> Result<AccountResponseData, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt = conn.prepare(
        "
        SELECT device, email, fans, group_id, id, pwd, username FROM account
            ORDER BY id ASC;",
    )?;
    let account_iter = stmt.query_map((), |row| {
        Ok(AccountDetails {
            device: row.get(0)?,
            email: row.get(1)?,
            fans: row.get(2)?,
            group_id: row.get(3)?,
            id: row.get(4)?,
            pwd: row.get(5)?,
            username: row.get(6)?,
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
    SELECT device, email, fans, group_id, id, pwd, username FROM account
        WHERE device = ?1
        ORDER BY id DESC;",
    )?;
    let mut data = Vec::new();
    let account_iter = stmt.query_map(rusqlite::params![device], |row| {
        Ok(AccountDetails {
            device: row.get(0)?,
            email: row.get(1)?,
            fans: row.get(2)?,
            group_id: row.get(3)?,
            id: row.get(4)?,
            pwd: row.get(5)?,
            username: row.get(6)?,
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
    SELECT device, email, fans, group_id, id, pwd, username FROM account
        WHERE group_id = ?1
        ORDER BY id ASC;",
    )?;
    let mut data = Vec::new();
    let account_iter = stmt.query_map(rusqlite::params![group_id], |row| {
        Ok(AccountDetails {
            device: row.get(0)?,
            email: row.get(1)?,
            fans: row.get(2)?,
            group_id: row.get(3)?,
            id: row.get(4)?,
            pwd: row.get(5)?,
            username: row.get(6)?,
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
    SELECT account.device, account.email, account.fans, account.group_id, 
    account.id, account.pwd, account.username FROM account
        left join device on account.device = device.serial
        left join `group` on account.group_id = `group`.id
        WHERE device.agent_ip = ?1 AND `group`.auto_train = 1 and device.online = 1
        ORDER BY account.id ASC;",
    )?;
    let mut data = Vec::new();
    let account_iter = stmt.query_map(rusqlite::params![agent_ip], |row| {
        Ok(AccountDetails {
            device: row.get(0)?,
            email: row.get(1)?,
            fans: row.get(2)?,
            group_id: row.get(3)?,
            id: row.get(4)?,
            pwd: row.get(5)?,
            username: row.get(6)?,
        })
    })?;
    for account_result in account_iter {
        data.push(account_result?);
    }
    Ok(AccountResponseData { data })
}
pub fn update_username(
    conn: &Mutex<Connection>,
    old_username: &str,
    new_username: &str,
) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    conn.execute(
        "UPDATE account SET username = ?1 WHERE username = ?2",
        rusqlite::params![new_username, old_username],
    )?;
    Ok(())
}
pub fn update_username_device(
    conn: &Mutex<Connection>,
    username: &str,
    device: &str,
) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn().unwrap();
    let result = conn
        .execute(
            "UPDATE account SET device = ?1 WHERE username = ?2",
            rusqlite::params![device, username],
        )
        .unwrap();
    if result == 0 {
        //insert
        conn.execute(
            "INSERT INTO account (email, pwd, fans, device,group_id,username) VALUES ('', '', 0, ?1, 1, ?2)",
            rusqlite::params![
                device,username
            ],
        )?;
    }
    Ok(())
}
pub fn count_all() -> Result<i32, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt = conn.prepare("SELECT count(*) FROM account")?;
    let count = stmt.query_row([], |row| row.get(0))?;
    Ok(count)
}
pub fn count_account_by_group_id(group_id: i32) -> Result<i32, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt = conn.prepare("SELECT count(*) FROM account WHERE group_id = ?1")?;
    let count = stmt.query_row([group_id], |row| row.get(0))?;
    Ok(count)
}
