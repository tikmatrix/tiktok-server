use std::sync::Mutex;

use crate::{database, models::CountGroupByStatus, runtime_err::RunTimeError};
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostCommentData {
    pub post_url: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostCommentDetails {
    pub id: i32,
    pub post_url: String,
    pub topic_count: i32,
    pub comment_count: i32,
    pub success_comment_count: i32,
    pub fail_comment_count: i32,
    pub account_count: i32,
    pub create_time: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostCommentResponseData {
    pub data: Vec<PostCommentDetails>,
}
pub fn save_post_comment(
    conn: &Mutex<Connection>,
    post_comment: PostCommentData,
) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    conn.execute(
        "INSERT INTO post_comment (post_url) VALUES (?1)",
        rusqlite::params![post_comment.post_url],
    )?;
    Ok(())
}
pub fn list_all_post_comments() -> Result<PostCommentResponseData, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt = conn.prepare(
        "
        SELECT 
            id,
            post_url,
            COALESCE(topic_count, 0) as topic_count,
            COALESCE(account_count, 0) as account_count,
            COALESCE(comment_count, 0) as comment_count,
            COALESCE(success_comment_count, 0) as success_comment_count,
            COALESCE(fail_comment_count, 0) as fail_comment_count,
            create_time 
        FROM post_comment
        LEFT JOIN (
            SELECT post_comment_id, COUNT(*) as topic_count 
            FROM post_comment_topic 
            GROUP BY post_comment_id
        ) as topic_count ON post_comment.id = topic_count.post_comment_id
        LEFT JOIN (
            SELECT post_comment_id, COUNT(DISTINCT account_id) as account_count 
            FROM post_comment_topic_comment 
            GROUP BY post_comment_id
        ) as account_count ON post_comment.id = account_count.post_comment_id
        LEFT JOIN (
            SELECT post_comment_id, COUNT(*) as comment_count 
            FROM post_comment_topic_comment 
            GROUP BY post_comment_id
        ) as comment_count ON post_comment.id = comment_count.post_comment_id
        LEFT JOIN (
            SELECT post_comment_id, COUNT(*) as success_comment_count 
            FROM post_comment_topic_comment 
            WHERE status = 2
            GROUP BY post_comment_id
        ) as success_comment_count ON post_comment.id = success_comment_count.post_comment_id
        LEFT JOIN (
            SELECT post_comment_id, COUNT(*) as fail_comment_count 
            FROM post_comment_topic_comment 
            WHERE status = 3 
            GROUP BY post_comment_id
        ) as fail_comment_count ON post_comment.id = fail_comment_count.post_comment_id
        ORDER BY create_time DESC 
        LIMIT 100
    ",
    )?;
    let post_comments = stmt
        .query_map([], |row| {
            Ok(PostCommentDetails {
                id: row.get(0)?,
                post_url: row.get(1)?,
                topic_count: row.get(2)?,
                account_count: row.get(3)?,
                comment_count: row.get(4)?,
                success_comment_count: row.get(5)?,
                fail_comment_count: row.get(6)?,
                create_time: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<PostCommentDetails>, _>>()?;
    Ok(PostCommentResponseData {
        data: post_comments,
    })
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostCommentTopicData {
    pub post_comment_id: i32,
    pub content: String,
    pub account_count: i32,
    pub comments: Vec<PostCommentTopicCommentData>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostCommentTopicCommentData {
    pub account_id: i32,
    pub content: String,
    pub status: i32,
    pub no: i32,
    pub parent_no: i32,
}
pub fn save_post_comment_topic(
    conn: &Mutex<Connection>,
    post_comment_topic: PostCommentTopicData,
) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    conn.execute(
        "INSERT INTO post_comment_topic (post_comment_id,content,account_count) VALUES (?1,?2,?3)",
        rusqlite::params![
            post_comment_topic.post_comment_id,
            post_comment_topic.content,
            post_comment_topic.account_count
        ],
    )?;
    let mut stmt = conn.prepare("SELECT last_insert_rowid()")?;
    let post_comment_topic_id: i32 = stmt.query_row([], |row| row.get(0))?;
    for comment in post_comment_topic.comments {
        conn.execute(
            "INSERT INTO post_comment_topic_comment (post_comment_id,post_comment_topic_id,account_id,content,status,no,parent_no) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            rusqlite::params![
                post_comment_topic.post_comment_id,
                post_comment_topic_id,
                comment.account_id,
                comment.content,
                comment.status,
                comment.no,
                comment.parent_no
            ],
        )?;
    }

    Ok(())
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCommentJobStatusData {
    pub id: i32,
    pub status: i32,
}

pub fn update_post_comment_topic_comment_status(
    conn: &Mutex<Connection>,
    data: UpdateCommentJobStatusData,
) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    conn.execute(
        "UPDATE post_comment_topic_comment SET status = ?1 WHERE id = ?2",
        rusqlite::params![data.status, data.id],
    )?;
    Ok(())
}
#[derive(Debug, Deserialize, Serialize)]
pub struct CommentJobDetails {
    pub id: i32,
    pub post_url: String,
    pub content: String,
    pub reply_content: Option<String>,
    pub username: String,
    pub device: Option<String>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct CommentJobResponseData {
    pub data: Vec<CommentJobDetails>,
}
pub fn list_runable_comment_jobs(agent_ip: &str) -> Result<CommentJobResponseData, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt = conn.prepare(
        "
        SELECT aa.id,aa.post_url,aa.content,aa.reply_content,aa.username,aa.device FROM
        (SELECT a.id as id,e.post_url as post_url, a.content as content,b.content as reply_content,c.username as username,c.device as device FROM post_comment_topic_comment as a
            LEFT JOIN post_comment_topic_comment as b ON a.post_comment_id=b.post_comment_id and a.post_comment_topic_id=b.post_comment_topic_id and a.parent_no = b.no
            LEFT JOIN account as c ON a.account_id = c.id
            LEFT JOIN post_comment as e ON a.post_comment_id = e.id
            WHERE ((a.status < 2 AND a.parent_no=0) OR (a.status=0 AND a.parent_no>0 AND b.status = 2 ))
        ) as aa
        LEFT JOIN device as bb ON aa.device = bb.serial
        WHERE bb.online = 1
        AND bb.agent_ip = ?1
        ORDER BY aa.id DESC LIMIT 100",
    )?;

    let comment_jobs = stmt
        .query_map([agent_ip], |row| {
            Ok(CommentJobDetails {
                id: row.get(0)?,
                post_url: row.get(1)?,
                content: row.get(2)?,
                reply_content: row.get(3)?,
                username: row.get(4)?,
                device: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<CommentJobDetails>, _>>()?;

    Ok(CommentJobResponseData { data: comment_jobs })
}

pub fn count_by_status() -> Result<Vec<CountGroupByStatus>, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt = conn.prepare(
        "
    SELECT status,count(*) FROM post_comment_topic_comment
    GROUP BY status
    ",
    )?;
    let mut data = Vec::new();
    let job_iter = stmt.query_map((), |row| {
        Ok(CountGroupByStatus {
            status: row.get(0)?,
            count: row.get(1)?,
        })
    })?;
    for job in job_iter {
        data.push(job?);
    }
    Ok(data)
}
pub fn delete_all() -> Result<(), RunTimeError> {
    let conn = database::get_conn()?;
    //truncate table
    conn.execute("DELETE FROM post_comment", rusqlite::params![])?;
    //reset autoincrement
    conn.execute(
        "DELETE FROM sqlite_sequence WHERE name='post_comment'",
        rusqlite::params![],
    )?;
    //truncate table
    conn.execute("DELETE FROM post_comment_topic", rusqlite::params![])?;
    //reset autoincrement
    conn.execute(
        "DELETE FROM sqlite_sequence WHERE name='post_comment_topic'",
        rusqlite::params![],
    )?;
    //truncate table
    conn.execute(
        "DELETE FROM post_comment_topic_comment",
        rusqlite::params![],
    )?;
    //reset autoincrement
    conn.execute(
        "DELETE FROM sqlite_sequence WHERE name='post_comment_topic_comment'",
        rusqlite::params![],
    )?;
    Ok(())
}
