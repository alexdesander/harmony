use std::{
    env::set_var,
    fs,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex},
};

use archiver::archiver_task;
use common::candidate::Candidate;
use database::Database;
use once_cell::sync::Lazy;
use tracing::{debug, warn, Level};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

pub mod archiver;
pub mod database;

pub static ARCHIVE_DIR: Lazy<PathBuf> = Lazy::new(get_archive_dir);
pub static DOWNLOAD_DIR: Lazy<PathBuf> = Lazy::new(get_download_dir);
pub static TRACK_DIR: Lazy<PathBuf> = Lazy::new(get_track_dir);

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    setup_tracing();

    let database = Arc::new(Mutex::new(Database::new(ARCHIVE_DIR.clone())));
    let (sender, receiver) = crossbeam::channel::unbounded();

    tokio::task::spawn_blocking(move || archiver_task(receiver, database.clone()));

    sender
        .send(Candidate {
            url: "https://www.youtube.com/watch?v=dngleMsdEL0".to_string(),
            title: None,
            artists: vec![],
        })
        .unwrap();

    loop {}
}

fn setup_tracing() {
    set_var("RUST_LOG", "none,server=trace,common=trace");
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_max_level(Level::TRACE)
        .with_span_events(FmtSpan::ACTIVE)
        .with_line_number(true)
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
}

fn get_archive_dir() -> PathBuf {
    let raw = match std::env::var("HARMONY_ARCHIVE_DIR") {
        Ok(raw) => raw,
        Err(e) => {
            warn!("Unable to get HARMONY_ARCHIVE_DIR due to: '{e}'. Falling back to './harchive'");
            "./harchive".to_owned()
        }
    };
    debug!("Creating/validating archive directory: {}", raw);

    let path = PathBuf::from_str(&raw).expect("Expected HARMONY_ARCHIVE_DIR to be a valid path");

    // Create directory
    if path.exists() {
        if !path.is_dir() {
            panic!("{:?} should be a directory", &path);
        }
    } else {
        if let Err(e) = fs::create_dir_all(&path) {
            panic!(
                "Expected to be able to create all parent directories of {:?}, error: {e}",
                &path
            );
        }
    }

    path
}

fn get_download_dir() -> PathBuf {
    let mut archive_dir = ARCHIVE_DIR.clone();
    archive_dir.push("downloads");
    fs::create_dir_all(&archive_dir).unwrap();
    archive_dir
}

fn get_track_dir() -> PathBuf {
    let mut archive_dir = ARCHIVE_DIR.clone();
    archive_dir.push("tracks");
    fs::create_dir_all(&archive_dir).unwrap();
    archive_dir
}
