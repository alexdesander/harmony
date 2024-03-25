use std::{
    env::set_var,
    fs,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex},
};

use archiver::archiver_task;
use auth::{auth_middleware, use_secret, TokenManager, TokenQuery};
use axum::{
    extract::Query,
    middleware,
    routing::{get, post},
    Router,
};
use database::Database;
use once_cell::sync::Lazy;
use requests::{archive_track, download_tracks, get_all_tracks};
use tower_http::cors::{Any, CorsLayer};
use tracing::{debug, warn, Level};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

pub mod archiver;
pub mod auth;
pub mod database;
pub mod requests;

pub static ARCHIVE_DIR: Lazy<PathBuf> = Lazy::new(get_archive_dir);
pub static DOWNLOAD_DIR: Lazy<PathBuf> = Lazy::new(get_download_dir);
pub static TRACK_DIR: Lazy<PathBuf> = Lazy::new(get_track_dir);

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    setup_tracing();

    let database = Arc::new(Mutex::new(Database::new(ARCHIVE_DIR.clone())));
    let token_manager = Arc::new(TokenManager::new(&std::env::var("SECRET").unwrap()));
    let (sender, receiver) = crossbeam::channel::unbounded();

    let _database = database.clone();
    tokio::task::spawn_blocking(move || archiver_task(receiver, _database));

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);

    let _token_manager = token_manager.clone();
    let app = Router::new()
        .route(
            "/get_all_tracks",
            get({
                let db = database.clone();
                move || get_all_tracks(db)
            }),
        )
        .route(
            "/download_tracks",
            post({
                let db = database.clone();
                move |_query: Query<TokenQuery>, body| download_tracks(db, body)
            }),
        )
        .route(
            "/archive_track",
            post({
                let sender = sender.clone();
                move |body| archive_track(sender, body)
            }),
        )
        .layer(middleware::from_fn(move |jar, query, request, next| {
            auth_middleware(jar, query, _token_manager.clone(), request, next)
        }))
        .route(
            "/use_secret",
            post({
                let token_manager = token_manager.clone();
                move |body| use_secret(token_manager, body)
            }),
        )
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
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
