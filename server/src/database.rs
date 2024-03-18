use std::path::PathBuf;

use common::track::Track;
use rusqlite::{Connection, OpenFlags};

pub struct Database {
    con: Connection,
}

impl Database {
    pub fn new(mut archive_dir: PathBuf) -> Self {
        // Create/open file
        archive_dir.push("harmony.db3");
        let con = Connection::open_with_flags(
            &archive_dir,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .expect(&format!(
            "Expected to be able to open/create {:?}",
            archive_dir
        ));

        // Create tables
        con.execute("
            BEGIN;
            CREATE TABLE IF NOT EXISTS tracks(
                id INTEGER NOT NULL PRIMARY KEY,
                url TEXT NOT NULL PRIMARY KEY,
                title TEXT NOT NULL,
                date_added TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS track_artists(
                track_id INTEGER NOT NULL,
                artist_id INTEGER NOT NULL,
                PRIMARY KEY (track_id, artist_id)
            );
            CREATE TABLE IF NOT EXISTS artists(
                id INTEGER NOT NULL PRIMARY KEY,
                name TEXT NOT NULL
            );
            COMMIT;
        ", []).expect("Expected SQL query to work");

        Self { con }
    }

    pub fn insert_tracks<'a>(&mut self, tracks: impl Iterator<Item = &'a Track>) {
        
    }
}
