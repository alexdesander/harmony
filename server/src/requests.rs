use std::sync::{Arc, Mutex};

use axum::{
    body::{Body, Bytes},
    http::header,
    response::IntoResponse,
};
use common::candidate::Candidate;
use crossbeam::channel::Sender;
use tokio_util::io::ReaderStream;

use crate::{database::Database, TRACK_DIR};

pub async fn get_all_tracks(database: Arc<Mutex<Database>>) -> Vec<u8> {
    let tracks = database.lock().unwrap().all_tracks();
    bitcode::serialize(&tracks).unwrap()
}

pub async fn archive_track(sender: Sender<Candidate>, body: Bytes) -> Result<(), String> {
    let candidate: Candidate = match bitcode::decode(&body) {
        Ok(candidate) => candidate,
        Err(e) => return Err(e.to_string()),
    };
    sender.send(candidate).unwrap();
    Ok(())
}

pub async fn download_tracks(database: Arc<Mutex<Database>>, body: String) -> impl IntoResponse {
    let ids_encoded = body
        .trim()
        .trim_start_matches("ids=%5B")
        .trim_end_matches("%5D");
    let ids_encoded = format!("[{}]", ids_encoded);
    let ids: Vec<u32> = match serde_json::from_str(&ids_encoded) {
        Ok(ids) => ids,
        Err(e) => return Err(e.to_string()),
    };
    let tracks = match database.lock().unwrap().get_tracks(ids.into_iter()) {
        Ok(tracks) => tracks,
        Err(e) => return Err(e.to_string()),
    };

    // Handle case of 1 track
    let (file, filename) = match tracks.len() {
        1 => {
            let track = &tracks[0];
            let file_name = track.file_name();
            let mut path = TRACK_DIR.clone();
            path.push(&file_name);
            (
                match tokio::fs::File::open(path).await {
                    Ok(file) => file,
                    Err(err) => return Err(err.to_string()),
                },
                file_name,
            )
        }
        _ => todo!(),
    };

    // convert the `AsyncRead` into a `Stream`
    let stream = ReaderStream::new(file);
    // convert the `Stream` into an `axum::body::HttpBody`
    let body = Body::from_stream(stream);

    let headers = [(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", filename),
    )];

    Ok((headers, body))
}
