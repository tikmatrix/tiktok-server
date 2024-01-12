use std::sync::Mutex;

use crate::models::{TrainJobData, TrainJobDetails, TrainJobResponseData};
use crate::{database, runtime_err::RunTimeError};
use rusqlite::{Connection, Result};

pub fn save(conn: &Mutex<Connection>, job_data: TrainJobData) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    conn.execute(
        "INSERT INTO train_job (group_id, account, click, follow, favorites, status,start_time) VALUES (?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            job_data.group_id,
            job_data.account,
            job_data.click,
            job_data.follow,
            job_data.favorites,
            job_data.status,
            job_data.start_time,
        ],
    )?;
    Ok(())
}
pub fn update(conn: &Mutex<Connection>, job_data: TrainJobData) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    conn.execute(
        "UPDATE train_job SET status = ?1 WHERE id = ?2",
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
pub fn list_all() -> Result<TrainJobResponseData, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt = conn.prepare("
    SELECT train_job.id,train_job.group_id,train_job.account,train_job.click,train_job.follow,train_job.favorites,train_job.status,train_job.start_time,train_job.end_time,account.device FROM train_job
    left join account on train_job.account = account.email
    ORDER BY train_job.id DESC
    ")?;
    let mut data = Vec::new();
    let job_iter = stmt.query_map((), |row| {
        Ok(TrainJobDetails {
            id: row.get(0)?,
            group_id: row.get(1)?,
            account: row.get(2)?,
            click: row.get(3)?,
            follow: row.get(4)?,
            favorites: row.get(5)?,
            status: row.get(6)?,
            start_time: row.get(7)?,
            end_time: row.get(8)?,
            device: row.get(9)?,
        })
    })?;
    for job in job_iter {
        data.push(job?);
    }
    Ok(TrainJobResponseData { data })
}
pub fn del(id: i32) -> Result<(), RunTimeError> {
    let conn = database::get_conn()?;
    conn.execute("DELETE FROM train_job WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}
pub fn list_runable(agent_ip: String) -> Result<TrainJobResponseData, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt = conn.prepare("
    SELECT train_job.id,train_job.group_id,train_job.account,train_job.click,train_job.follow,train_job.favorites,train_job.status,train_job.start_time,train_job.end_time,account.device FROM train_job
    left join account on train_job.account = account.email
    left join device on account.device = device.serial
    WHERE train_job.status = 0 AND device.agent_ip = ?1 
    AND train_job.start_time < datetime('now', 'localtime') 
    AND device.online = 1
    ORDER BY train_job.id ASC
    ")?;
    let mut data = Vec::new();
    let job_iter = stmt.query_map(rusqlite::params![agent_ip], |row| {
        Ok(TrainJobDetails {
            id: row.get(0)?,
            group_id: row.get(1)?,
            account: row.get(2)?,
            click: row.get(3)?,
            follow: row.get(4)?,
            favorites: row.get(5)?,
            status: row.get(6)?,
            start_time: row.get(7)?,
            end_time: row.get(8)?,
            device: row.get(9)?,
        })
    })?;
    for publish_job in job_iter {
        data.push(publish_job?);
    }
    Ok(TrainJobResponseData { data })
}

pub fn count_job_by_account_today(
    account: String,
    start_time: String,
) -> Result<i32, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt = conn.prepare(
        "
    SELECT count(*) FROM train_job
    left join account on train_job.account = account.email
    WHERE account.email = ?1 AND train_job.start_time = ?2 AND DATE(create_time) = DATE('now')
    ",
    )?;
    let mut count = 0;
    let job_iter = stmt.query_map(
        rusqlite::params![account, start_time],
        |row| Ok(row.get(0)?),
    )?;
    for job in job_iter {
        count = job?;
    }
    Ok(count)
}