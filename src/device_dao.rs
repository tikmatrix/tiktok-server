use std::{
    fs::File,
    io::Read,
    sync::{mpsc::Sender, Arc, Mutex},
    vec,
};

use crate::{
    aes_util, database,
    ddl_actor::DdlMessage,
    models::{DeviceData, DeviceDetails, DeviceResponseData, LicenseDetails},
    runtime_err::RunTimeError,
};
use local_ip_address::local_ip;
use rusqlite::{types::Value, Connection, Result};

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
    SELECT id,serial, forward_port, online, ip,  agent_ip, init
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

    query.push_str(" ORDER BY serial ASC");
    if !is_license_valid() {
        query.push_str(" LIMIT 1 ");
    }
    let mut stmt = conn.prepare(&query)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params), |row| {
        Ok(DeviceDetails {
            id: row.get(0)?,
            serial: row.get(1)?,
            forward_port: row.get(2)?,
            online: row.get(3)?,
            ip: row.get(4)?,
            agent_ip: row.get(5)?,
            init: row.get(6)?,
        })
    })?;

    let mut devices: Vec<DeviceDetails> = Vec::new();
    for device in rows {
        devices.push(device?);
    }
    Ok(DeviceResponseData { data: devices })
}
pub fn is_license_valid() -> bool {
    let file = File::open(".license");
    if file.is_err() {
        return false;
    }
    let mut file = file.unwrap();
    let mut code = String::new();
    file.read_to_string(&mut code).unwrap();
    let code = code.trim().to_string();
    let license = aes_util::aes_decrypt(&code);
    if license.is_empty() {
        return false;
    }
    let license_details: LicenseDetails = serde_json::from_str(&license).unwrap();
    if license_details.expire > chrono::Local::now().timestamp() {
        return true;
    }
    false
}

pub fn save(
    ddl_sender: &Arc<Mutex<Sender<DdlMessage>>>,
    device_data: DeviceData,
) -> Result<bool, RunTimeError> {
    // let _lock = conn.lock();
    let conn = database::get_conn()?;
    //根据serial查询是否存在
    let mut stmt: rusqlite::Statement<'_> =
        conn.prepare("SELECT serial FROM device WHERE serial = ?1;")?;
    let mut rows = stmt.query(rusqlite::params![device_data.serial])?;
    if let Some(_row) = rows.next()? {
        // let start_time = chrono::Local::now();
        log::debug!("device {} already exists", device_data.serial);
        //存在则更新
        ddl_sender
            .lock()
            .unwrap()
            .send(DdlMessage {
                sql: "UPDATE device SET forward_port = ?1, online = ?2, ip = ?3, agent_ip = ?4
                WHERE serial = ?5"
                    .to_string(),
                params: vec![
                    Value::Integer(device_data.forward_port as i64),
                    Value::Integer(device_data.online as i64),
                    Value::Text(device_data.ip.unwrap_or("".to_string())),
                    Value::Text(device_data.agent_ip),
                    Value::Text(device_data.serial),
                ],
            })
            .unwrap();
        return Ok(false);
    }
    //不存在则插入
    log::info!("device {} not exists", device_data.serial);
    let master_ip = local_ip().unwrap().to_string();
    ddl_sender
        .lock()
        .unwrap()
        .send(DdlMessage {
            sql: "INSERT INTO device (serial, forward_port, online, ip, agent_ip, master_ip)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
                .to_string(),
            params: vec![
                Value::Text(device_data.serial),
                Value::Integer(device_data.forward_port as i64),
                Value::Integer(device_data.online as i64),
                Value::Text(device_data.ip.unwrap_or("".to_string())),
                Value::Text(device_data.agent_ip),
                Value::Text(master_ip),
            ],
        })
        .unwrap();
    Ok(true)
}
