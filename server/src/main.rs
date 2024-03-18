use std::{fs, path::PathBuf, str::FromStr};

use database::Database;
use tracing::{debug, level_filters::LevelFilter, warn};

pub mod database;

fn main() {
    setup_tracing();
    let archive_dir = get_archive_dir();

    let database = Database::new(archive_dir);
}

fn setup_tracing() {
    let subscriber = tracing_subscriber::fmt()
        .with_file(true)
        .with_line_number(true)
        .with_max_level(if cfg!(debug_assertions) {
            LevelFilter::DEBUG
        } else {
            LevelFilter::INFO
        })
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
}

fn get_archive_dir() -> PathBuf {
    let raw = match std::env::var("HARMONY_ARCHIVE_DIR") {
        Ok(raw) => raw,
        Err(e) => {
            warn!("Unable to get HARMONY_ARCHIVE_DIR due to: '{e}'. Falling back to './harchive'");
            "./harchive".to_owned()
        },
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
