use common::track::Track;
use reqwest::{header, Client};

use crate::BASE_API_URL;

pub async fn get_all_tracks(api_token: String) -> Result<Vec<Track>, String> {
    match get_all_tracks_inner(api_token).await {
        Ok(tracks) => Ok(tracks),
        Err(e) => Err(e.to_string()),
    }
}

async fn get_all_tracks_inner(api_token: String) -> anyhow::Result<Vec<Track>> {
    let mut headers = header::HeaderMap::default();
    headers.insert("API_TOKEN", api_token.parse().unwrap());
    let client = Client::builder().default_headers(headers).build().unwrap();

    let response = client
        .get(format!("{}get_all_tracks", *BASE_API_URL))
        .send()
        .await?;

    response.error_for_status_ref()?;

    let bytes = response.bytes().await?;

    let tracks: Vec<Track> = bitcode::deserialize(&bytes)?;
    Ok(tracks)
}
