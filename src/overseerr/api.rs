use color_eyre::Result;
use serde::de::DeserializeOwned;

use super::responses::RequestResponse;
use crate::config::Config;

pub async fn get<T>(path: &str) -> Result<RequestResponse<T>>
where
    T: DeserializeOwned,
{
    let client = reqwest::Client::new();
    let config = Config::global();

    let response_data: RequestResponse<T> = client
        .get(format!("{}/api/v1{}", &config.overseerr.url, path))
        .header("X-API-Key", &config.overseerr.api_key)
        .send()
        .await?
        .json()
        .await?;

    Ok(response_data)
}
