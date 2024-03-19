use std::{
    iter::once,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
};

use anyhow::Context;
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use audiotags::Tag;
use chrono::Utc;
use common::{candidate::Candidate, track::Track};
use crossbeam::channel::Receiver;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::{database::Database, DOWNLOAD_DIR, TRACK_DIR};

pub fn archiver_task(receiver: Receiver<Candidate>, db: Arc<Mutex<Database>>) {
    let mut gpt_client = match std::env::var("OPENAI_API_KEY") {
        Ok(_) => Some(Client::new()),
        Err(_) => {
            info!("OPENAI_API_KEY is not set: Not using OpenAI capabilities");
            None
        }
    };

    loop {
        let mut candidate = receiver.recv().unwrap();
        debug!("New candidate: {:?}", candidate);
        if db.lock().unwrap().is_track_archived(&candidate.url) {
            debug!("Track is already archived, skipping it.");
            continue;
        }
        if candidate.normalize_url().is_err() {
            info!("Candidate with invalid url received, skipping it.");
            continue;
        }

        debug!("Cleaning DOWNLOAD_DIR");
        std::fs::remove_dir_all(DOWNLOAD_DIR.clone()).unwrap();
        std::fs::create_dir(DOWNLOAD_DIR.clone()).unwrap();

        let track_id = db.lock().unwrap().next_track_id();

        debug!("Filling metadata");
        pollster::block_on(fill_metadata(&mut candidate, &mut gpt_client));

        debug!("Downloading track");
        if let Err(reason) = download_track(track_id, &candidate) {
            error!(
                "Unable to download track {:?} because of: {reason}",
                candidate
            );
            continue;
        }

        debug!("Setting audio tags");
        set_audio_tags(&candidate, track_id);

        let track = Track::new(
            track_id,
            candidate.url,
            candidate.title.unwrap(),
            candidate.artists,
            Utc::now().date_naive(),
        );
        debug!("Moving track from download_dir to tracks");
        let mut old_path = DOWNLOAD_DIR.clone();
        old_path.push(format!("{}.m4a", track_id));
        let mut new_path = TRACK_DIR.clone();
        new_path.push(track.file_name());
        std::fs::rename(old_path, new_path).unwrap();

        debug!("Inserting track into database");
        {
            let mut db = db.lock().unwrap();
            db.insert_tracks(once(&track));
        }
    }
}

// ./yt-dlp --print "%(track)s<<harmony>>%(artist)s<<harmony>>%(title)s<<harmony>>%(uploader)s"
async fn fill_metadata(candidate: &mut Candidate, ai: &mut Option<Client<OpenAIConfig>>) {
    // Get raw metadata from yt-dlp
    let mut raw = get_raw_metadata(candidate).unwrap();

    // TODO: Add chatgpt processing
    if let RawMetadata::Video {
        ref title,
        ref uploader,
    } = raw
    {
        if let Some(ai) = ai {
            match process_metadata_using_chatgpt(&title, ai).await {
                Ok(response) => {
                    let artists = if response.artists.is_empty() {
                        vec![uploader.clone()]
                    } else {
                        response.artists
                    };
                    raw = RawMetadata::Track {
                        title: response.title,
                        artists,
                    }
                }
                Err(e) => warn!("ChatGPT unable to fill in metadata: {e}"),
            }
        }
    }

    let (title, artists) = match raw {
        RawMetadata::Track { title, artists } => (title, artists),
        RawMetadata::Video { title, uploader } => (title, vec![uploader]),
    };
    candidate.title = Some(title);
    candidate.artists = artists;
}

enum RawMetadata {
    Track { title: String, artists: Vec<String> },
    Video { title: String, uploader: String },
}

fn get_raw_metadata(candidate: &Candidate) -> Result<RawMetadata, ()> {
    let mut cmd = Command::new("yt-dlp");
    cmd.args([
        "--print",
        "%(track)s<<harmony>>%(artist)s<<harmony>>%(title)s<<harmony>>%(uploader)s",
    ]);
    cmd.arg(candidate.url.clone());
    cmd.stdin(Stdio::null());
    cmd.stderr(Stdio::null());
    let output = cmd.output().unwrap();
    if output.status.success() {
        let data = String::from_utf8_lossy(&output.stdout).to_string();
        let mut splits = data.split("<<harmony>>");

        let track_title = splits.next().unwrap().trim().to_owned();
        let track_artists = splits
            .next()
            .unwrap()
            .trim()
            .split(", ")
            .filter(|s| *s != "NA" && !s.is_empty())
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();
        let video_title = splits.next().unwrap().trim().to_owned();
        let video_uploader = splits.next().unwrap().trim().to_owned();

        if !track_title.is_empty() && track_title != "NA" {
            let mut title = track_title;
            if title.is_empty() {
                title = "PLACEHOLDER".to_string();
            }
            let artists;
            if !track_artists.is_empty() {
                artists = track_artists;
            } else {
                let mut artist = video_uploader;
                if artist.is_empty() {
                    artist = "PLACEHOLDER".to_string();
                }
                artists = vec![artist];
            }
            return Ok(RawMetadata::Track { title, artists });
        }

        let mut title = video_title;
        if title.is_empty() {
            title = "PLACEHOLDER".to_string();
        }
        let mut uploader = video_uploader;
        if uploader.is_empty() {
            uploader = "PLACEHOLDER".to_string();
        }
        return Ok(RawMetadata::Video { title, uploader });
    }
    Err(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GptResponse {
    title: String,
    artists: Vec<String>,
}

async fn process_metadata_using_chatgpt(
    video_title: &str,
    ai: &mut Client<OpenAIConfig>,
) -> anyhow::Result<GptResponse> {
    debug!("Asking ChatGPT to extract info from video title");
    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(256u16)
        .model("gpt-3.5-turbo-0125")
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content(r#"Given is the titel of a music video. The video title contains the song title and may contain song artists. Extract the song title and artists and present them like this:
                {
                    "title": "TITLE",
                    "artists": ["ARTIST1", "ARTIST2"]
                }"#).build().unwrap().into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(video_title)
                .build().unwrap().into()
        ]).build().unwrap();

    let response = ai.chat().create(request).await?;
    let response = response
        .choices
        .get(0)
        .context("No response choice from the ai")?;

    let response: GptResponse = serde_json::from_str(
        &response
            .message
            .content
            .clone()
            .context("No text from the ai")?,
    )?;

    Ok(response)
}

fn download_track(id: u32, candidate: &Candidate) -> Result<(), String> {
    let mut cmd = Command::new("yt-dlp");
    cmd.args(["-o", &format!("{}.m4a", id.to_string())]);
    cmd.args([
        "--no-warnings",
        "-f",
        "bestaudio[ext=m4a]",
        "--add-metadata",
        "--embed-metadata",
        "--xattrs",
        &candidate.url,
    ]);
    cmd.current_dir(DOWNLOAD_DIR.clone());
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    let output = cmd.output().unwrap();
    if output.status.success() {
        Ok(())
    } else {
        let reason = format!(
            "yt-dlp was unable to download track: {} because: {}",
            candidate.url,
            String::from_utf8_lossy(&output.stderr)
        );
        Err(reason)
    }
}

fn set_audio_tags(candidate: &Candidate, id: u32) {
    let mut path = DOWNLOAD_DIR.clone();
    path.push(format!("{}.m4a", id));

    let mut tag = Tag::new().read_from_path(&path).unwrap();
    tag.set_title(&candidate.title.as_ref().unwrap());
    tag.set_artist(&candidate.artists.join(", "));
    tag.write_to_path(path.to_str().unwrap()).unwrap();
}
