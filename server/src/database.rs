use std::path::PathBuf;

use chrono::NaiveDate;
use common::track::Track;
use rusqlite::{Connection, OpenFlags, OptionalExtension};

pub struct Database {
    con: Connection,
    next_track_id: u32,
    next_artist_id: u32,
}

impl Database {
    pub fn new(mut archive_dir: PathBuf) -> Self {
        // Create/open file
        archive_dir.push("harmony.db3");
        let mut con = Connection::open_with_flags(
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
        let tx = con.transaction().unwrap();
        tx.execute(
            "CREATE TABLE IF NOT EXISTS tracks(
            id INTEGER NOT NULL PRIMARY KEY,
            url TEXT NOT NULL,
            title TEXT NOT NULL,
            date_archived TEXT NOT NULL);",
            [],
        )
        .unwrap();

        tx.execute(
            "CREATE TABLE IF NOT EXISTS track_artists(
            track_id INTEGER NOT NULL,
            artist_id INTEGER NOT NULL,
            PRIMARY KEY (track_id, artist_id));",
            [],
        )
        .unwrap();

        tx.execute(
            "CREATE TABLE IF NOT EXISTS artists(
            id INTEGER NOT NULL PRIMARY KEY,
            name TEXT NOT NULL);",
            [],
        )
        .unwrap();
        tx.commit().unwrap();

        // Get next ids
        let next_track_id = con
            .query_row("select id from tracks ORDER BY id DESC LIMIT 1;", [], |v| {
                let result: u32 = v.get(0).unwrap();
                Ok(result)
            })
            .optional()
            .expect("Expected query to work")
            .unwrap_or_default()
            + 1;

        let next_artist_id = con
            .query_row(
                "select id from artists ORDER BY id DESC LIMIT 1;",
                [],
                |v| {
                    let result: u32 = v.get(0).unwrap();
                    Ok(result)
                },
            )
            .optional()
            .expect("Expected query to work")
            .unwrap_or_default()
            + 1;

        Self {
            con,
            next_track_id,
            next_artist_id,
        }
    }

    pub fn next_track_id(&mut self) -> u32 {
        self.next_track_id += 1;
        self.next_track_id - 1
    }

    pub fn next_artist_id(&mut self) -> u32 {
        self.next_artist_id += 1;
        self.next_artist_id - 1
    }

    // Insert or replace tracks
    pub fn insert_tracks<'a>(&mut self, tracks: impl Iterator<Item = &'a Track> + Clone) {
        // Insert tracks
        let tx = self.con.transaction().unwrap();
        {
            let tracks = tracks.clone();
            let mut track_stmt = tx
                .prepare(
                    "REPLACE INTO tracks (id, url, title, date_archived) VALUES (?1, ?2, ?3, ?4)",
                )
                .unwrap();
            for track in tracks {
                let values = (
                    track.id(),
                    track.url(),
                    track.title(),
                    track.date_archived(),
                );
                track_stmt.execute(values).unwrap();
            }
        }
        tx.commit().unwrap();

        // Insert artists and track_artists entries
        {
            for track in tracks {
                for artist in track.artists() {
                    let id = self.insert_artist(artist);
                    self.con
                        .execute(
                            "INSERT OR IGNORE INTO track_artists (track_id, artist_id) VALUES (?1, ?2)",
                            (*track.id(), id),
                        )
                        .unwrap();
                }
            }
        }
    }

    pub fn remove_tracks<'a>(&mut self, ids: impl Iterator<Item = u32> + Clone) {
        let tx = self.con.transaction().unwrap();
        {
            // Drop tracks
            let mut stmt = tx.prepare("DELETE FROM tracks WHERE id = (?1)").unwrap();
            for id in ids.clone() {
                stmt.execute([id]).unwrap();
            }

            // Drop track_artists
            let mut stmt = tx
                .prepare("DELETE FROM track_artists WHERE track_id = (?1)")
                .unwrap();
            for id in ids {
                stmt.execute([id]).unwrap();
            }
        }
        tx.commit().unwrap();
    }

    pub fn is_track_archived(&self, url: &str) -> bool {
        self.con
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM tracks WHERE url = ?1)",
                [url],
                |v| v.get(0),
            )
            .unwrap()
    }

    pub fn artist_id(&mut self, artist: &str) -> Option<u32> {
        self.con
            .query_row(
                "SELECT id FROM artists WHERE name = ?1 COLLATE NOCASE",
                [artist],
                |r| r.get(0),
            )
            .optional()
            .unwrap()
    }

    // Inserts artist and returns id of artist
    pub fn insert_artist(&mut self, artist: &str) -> u32 {
        match self.artist_id(artist) {
            Some(id) => id,
            None => {
                let id = self.next_artist_id();
                self.con
                    .execute(
                        "INSERT INTO artists (id, name) VALUES (?1, ?2)",
                        (id, artist),
                    )
                    .unwrap();
                id
            }
        }
    }

    pub fn all_tracks(&mut self) -> Vec<Track> {
        // Get tracks WITHOUT artists
        let mut sql = self
            .con
            .prepare("SELECT id, url, title, date_archived FROM tracks")
            .unwrap();

        let mut tracks = sql
            .query_map([], |row| {
                let id: u32 = row.get(0).unwrap();
                let url: String = row.get(1).unwrap();
                let title: String = row.get(2).unwrap();
                let date_archived: NaiveDate = row.get(3).unwrap();

                Ok(Track {
                    id,
                    url,
                    title,
                    artists: vec![],
                    date_archived,
                })
            })
            .unwrap()
            .map(|track| track.expect("Expected all tracks read from database to be valid."))
            .collect::<Vec<_>>();

        // Get artists
        let mut sql = self
            .con
            .prepare(
                "
            SELECT
                artists.name AS artist_name
            FROM
                artists
            JOIN
                track_artists ON artists.id = track_artists.artist_id
            WHERE
                track_artists.track_id = ?1;
        ",
            )
            .unwrap();

        for track in &mut tracks {
            let artists = sql
                .query_map([track.id], |row| {
                    let name: String = row.get_unwrap(0);
                    Ok(name)
                })
                .unwrap()
                .map(|a| a.unwrap());
            track.artists.extend(artists)
        }
        tracks
    }
}
