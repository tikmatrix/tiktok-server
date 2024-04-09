use std::{
    sync::{mpsc::Sender, Arc, Mutex},
    vec,
};

use crate::{
    database,
    ddl_actor::DdlMessage,
    models::{DeviceData, DeviceDetails, DeviceResponseData},
    runtime_err::RunTimeError,
};
use local_ip_address::local_ip;
use rusqlite::{types::Value, Connection, Result};
use serde::{Deserialize, Serialize};

pub fn update_init(
    conn: &Mutex<Connection>,
    serial: &String,
    init: &i32,
) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    // let conn = conn.lock().unwrap();
    conn.execute(
        "UPDATE device SET init = ?1 WHERE serial = ?2",
        rusqlite::params![init, serial],
    )?;
    Ok(())
}
pub fn update_online(serial: &String, online: &i32) -> Result<(), RunTimeError> {
    let conn = database::get_conn()?;
    conn.execute(
        "UPDATE device SET online = ?1 WHERE serial = ?2",
        rusqlite::params![online, serial],
    )?;
    Ok(())
}
pub fn list_online_device(
    serial: Option<String>,
    agent_ip: Option<String>,
) -> Result<DeviceResponseData, RunTimeError> {
    let conn = database::get_conn()?;
    let mut query = "
    SELECT id,serial, online, agent_ip, init
    FROM device 
    WHERE online = 1
"
    .to_string();

    let mut params: Vec<rusqlite::types::Value> = Vec::new();

    if let Some(serial) = serial {
        query.push_str(" AND serial = ? ");
        params.push(rusqlite::types::Value::Text(serial));
    }

    if let Some(agent_ip) = agent_ip {
        query.push_str(" AND agent_ip = ? ");
        params.push(rusqlite::types::Value::Text(agent_ip));
    }
    let adb_mode = std::env::var("ADB_MODE").unwrap_or(String::from("USB"));
    if adb_mode == "TCP" {
        query.push_str(" AND serial LIKE '%:%' ");
    } else {
        query.push_str(" AND serial NOT LIKE '%:%' ");
    }
    query.push_str(" ORDER BY serial ASC");

    let mut stmt = conn.prepare(&query)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params), |row| {
        Ok(DeviceDetails {
            id: row.get(0)?,
            serial: row.get(1)?,
            online: row.get(2)?,
            agent_ip: row.get(3)?,
            init: row.get(4)?,
        })
    })?;

    let mut devices: Vec<DeviceDetails> = Vec::new();
    for device in rows {
        devices.push(device?);
    }
    Ok(DeviceResponseData { data: devices })
}

fn is_tcp_connection(serial: &str) -> bool {
    serial.contains(":")
}
pub fn save(
    ddl_sender: &Arc<Mutex<Sender<DdlMessage>>>,
    device_data: DeviceData,
) -> Result<bool, RunTimeError> {
    // let _lock = conn.lock();
    let conn = database::get_conn()?;
    //需要根据serial判断是usb连接还是tcp连接
    let mut exists_id = 0;
    let mut stmt = conn.prepare("SELECT id FROM device WHERE serial = ?1;")?;
    let mut rows = stmt.query(rusqlite::params![device_data.serial])?;
    while let Some(row) = rows.next()? {
        exists_id = row.get(0)?;
    }
    if exists_id > 0 {
        log::debug!("device {} already exists", device_data.serial);
        //存在则更新
        ddl_sender
            .lock()
            .unwrap()
            .send(DdlMessage {
                sql: "UPDATE device SET online = ?1, agent_ip = ?2, serial = ?3
                WHERE id = ?4"
                    .to_string(),
                params: vec![
                    Value::Integer(device_data.online as i64),
                    Value::Text(device_data.agent_ip),
                    Value::Text(device_data.serial),
                    Value::Integer(exists_id),
                ],
            })
            .unwrap();
        return Ok(false);
    }
    //不存在则插入
    log::info!("device {} not exists", device_data.serial);
    let master_ip = local_ip().unwrap().to_string();
    let mut init = 0;
    if is_tcp_connection(&device_data.serial) {
        //切换tcp不用重新初始化
        init = 1;
    }
    ddl_sender
        .lock()
        .unwrap()
        .send(DdlMessage {
            sql: "INSERT INTO device (serial, online, agent_ip, master_ip, init)
        VALUES (?1, ?2, ?3, ?4, ?5)"
                .to_string(),
            params: vec![
                Value::Text(device_data.serial),
                Value::Integer(device_data.online as i64),
                Value::Text(device_data.agent_ip),
                Value::Text(master_ip),
                Value::Integer(init),
            ],
        })
        .unwrap();
    Ok(true)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub ip: String,
    pub count: i32,
}
pub fn list_online_agent() -> Result<Vec<Node>, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt =
        conn.prepare("SELECT agent_ip, count(*) as count FROM device GROUP BY agent_ip")?;
    let rows = stmt.query_map((), |row| {
        Ok(Node {
            ip: row.get(0)?,
            count: row.get(1)?,
        })
    })?;
    let mut nodes: Vec<Node> = Vec::new();
    for node in rows {
        nodes.push(node?);
    }
    Ok(nodes)
}
pub fn count_online_device() -> Result<i32, RunTimeError> {
    let conn = database::get_conn()?;
    let adb_mode = std::env::var("ADB_MODE").unwrap_or(String::from("USB"));
    let mut query = "
    SELECT count(*) as count
    FROM device 
    WHERE online = 1
"
    .to_string();
    if adb_mode == "TCP" {
        query.push_str(" AND serial LIKE '%:%' ");
    } else {
        query.push_str(" AND serial NOT LIKE '%:%' ");
    }
    let mut stmt = conn.prepare(&query)?;
    let mut rows = stmt.query(rusqlite::params![])?;
    let mut count = 0;
    while let Some(row) = rows.next()? {
        count = row.get(0)?;
    }
    Ok(count)
}
