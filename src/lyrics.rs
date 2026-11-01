use std::ops::Deref;
use urlencoding;
use reqwest;
use tokio::time::{sleep, Duration};
use crate::apple_music_navigation::Track;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct LrcSearchResult {
    pub synced_lyrics: String,
}


#[derive(Deserialize, Debug)]
pub struct LyricLine{
    pub timestamp: u32,
    pub text: String
}

fn parse_time_to_ms(time_str: &str) -> Option<u32> {
    let mut split = time_str.split(':');
    let mins: u32 = split.next()?.parse().ok()?;
    let rest = split.next()?;

    let mut sec_split = rest.split('.');
    let secs: u32 = sec_split.next()?.parse().ok()?;
    let millis_part = sec_split.next().unwrap_or("0");

    let millis: u32 = match millis_part.len() {
        1 => millis_part.parse::<u32>().ok()? * 100,
        2 => millis_part.parse::<u32>().ok()? * 10,
        _ => millis_part.parse::<u32>().ok()?,
    };

    Some((mins * 60 + secs) * 1000 + millis)
}


pub async fn request_lyric(track: &Track) -> Result<Option<Vec<LyricLine>>, String> {
    
    let url = format!("https://lrclib.net/api/search?track_name={}&artist_name={}&album_name={}",
       urlencoding::encode(track.title.as_deref().unwrap()),
       urlencoding::encode(track.artist.as_deref().unwrap()),
       urlencoding::encode( track.album.as_deref().unwrap()));
    
    let mut res = reqwest::get(&url).await
        .map_err(|e| format!("Error sending lyrcs API request: {}",e))?;

    let mut retries = 3;
    let mut retry_delay = 200;
    while !res.status().is_success() && retries > 0 {
        res = reqwest::get(&url).await
        .map_err(|e| format!("Error sending lyrcs API request: {}",e))?;

        sleep(Duration::from_millis(retry_delay)).await;

        retries -= 1;
        retry_delay *= 2;
    }

    if !res.status().is_success() {
        return Ok(None)
    }

    let results_vec: Vec<LrcSearchResult> = res.json().await
        .map_err(|e| e.to_string())?;

    let best_match = match results_vec.first() {
        Some(m) => m,
        None => return Ok(None)
    };

    let synced_str = best_match.synced_lyrics.deref();

    let mut lyrics = Vec::new();
    for line in synced_str.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.contains(']') {
            let parts: Vec<&str> = line.splitn(2, ']').collect();
            if parts.len() == 2 {
                let time_str = &parts[0][1..];
                let text = parts[1].trim().to_string();

                if let Some(timestamp) = parse_time_to_ms(time_str) {
                    lyrics.push(LyricLine { timestamp, text });
                }
            }
        }
    }




    Ok(Some(lyrics))
}



//INDEX: 0 = PREVIOUS LYRIC INDEX:1 CURRENT LYRIC INDEX:2 NEXT LYRIC
pub fn get_current_lyrics<'a>(lyrics: &'a [LyricLine], time: Option<u32>) -> [&'a str; 3] {
    
    
    if lyrics.is_empty() || time.is_none() {
        return ["", "", ""];
    }

    let current_time = time.unwrap();

    let mut active_idx = 0;
    for (i, line) in lyrics.iter().enumerate() {
        if current_time >= line.timestamp {
            active_idx = i;
        } else {
            break;
        }
    }
    let prev = if active_idx > 0 {
        lyrics[active_idx - 1].text.as_str()
    } else {
        ""
    };

    let current = lyrics[active_idx].text.as_str();

    let next = if active_idx + 1 < lyrics.len() {
        lyrics[active_idx + 1].text.as_str()
    } else {
        ""
    };
    [prev, current, next]
}