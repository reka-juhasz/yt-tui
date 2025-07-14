use anyhow::anyhow;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use std::fs::File;
use std::io::Write;

//file writing imports stay to help w debugging
use std::process::Command;
#[derive(Debug, Deserialize)]
pub struct PlaylistListResponse {
    pub items: Vec<PlaylistItem>,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistItem {
    pub id: String,
    pub snippet: Snippet,
}

#[derive(Debug, Deserialize)]
pub struct Snippet {
    pub title: String,
    pub resourceId: Option<ResourceId>,
}

#[derive(Debug, Deserialize)]
pub struct ResourceId {
    pub videoId: String,
}
/// direct audio stream from yt, using yt-dlp
pub fn get_audio_url(video_url: &str) -> Result<String> {
    let output = Command::new("yt-dlp")
        .args(["-g", "-f", "bestaudio", video_url])
        .output()
        .map_err(|e| anyhow!("Failed to run yt-dlp: {}", e))?;

    if !output.status.success() {
        return Err(anyhow!(
            "yt-dlp failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let last_line = stdout
        .lines()
        .last()
        .ok_or_else(|| anyhow!("No URL returned by yt-dlp"))?;

    Ok(last_line.to_string())
}

/// plays a single song using mpv
pub fn play_song(link: &str) {
    match get_audio_url(link) {
        Ok(audio_url) => {
            let status = Command::new("mpv")
                .args([
                    "--no-video",
                    "--really-quiet",
                    "--no-config",
                    "--idle=no",
                    &audio_url,
                ])
                .status();

            match status {
                Ok(status) if status.success() => {}
                Ok(status) => eprintln!("❌ mpv exited with status: {:?}", status.code()),
                Err(e) => eprintln!("❌ Failed to start mpv: {}", e),
            }
        }
        Err(e) => {
            eprintln!("⚠️ Failed to get audio stream: {}", e);
        }
    }
}

pub async fn list_playlists(access_token: &str) -> Result<Vec<(String, String)>> {
    let url =
        "https://www.googleapis.com/youtube/v3/playlists?part=snippet&mine=true&maxResults=50";

    let client = reqwest::Client::new(); // new client
    let response = client
        .get(url) //url that we request
        .bearer_auth(access_token) //oauth token with the bearer schema
        .send() //sending the request
        .await //waiting for response
        .context("Failed to send playlist request")?; //context if something goes wrong

    let status = response.status(); //status code for request reponse
    let text = response
        .text()
        .await
        .context("Failed to read response text")?;
    // always write raw response, even if it's an error

    if !status.is_success() {
        return Err(anyhow!("YouTube API error ({}): {}", status.as_u16(), text));
    }

    let playlists_response: PlaylistListResponse =
        serde_json::from_str(&text).context("Failed to parse playlists JSON")?;

    let playlists = playlists_response
        .items
        .into_iter()
        .map(|item| (item.snippet.title, item.id))
        .collect();

    Ok(playlists)
}

pub async fn get_videos_from_playlist(
    access_token: &str,
    playlist_id: &str,
) -> Result<Vec<(String, String)>> {
    let url = format!(
        "https://www.googleapis.com/youtube/v3/playlistItems?part=snippet&playlistId={}&maxResults=50",
        playlist_id
    );

    let client = Client::new();
    let response = client
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .context("Failed to send playlistItems request")?;

    let status = response.status();
    let text = response
        .text()
        .await
        .context("Failed to read response text")?;

    if !status.is_success() {
        return Err(anyhow::anyhow!(
            "YouTube API error ({}): {}",
            status.as_u16(),
            text
        ));
    }

    let playlist_items: PlaylistListResponse =
        serde_json::from_str(&text).context("Failed to parse playlistItems JSON")?;

    let video_urls = playlist_items
        .items
        .into_iter()
        .filter_map(|item| {
            if let Some(resource) = item.snippet.resourceId {
                let url = format!("https://www.youtube.com/watch?v={}", resource.videoId);
                Some((item.snippet.title, url))
            } else {
                None
            }
        })
        .collect();

    Ok(video_urls)
}

pub async fn play_playlist(access_token: &str, playlist_id: &str) -> Result<()> {
    let videos = get_videos_from_playlist(access_token, playlist_id).await?;

    if videos.is_empty() {
        println!("📭 No videos found in the playlist.");
        return Ok(());
    }

    for (title, url) in videos {
        println!("▶️ Now playing: {}", title);
        play_song(&url);
    }

    Ok(())
}
