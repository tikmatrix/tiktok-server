use std::{
    sync::{mpsc::Sender, Arc, Mutex},
    vec,
};

use crate::{database, ddl_actor::DdlMessage, runtime_err::RunTimeError};
use rusqlite::{types::Value, Connection, Result};
use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DataAnalytics {
    pub id: i32,
    pub username: String,
    pub day_hour: String,
    pub follower_count: i32,
    pub video_count: i32,
    pub video_collect_count: i32,
    pub video_comment_count: i32,
    pub video_like_count: i32,
    pub video_play_count: i32,
}
pub fn save(
    ddl_sender: &Arc<Mutex<Sender<DdlMessage>>>,
    data_analytics: DataAnalytics,
) -> Result<bool, RunTimeError> {
    ddl_sender
        .lock()
        .unwrap()
        .send(DdlMessage {
            sql: "INSERT INTO data_analytics(username, day_hour, follower_count, video_count, video_collect_count, video_comment_count, video_like_count, video_play_count)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
                .to_string(),
            params: vec![
                Value::Text(data_analytics.username),
                Value::Text(data_analytics.day_hour),
                Value::Integer(data_analytics.follower_count as i64),
                Value::Integer(data_analytics.video_count as i64),
                Value::Integer(data_analytics.video_collect_count as i64),
                Value::Integer(data_analytics.video_comment_count as i64),
                Value::Integer(data_analytics.video_like_count as i64),
                Value::Integer(data_analytics.video_play_count as i64),
            ],
        })
        .unwrap();
    Ok(true)
}
pub fn list_all() -> Result<Vec<DataAnalytics>, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt = conn.prepare(
        "SELECT 0 AS id,
    username,
    day_hour,
    MAX(follower_count) AS follower_count,
    MAX(video_count) AS video_count,
    MAX(video_collect_count) AS video_collect_count,
    MAX(video_comment_count) AS video_comment_count,
    MAX(video_like_count) AS video_like_count,
    MAX(video_play_count) AS video_play_count
     FROM data_analytics group by username,day_hour
     ORDER BY day_hour,MAX(video_play_count) DESC LIMIT 3000",
    )?;
    let data_analytics = stmt
        .query_map([], |row| {
            Ok(DataAnalytics {
                id: row.get(0)?,
                username: row.get(1)?,
                day_hour: row.get(2)?,
                follower_count: row.get(3)?,
                video_count: row.get(4)?,
                video_collect_count: row.get(5)?,
                video_comment_count: row.get(6)?,
                video_like_count: row.get(7)?,
                video_play_count: row.get(8)?,
            })
        })?
        .map(|row| row.unwrap())
        .collect();
    Ok(data_analytics)
}
