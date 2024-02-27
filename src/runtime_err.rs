use derive_more::{Display, From};
use reqwest::StatusCode;
#[derive(Debug, Display, From)]
pub enum RunTimeError {
    ReqwestError(reqwest::Error),
    DatabaseError(rusqlite::Error),
    SerdeError(serde_json::Error),
    CustomError(String),
    NotFound,
}
impl RunTimeError {
    pub(crate) fn new(arg: &str) -> RunTimeError {
        RunTimeError::CustomError(arg.to_string())
    }
}
impl actix_web::ResponseError for RunTimeError {
    fn status_code(&self) -> StatusCode {
        match self {
            RunTimeError::NotFound => StatusCode::NOT_FOUND,
            RunTimeError::SerdeError(_)
            | RunTimeError::DatabaseError(_)
            | RunTimeError::ReqwestError(_)
            | RunTimeError::CustomError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
