use std::sync::Mutex;

use crate::{database, runtime_err::RunTimeError};
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
        "SELECT id,post_url,COALESCE(topic_count, 0) as topic_count,COALESCE(account_count, 0) as account_count,COALESCE(comment_count, 0) as comment_count,create_time FROM post_comment
        LEFT JOIN (SELECT post_comment_id,COUNT(*) as topic_count FROM post_comment_topic GROUP BY post_comment_id) as topic_count
        ON post_comment.id = topic_count.post_comment_id
        LEFT JOIN (SELECT post_comment_id,COUNT(DISTINCT account_id) as account_count FROM post_comment_topic_comment GROUP BY post_comment_id) as account_count
        ON post_comment.id = account_count.post_comment_id
        LEFT JOIN (SELECT post_comment_id,COUNT(*) as comment_count FROM post_comment_topic_comment GROUP BY post_comment_id) as comment_count
        ON post_comment.id = comment_count.post_comment_id
        ORDER BY create_time DESC LIMIT 100
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
                create_time: row.get(5)?,
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
    pub post_comment_id: i32,
    pub post_comment_topic_id: i32,
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
                comment.post_comment_id,
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
