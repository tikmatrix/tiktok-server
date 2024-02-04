use std::sync::Mutex;

use crate::models::{PublishJobData, PublishJobDetails, PublishJobResponseData};
use crate::{database, runtime_err::RunTimeError};
use rusqlite::{Connection, Result};

pub fn save(conn: &Mutex<Connection>, job_data: PublishJobData) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    conn.execute(
        "INSERT INTO publish_job (material, account, title, tags, status, start_time,publish_type,product_link,group_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?,?)",
        rusqlite::params![
            job_data.material,
            job_data.account,
            job_data.title,
            job_data.tags,
            job_data.status,
            job_data.start_time,
            job_data.publish_type,
            job_data.product_link,
            job_data.group_id,
        ],
    )?;
    Ok(())
}
pub fn update(conn: &Mutex<Connection>, job_data: PublishJobData) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    conn.execute(
        "UPDATE publish_job SET status = ?1 WHERE id = ?2",
        rusqlite::params![job_data.status, job_data.id,],
    )?;
    if job_data.status.unwrap() == 2 {
        //update end_time
        let end_time = chrono::Local::now().naive_local();
        //convert to string
        let end_time = end_time.format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "UPDATE train_job SET end_time = ?1 WHERE id = ?2",
            rusqlite::params![end_time, job_data.id,],
        )?;
    }
    Ok(())
}
pub fn list_all() -> Result<PublishJobResponseData, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt = conn.prepare("
    SELECT publish_job.id,publish_job.material, publish_job.account, publish_job.title, publish_job.tags, publish_job.status, 
    publish_job.start_time,publish_job.end_time,account.device,publish_job.group_id,
    publish_job.publish_type,publish_job.product_link,account.username
    FROM publish_job
    left join account on publish_job.account = account.email
    ORDER BY publish_job.id DESC LIMIT 200
    ")?;
    let mut data = Vec::new();
    let job_iter = stmt.query_map((), |row| {
        Ok(PublishJobDetails {
            id: row.get(0)?,
            material: row.get(1)?,
            account: row.get(2)?,
            title: row.get(3)?,
            tags: row.get(4)?,
            status: row.get(5)?,
            start_time: row.get(6)?,
            end_time: row.get(7)?,
            device: row.get(8)?,
            group_id: row.get(9)?,
            publish_type: row.get(10)?,
            product_link: row.get(11)?,
            username: row.get(12)?,
        })
    })?;
    for publish_job in job_iter {
        data.push(publish_job?);
    }
    Ok(PublishJobResponseData { data })
}
pub fn del(id: i32) -> Result<(), RunTimeError> {
    let conn = database::get_conn()?;
    conn.execute(
        "DELETE FROM publish_job WHERE id = ?1",
        rusqlite::params![id],
    )?;
    Ok(())
}
pub fn list_runable(agent_ip: String) -> Result<PublishJobResponseData, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt = conn.prepare("
    SELECT publish_job.id,publish_job.material, publish_job.account, publish_job.title, publish_job.tags, 
    publish_job.status, publish_job.start_time,publish_job.end_time,account.device,publish_job.group_id,
    publish_job.publish_type,publish_job.product_link,account.username
    FROM publish_job
    left join account on publish_job.account = account.email
    left join device on account.device = device.serial
    WHERE publish_job.status = 0 AND device.agent_ip = ?1 
    AND publish_job.start_time < datetime('now', 'localtime') 
    AND device.online = 1
    ORDER BY publish_job.id ASC
    ")?;
    let mut data = Vec::new();
    let job_iter = stmt.query_map(rusqlite::params![agent_ip], |row| {
        Ok(PublishJobDetails {
            id: row.get(0)?,
            material: row.get(1)?,
            account: row.get(2)?,
            title: row.get(3)?,
            tags: row.get(4)?,
            status: row.get(5)?,
            start_time: row.get(6)?,
            end_time: row.get(7)?,
            device: row.get(8)?,
            group_id: row.get(9)?,
            publish_type: row.get(10)?,
            product_link: row.get(11)?,
            username: row.get(12)?,
        })
    })?;
    for publish_job in job_iter {
        data.push(publish_job?);
    }
    Ok(PublishJobResponseData { data })
}

pub fn count_job_by_account_today(
    account: String,
    start_time: String,
) -> Result<i32, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt = conn.prepare(
        "
    SELECT count(*) FROM publish_job
    left join account on publish_job.account = account.email
    WHERE account.email = ?1 AND publish_job.start_time = ?2 AND DATE(create_time) = DATE('now')
    ",
    )?;
    let mut count = 0;
    let job_iter = stmt.query_map(
        rusqlite::params![account, start_time],
        |row| Ok(row.get(0)?),
    )?;
    for publish_job in job_iter {
        count = publish_job?;
    }
    Ok(count)
}
