use chrono::{NaiveDate, Utc};
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Getters)]
pub struct Track {
    id: u32,
    url: String,
    title: String,
    artists: Vec<String>,
    date_archived: NaiveDate,
}

impl Track {
    pub fn get_file_name(&self) -> String {
        let mut result = String::new();
        // Add comma separated artists
        let artists = self.artists.iter().map(|a| a.as_str()).intersperse(&", ");
        for artist in artists {
            result.push_str(&artist);
        }

        result.push_str(" - ");

        // Add title
        result.push_str(&self.title);

        // Add file extension
        result.push_str(".m4a");

        // Filenameify
        filenamify::filenamify(result)
    }
}
