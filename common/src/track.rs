use chrono::NaiveDate;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Getters)]
pub struct Track {
    pub id: u32,
    pub url: String,
    pub title: String,
    pub artists: Vec<String>,
    pub date_archived: NaiveDate,
    pub file_name: String,
}

impl Track {
    pub fn new(
        id: u32,
        url: String,
        title: String,
        artists: Vec<String>,
        date_archived: NaiveDate,
    ) -> Self {
        let mut track = Self {
            id,
            url,
            title,
            artists,
            date_archived,
            file_name: "".to_string(),
        };
        track.file_name = track.calculate_file_name();
        track
    }

    fn calculate_file_name(&self) -> String {
        let mut result = String::new();
        // Add comma separated artists
        let artists = self.artists.iter().map(|a| a.as_str()).intersperse(&", ");
        for artist in artists {
            result.push_str(&artist);
        }

        result.push_str(" - ");

        // Add title
        result.push_str(&self.title);

        // Add id
        result.push_str(format!(".{}", self.id()).as_str());

        // Add file extension
        result.push_str(".m4a");

        // Filenameify
        filenamify::filenamify(result)
    }
}
