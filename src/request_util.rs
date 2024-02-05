use serde::de::DeserializeOwned;

use crate::runtime_err::RunTimeError;

pub async fn get_json<T: DeserializeOwned>(host: &str, url_path: &str) -> Result<T, RunTimeError> {
    let response = match reqwest::get(format!("http://{}:8091{}", host, url_path)).await {
        Ok(response) => response,
        Err(e) => {
            log::error!("Failed to send request: {:?}", e);
            return Err(e.into());
        }
    };

    let response_status = response.status();
    let text = response
        .text()
        .await
        .unwrap_or_else(|_| String::from("Failed to read response text"));
    log::debug!("response_status: {:?}, text: {:?}", response_status, text);
    let json: T = match serde_json::from_str(&text) {
        Ok(json) => json,
        Err(e) => {
            log::error!(
                "Failed to parse response as JSON: {:?},response_status:{:?}",
                text,
                response_status
            );
            return Err(e.into());
        }
    };

    Ok(json)
}

const URL: &str = "https://tiktok.niostack.com";

pub fn get_json_api<T: DeserializeOwned>(url_path: &str) -> Result<T, RunTimeError> {
    let mut url = String::from(URL);
    if cfg!(debug_assertions) {
        url = String::from("http://localhost:8095");
    }
    let response = match reqwest::blocking::get(format!("{}{}", url, url_path)) {
        Ok(response) => response,
        Err(e) => {
            return Err(e.into());
        }
    };

    let text = response
        .text()
        .unwrap_or_else(|_| String::from("Failed to read response text"));
    let json: T = match serde_json::from_str(&text) {
        Ok(json) => json,
        Err(e) => {
            return Err(e.into());
        }
    };

    Ok(json)
}
