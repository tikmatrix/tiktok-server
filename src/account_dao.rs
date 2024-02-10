use std::sync::Mutex;

use crate::{database, models::AccountData, runtime_err::RunTimeError};
use rusqlite::{Connection, Result};

use crate::models::{AccountDetails, AccountResponseData};

pub fn save(conn: &Mutex<Connection>, data: AccountData) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    conn.execute(
        "INSERT INTO account (email, pwd, fans, shop_creator, device,group_id,username) VALUES (?1, ?2, ?3, ?4, ?5, ?6,?7)",
        rusqlite::params![
            data.email,
            data.pwd,
            data.fans,
            data.shop_creator,
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
        "select id,email, pwd, fans, shop_creator, device,group_id,username,earnings,today_sales,today_sold_items,today_orders from account where id = ?1",
    )?;
    let mut account_iter = stmt.query_map(rusqlite::params![data.id.unwrap()], |row| {
        Ok(AccountDetails {
            id: row.get(0)?,
            email: row.get(1)?,
            pwd: row.get(2)?,
            fans: row.get(3)?,
            shop_creator: row.get(4)?,
            device: row.get(5)?,
            group_id: row.get(6)?,
            username: row.get(7)?,
            earnings: row.get(8)?,
            today_sales: row.get(9)?,
            today_sold_items: row.get(10)?,
            today_orders: row.get(11)?,
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
    if data.shop_creator != 0 {
        account.shop_creator = data.shop_creator;
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
    if data.earnings != None {
        account.earnings = data.earnings.unwrap();
    }
    if data.today_sales != None {
        account.today_sales = data.today_sales.unwrap();
    }
    if data.today_sold_items != None {
        account.today_sold_items = data.today_sold_items.unwrap();
    }
    if data.today_orders != None {
        account.today_orders = data.today_orders.unwrap();
    }

    conn.execute(
        "UPDATE account SET email = ?1, pwd = ?2, fans = ?3, shop_creator = ?4,
         device = ?5, group_id = ?6, username = ?7 ,earnings = ?8,today_sales = ?9,
         today_sold_items = ?10,today_orders = ?11 
         WHERE id = ?12",
        rusqlite::params![
            account.email,
            account.pwd,
            account.fans,
            account.shop_creator,
            account.device.unwrap_or_default(),
            account.group_id.unwrap_or_default(),
            account.username.unwrap_or_default(),
            account.earnings,
            account.today_sales,
            account.today_sold_items,
            account.today_orders,
            account.id,
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
    SELECT id,email, pwd, fans, shop_creator, device,group_id,username,earnings,today_sales,today_sold_items,today_orders FROM account
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
            username: row.get(7)?,
            earnings: row.get(8)?,
            today_sales: row.get(9)?,
            today_sold_items: row.get(10)?,
            today_orders: row.get(11)?,
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
    SELECT id,email, pwd, fans, shop_creator, device, group_id,username,earnings,today_sales,today_sold_items,today_orders FROM account
        WHERE device = ?1
        ORDER BY id DESC;",
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
            username: row.get(7)?,
            earnings: row.get(8)?,
            today_sales: row.get(9)?,
            today_sold_items: row.get(10)?,
            today_orders: row.get(11)?,
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
    SELECT id,email, pwd, fans, shop_creator, device, group_id,username,earnings,today_sales,today_sold_items,today_orders FROM account
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
            username: row.get(7)?,
            earnings: row.get(8)?,
            today_sales: row.get(9)?,
            today_sold_items: row.get(10)?,
            today_orders: row.get(11)?,
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
    SELECT account.id,account.email, account.pwd, account.fans, account.shop_creator,
     account.device, account.group_id, account.username,account.earnings,account.today_sales,
     account.today_sold_items,account.today_orders FROM account
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
            username: row.get(7)?,
            earnings: row.get(8)?,
            today_sales: row.get(9)?,
            today_sold_items: row.get(10)?,
            today_orders: row.get(11)?,
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
