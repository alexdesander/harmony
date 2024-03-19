use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    pub url: String,
    pub title: Option<String>,
    pub artists: Vec<String>,
}

impl Candidate {
    // Returns Err(()) if url is invalid
    pub fn normalize_url(&mut self) -> Result<(), ()> {
        self.url = match normalize_and_validate_url(&self.url) {
            Some(url) => url,
            None => return Err(()),
        };

        Ok(())
    }
}

fn normalize_and_validate_url(url: &str) -> Option<String> {
    let url = url
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.");

    // YOUTUBE LINKS
    // Check youtu.be links
    if url.starts_with("youtu.be/") {
        let id = url
            .trim_start_matches("youtu.be/")
            .split("?")
            .next()?
            .to_string();
        return Some(format!("youtu.be/{}", id));
    }

    // Check youtube.com links
    if url.starts_with("youtube.com/watch?v=") {
        let id = url
            .trim_start_matches("youtube.com/watch?v=")
            .split("&")
            .next()?
            .to_string();
        return Some(format!("youtu.be/{}", id));
    }

    // SOUNDCLOUD LINKS
    // Check soundcloud.com
    /*if url.starts_with("soundcloud.com/") {
        let mut splits = url.trim_start_matches("soundcloud.com/").split("/");
        let artist = splits.next()?.to_string();
        let id = splits.next()?.split("?").next()?.to_string();
        return Some(format!("soundcloud.com/{}/{}", artist, id));
    }*/

    // Check on.soundcloud.com
    // WE DO NOT SUPPORT THIS CURRENTLY

    None
}
