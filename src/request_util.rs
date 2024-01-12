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
            log::error!("Failed to parse response as JSON: {:?}", e);
            return Err(e.into());
        }
    };

    Ok(json)
}
