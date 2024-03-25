use anyhow::Context;
use common::{candidate::Candidate, track::Track};
use leptos::{Signal, SignalGet, SignalGetUntracked, SignalSet};
use leptos_use::{use_cookie, utils::FromToStringCodec};
use once_cell::sync::Lazy;
use reqwest::Client;

use crate::BASE_API_URL;

pub static REQWEST_CLIENT: Lazy<Client> = Lazy::new(|| Client::builder().build().unwrap());

pub fn reset_token_if_needed(error_string: &str) {
    if error_string.contains("401 Unauthorized") {
        let (_, set_api_token) = use_cookie::<String, FromToStringCodec>("api_token");
        set_api_token.set(None);
    }
}

pub async fn get_all_tracks(api_token: Signal<Option<String>>) -> Result<Vec<Track>, String> {
    match get_all_tracks_inner(api_token).await {
        Ok(tracks) => Ok(tracks),
        Err(e) => {
            let cause = e.to_string();
            reset_token_if_needed(&cause);
            Err(cause)
        }
    }
}

async fn get_all_tracks_inner(api_token: Signal<Option<String>>) -> anyhow::Result<Vec<Track>> {
    let response = REQWEST_CLIENT
        .get(format!("{}get_all_tracks", *BASE_API_URL))
        .header(
            "api_token",
            api_token.get_untracked().context("No api_token set")?,
        )
        .send()
        .await?;

    response.error_for_status_ref()?;

    let bytes = response.bytes().await?;

    let mut tracks: Vec<Track> = serde_json::from_slice(&bytes)?;
    tracks.sort_unstable_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));

    Ok(tracks)
}

pub async fn archive_track(
    api_token: Signal<Option<String>>,
    candidate: Candidate,
) -> Result<(), String> {
    match archive_track_inner(api_token, candidate).await {
        Ok(_) => Ok(()),
        Err(e) => {
            let cause = e.to_string();
            reset_token_if_needed(&cause);
            Err(cause)
        }
    }
}

async fn archive_track_inner(
    api_token: Signal<Option<String>>,
    candidate: Candidate,
) -> anyhow::Result<()> {
    let response = REQWEST_CLIENT
        .post(format!("{}archive_track", *BASE_API_URL))
        .body(serde_json::to_string(&candidate)?)
        .header(
            "api_token",
            api_token.get_untracked().context("No api_token set")?,
        )
        .send()
        .await?;

    response.error_for_status_ref()?;
    Ok(())
}
