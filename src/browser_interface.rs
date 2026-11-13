use std::vec;
use crate::{commands::MediaType, daemon_controls::format_time};
use thirtyfour::prelude::*;
use serde::{Serialize, Deserialize};
use wincode_derive::{SchemaRead, SchemaWrite};

#[derive(Debug, Serialize, Deserialize, SchemaRead, SchemaWrite, PartialEq, Clone)]
pub struct Artwork {
    pub height: u32,
    pub width: u32,
    pub url: String
}


#[derive(Debug, Serialize, Deserialize, SchemaRead, SchemaWrite, PartialEq, Clone)]
pub struct Track {
    pub id: Option<String>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration_ms: Option<u32>,
    pub current_ms: Option<u32>,
    pub artwork: Option<Artwork>
}

impl Track {
    pub fn pretty_display(&self) -> String {
    let duration = if self.duration_ms.is_some() {
        format_time(self.duration_ms.unwrap())
    } else {
        "Unknown Duration".to_string()
    };
    


    format!(
            "{} - (id: {})\n{} | {} | ({})",
            self.title.clone().unwrap_or_else(|| "Unknown Title".to_string()),
            self.id.clone().unwrap_or_else(|| "Unknown ID".to_string()),
            self.artist.clone().unwrap_or_else(|| "Unknown Artist".to_string()),
            self.album.clone().unwrap_or_else(|| "Unknown Album".to_string()),
            duration

        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Playlist {
    pub id: Option<String>,
    pub name: Option<String>,
    pub desc: Option<String>
}

impl Playlist {
    pub fn pretty_display(&self) -> String {
        format!(
            "{} - (id: {})\n{}",
            self.name.clone().unwrap_or_else(|| "Unknown Name".to_string()),
            self.id.clone().unwrap_or_else(|| "Unknown ID".to_string()),
            self.desc.clone().unwrap_or_default()
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Album {
    pub id: Option<String>,
    pub name: Option<String>,
    pub artist: Option<String>,
    pub track_count: Option<u32>,
}

impl Album {
    pub fn pretty_display(&self) -> String {
        format!(
            "{}(id: {})\n{} | Tracks: {}",
            self.name.clone().unwrap_or_else(|| "Unknown Name".to_string()),
            self.id.clone().unwrap_or_else(|| "Unknown ID".to_string()),
            self.artist.clone().unwrap_or_else(|| "Unknown Artist".to_string()),
            self.track_count.clone().unwrap_or_else(|| 0)
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Artist {
    pub id: Option<String>,
    pub name: Option<String>,
}

impl Artist {
    pub fn pretty_display(&self) -> String {
        format!(
            "{}(id: {})",
            self.name.clone().unwrap_or_else(|| "Unknown Name".to_string()),
            self.id.clone().unwrap_or_else(|| "Unknown ID".to_string())
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SearchResult {
    Track(Track),
    Album(Album),
    Playlist(Playlist),
    Artist(Artist)
}

pub trait MusicDriver {
    async fn search(&self, query: &str, item_type: Vec<MediaType>, is_library: bool, limit: u32) -> Result<SearchResult, String>; 
    async fn queue(&self, item_type: &MediaType, id: &str, is_next: bool) -> Result<String, String>;
    async fn set_queue(&self, item_type: &MediaType, id: &str, is_library: bool) -> Result<String, String>;
    async fn play(&self) -> Result<(), String>; 
    async fn pause(&self) -> Result<(), String>; 
    async fn next_track(&self) -> Result<(), String>; 
    async fn previous_track(&self) -> Result<(), String>;
    async fn shuffle_toggle(&self) -> Result<i64, String>;
    async fn get_current_track(&self) -> Result<Option<Track>, String>; 
    async fn clear_queue(&self) -> Result<(), String>; 
    async fn remove_queue(&self, index: u32) -> Result<(), String>;
    async fn list_queue(&self) -> Result<Vec<Track>, String>;
    async fn volume(&self, val: Option<u32>) -> Result<i32, String>;
    async fn autoplay_toggle(&self) -> Result<bool, String>;
    async fn create_pl(&self, name: &str, desc: &str) -> Result<String, String>;
    async fn add_to_pl(&self, track_id: &str, pl_id: &str) -> Result<String, String>;
    async fn add_to_fav(&self, track_id: &str) -> Result<String, String>;
}

impl MusicDriver for WebDriver {
    //WORKING
async fn search(&self, term: &str, item_types: Vec<MediaType>, is_library: bool, limit: u32) -> Result<SearchResult, String> {
    let types_param = item_types
        .iter()
        .map(|t| {
            if is_library {
                format!("library-{}", t.as_str())
            } else {
                t.as_str().to_string()
            }
        })
        .collect::<Vec<String>>()
        .join(",");

    let script = format!(r#"
    return (async () => {{
        try {{
            const mk = MusicKit.getInstance();
            if (!mk || !mk.isAuthorized) {{
                await mk?.authorize();
            }}
            
            const storefront = mk.storefrontId || 'it';
            const isLib = {};
            const term = "{}";
            const typesParam = "{}";
            const limit = {};
            
            const endpoint = isLib 
                ? "v1/me/library/search" 
                : `v1/catalog/${{storefront}}/search`;
            
            const response = await mk.api.music(endpoint, {{
                term: term,
                types: typesParam,
                limit: 1
            }});
            
            const results = response?.data?.results;
            if (!results) return null;
            
            let rawItem = null;
            for (const key of Object.keys(results)) {{
                const items = results[key]?.data;
                if (items && items.length > 0) {{
                    rawItem = items[0];
                    break;
                }}
            }}
            
            if (!rawItem) return null;
            
            const typeStr = rawItem.type || "";
            const attrs = rawItem.attributes || {{}};
            
            // Map Apple Music JSON:API structure to match Rust's #[serde(tag = "type")] enum
            if (typeStr.includes("songs")) {{
                return {{
                    type: "Track",
                    id: rawItem.id,
                    title: attrs.name || null,
                    artist: attrs.artistName || null,
                    album: attrs.albumName || null,
                    duration_ms: attrs.durationInMillis || null,
                    current_ms: null,
                    artwork: attrs.artwork || null
                }};
            }} else if (typeStr.includes("albums")) {{
                return {{
                    type: "Album",
                    id: rawItem.id,
                    name: attrs.name || null,
                    artist: attrs.artistName || null,
                    track_count: attrs.trackCount || null
                }};
            }} else if (typeStr.includes("playlists")) {{
                const descVal = attrs.description;
                const desc = typeof descVal === 'object' ? descVal?.standard : (descVal || null);
                return {{
                    type: "Playlist",
                    id: rawItem.id,
                    name: attrs.name || null,
                    desc: desc
                }};
            }} else if (typeStr.includes("artists")) {{
                return {{
                    type: "Artist",
                    id: rawItem.id,
                    name: attrs.name || null
                }};
            }}
            
            return null;
        }} catch(e) {{
            return null;
        }}
    }})();
    "#, 
    if is_library { "true" } else { "false" },
    term,
    types_param,
    limit
    );

    let res = self.execute(script, vec![]).await
        .map_err(|e| e.to_string())?;

    let search_result: Option<SearchResult> = serde_json::from_value(res.json().clone())
        .map_err(|e| format!("Deserialization error: {}", e))?;

    search_result.ok_or_else(|| "No Matches Found".to_string())
}
    //WORKING
    async fn queue(&self, item_type: &MediaType, id: &str, is_next: bool) -> Result<String, String> {
    
    let method = if is_next {
        "playNext"
    } else {
        "playLater"
    };

    if item_type == &MediaType::Artist {
        return Err("Cant Queue an artist".to_string());
    }
    
    let script = format!(r#"
    return (async () => {{
        const mk = MusicKit.getInstance();
        if (!mk || !mk.isAuthorized ) {{
        await mk?.authorize();
    }}
    try {{ 
    await mk.{}({{ {}: '{}'}});
    return true;
    }}
    catch (e) {{
        return false;
    }}
    }})()
    "#,method, item_type.as_str(), id);

    let res = self.execute(script, vec![]).await
        .map_err(|e| e.to_string())?;


    let is_success: bool = serde_json::from_value(res.json().clone())
        .map_err(|e| e.to_string())?;

    if is_success {
        return Ok(id.to_string())
    }

    Err("Could Not play next, maybe wrong id".to_string())
        
    }

    //WORKING
async fn set_queue(&self, item_type: &MediaType, id: &str, is_library: bool) -> Result<String, String> {
    let key = if is_library {
        format!("library-{}", item_type.as_str())
    } else {
        match item_type {
            MediaType::Song => "song".to_string(),
            MediaType::Album => "album".to_string(),
            MediaType::Playlist => "playlist".to_string(),
            MediaType::Artist => "artist".to_string(),
        }
    };

    let script = format!(r#"
    return (async () => {{
        try {{
            const mk = MusicKit.getInstance();
            if (!mk || !mk.isAuthorized) {{
                await mk?.authorize();
            }}

            const key = "{}";
            const id = "{}";

            if (key === "artist" || key === "library-artists") {{
    let catalogId = id;
    if (key === "library-artists") {{
        const libRes = await mk.api.music(`v1/me/library/artists/${{id}}`, {{ include: 'catalog' }});
        catalogId = libRes?.data?.data?.[0]?.relationships?.catalog?.data?.[0]?.id;
        if (!catalogId) {{
            return {{ success: false, error: "No catalog id for this library artist" }};
        }}
    }}

    // Artist-only station: note the "a-" prefix
    const stationId = `ra.a-${{catalogId}}`;
    await mk.setQueue({{ station: stationId, startPlaying: true }});
    return {{ success: true, station: stationId }};
}}

            await mk.setQueue({{
                [key]: id
            }});

            return {{ success: true }};
        }} catch (e) {{
            return {{ success: false, error: e.toString() }};
        }}
    }})();
    "#, key, id);

    let res = self.execute(script, vec![]).await
        .map_err(|e| e.to_string())?;

    let res_val: serde_json::Value = res.json().clone();
    let is_success = res_val.get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if is_success {
        return Ok(format!("Queue Set ID: {}", id));
    }

    let err_msg = res_val.get("error")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown JavaScript error");

    Err(format!("SetQueue failed: {}", err_msg))
}
    //WORKING
    async fn play(&self) -> Result<(), String> {
    let script = "MusicKit.getInstance().play();";
    self.execute(script, vec![]).await.map_err(|e| e.to_string())?;
    Ok(())
    }   
    
    //WORKING
    async fn pause(&self) -> Result<(), String> {
    let script = "MusicKit.getInstance().pause();";
    self.execute(script, vec![]).await.map_err(|e| e.to_string())?;
    Ok(())
}
    
    //WORKING
    async fn next_track(&self) -> Result<(), String> {
    let script = "MusicKit.getInstance().skipToNextItem();";
    self.execute(script, vec![]).await.map_err(|e| e.to_string())?;
    Ok(())
}
    
    //WORKING
    async fn previous_track(&self) -> Result<(), String> {
    let script = "MusicKit.getInstance().skipToPreviousItem();";
    self.execute(script, vec![]).await.map_err(|e| e.to_string())?;
    Ok(())
}
    
    //WORKING
    async fn shuffle_toggle(&self) -> Result<i64, String> {
    let script = r#"
        return window.MusicKit.getInstance().shuffleMode;
    "#;

    let value = self.execute(script, vec![]).await
        .map_err(|e| format!("Failed to execute script: {}", e))?;

    // Convert the serde_json::Value to a Rust boolean
    let is_shuffled: i64 = value.json().as_i64().unwrap_or(0);

    let num_mode = if is_shuffled == 1 {
        0
    } else {
        1
    };

    let script = format!{"MusicKit.getInstance().shuffleMode = {};", num_mode};
    self.execute(script, vec![]).await.map_err(|e| e.to_string())?;
    Ok(num_mode)
}

    //WORKING
    async fn get_current_track(&self) -> Result<Option<Track>, String> {
    let script = r#"
return (() => {
    let mediaItem = MusicKit.getInstance().nowPlayingItem;
    
    if (mediaItem) {
        return {
            id: mediaItem.id,
            title: mediaItem.attributes.name,
            artist: mediaItem.attributes.artistName,
            album: mediaItem.attributes.albumName,
            duration_ms: mediaItem.attributes.durationInMillis,
            current_ms: mediaItem.attributes.durationInMillis - ((MusicKit.getInstance().currentPlaybackTimeRemaining) * 1000),
            artwork: mediaItem.attributes.artwork
        };
    }

    return null;
})();
    "#;

    let res = self.execute(script, vec![])
        .await
        .map_err(|e| e.to_string())?;

    let track_obj: Option<Track> = serde_json::from_value(res.json().clone())
        .map_err(|e| e.to_string())?;

    if track_obj.is_none() {
        return Ok(None)
    }
    Ok(track_obj)
}
    
    //WORKING
    async fn clear_queue(&self) -> Result<(), String> {
    let script = r#"
    MusicKit.getInstance().clearQueue()
    "#;

    self.execute(script, vec![]).await.map_err(|e| e.to_string())?;
    
    Ok(())
}
    
    //WORKING
    async fn remove_queue(&self, index: u32) -> Result<(), String> {
    let script = format!(r#"
    MusicKit.getInstance().queue.remove({})
    "#, index);
    self.execute(script, vec![]).await.map_err(|e| e.to_string())?;
    
    Ok(())
}
    
    //WORKING
    async fn list_queue(&self) -> Result<Vec<Track>, String> {
    let script = r#"
        return (async () => {
        let res = MusicKit.getInstance().queue._queueItems || [];
        let queue_position = MusicKit.getInstance().queue._position || 0;
        let queue = res.slice(queue_position);
        return queue.map(e => ({
            id: e.item.id,
            title: e.item.attributes.name,
            artist: e.item.attributes.artistName,
            album: e.item.attributes.albumName,
            duration_ms: e.item.attributes.durationInMillis,
            artwork: e.item.attributes.artwork
        }));
    })();
    "#;

    let res = self.execute(script, vec![]).await
        .map_err(|e| e.to_string())?;

    let parsed_res: Vec<Track> = serde_json::from_value(res.json().clone())
        .map_err(|e| e.to_string())?;



    Ok(parsed_res)
    } 
    
    //WORKING
    async fn volume(&self, val: Option<u32>) -> Result<i32, String> {
    let script =
        if val.is_none() {
                r#"
                return (async () => {
                    return MusicKit.getInstance().volume;
        })();
                "#.to_string()
        } else {
            format!(
                r#"
                return (async () => {{
                    MusicKit.getInstance().volume = {};
                    return MusicKit.getInstance().volume;
            }})();
                "#, val.unwrap() as f64 / 100.0
            )
        };

        let res = self.execute(script, vec![]).await
            .map_err(|e| e.to_string())?;

        let parsed_res: f64 = serde_json::from_value(res.json().clone())
            .map_err(|e| e.to_string())?;
    Ok((parsed_res * 100.0).round() as i32)
}
    //WORKING
    async fn autoplay_toggle(&self) -> Result<bool, String> {
        let script = r#"
            return (() => {
                const mk = MusicKit.getInstance();
                if (typeof mk.autoplayEnabled !== 'boolean') {
                    return { success: false, error: 'autoplayEnabled is ' + typeof mk.autoplayEnabled };
                }
                mk.autoplayEnabled = !mk.autoplayEnabled;
                return { success: true, enabled: mk.autoplayEnabled };
            })();
        "#;

        let res = self.execute(script, vec![]).await.map_err(|e| e.to_string())?;
        let v: serde_json::Value = res.json().clone();

        if v.get("success").and_then(|s| s.as_bool()) != Some(true) {
            return Err(format!("autoplay toggle failed: {}", v));
        }
        Ok(v.get("enabled").and_then(|e| e.as_bool()).unwrap_or(false))
    }
    //WORKING
    async fn create_pl(&self, name: &str, desc: &str) -> Result<String, String> {
let name_js = serde_json::to_string(name).map_err(|e| e.to_string())?;
let desc_js = serde_json::to_string(desc).map_err(|e| e.to_string())?;

let script = format!(r#"
    return (async () => {{
        try {{
            const mk = MusicKit.getInstance();
            const r = await fetch('https://amp-api.music.apple.com/v1/me/library/playlists', {{
                method: 'POST',
                headers: {{
                    Authorization: `Bearer ${{mk.developerToken}}`,
                    'Music-User-Token': mk.musicUserToken,
                    'Content-Type': 'application/json',
                }},
                body: JSON.stringify({{
                    attributes: {{
                        name: {},
                        description: {}
                    }}
                }}),
            }});
            const json = await r.json();
            const id = json?.data?.[0]?.id;

            if (r.status !== 201 || !id) {{
                return {{ success: false, error: 'create failed', status: r.status, body: json }};
            }}
            return {{ success: true, id }};
        }} catch (e) {{
            return {{ success: false, error: String(e) }};
        }}
    }})();
"#, name_js, desc_js);

    let res = self
        .execute(script, vec![])
        .await
        .map_err(|e| e.to_string())?;
    let v: serde_json::Value = res.json().clone();

    if v.get("success").and_then(|s| s.as_bool()) == Some(true) {
        return Ok(v["id"].as_str().unwrap_or_default().to_string());
    }
    Err(format!("create_playlist failed: {}", v))
}
    //WORKING
    async fn add_to_pl(&self, track_id: &str, pl_id: &str) -> Result<String, String> {
    let script = format!(
        r#"
        return (async () => {{
            const mk = MusicKit.getInstance();
            const devToken = mk.developerToken;
            const userToken = mk.musicUserToken;
            
            const trackId = "{}";
            const playlistId = "{}";

            if (!devToken || !userToken) {{
                throw new Error("MusicKit tokens not found. Ensure you are authorized.");
            }}

            // 1. Add the song to the user's library using native fetch
            const libRes = await fetch(`https://amp-api.music.apple.com/v1/me/library?ids[songs]=${{trackId}}`, {{
                method: 'POST',
                headers: {{
                    'Authorization': `Bearer ${{devToken}}`,
                    'Music-User-Token': userToken,
                    'Content-Type': 'application/json'
                }}
            }});

            await new Promise(resolve => setTimeout(resolve, 1500));

            // 2. Push the track into the target playlist with a guaranteed JSON body
            const plRes = await fetch(`https://amp-api.music.apple.com/v1/me/library/playlists/${{playlistId}}/tracks`, {{
                method: 'POST',
                headers: {{
                    'Authorization': `Bearer ${{devToken}}`,
                    'Music-User-Token': userToken,
                    'Content-Type': 'application/json'
                }},
                body: JSON.stringify({{
                    data: [
                        {{
                            id: trackId,
                            type: "songs"
                        }}
                    ]
                }})
            }});

            if (!plRes.ok) {{
                const errorDetails = await plRes.text();
                throw new Error(`Playlist insertion failed (${{plRes.status}}): ${{errorDetails}}`);
            }}

            return `Successfully added track ${{trackId}} to playlist ${{playlistId}}`;
        }})();
        "#,
        track_id, pl_id
    );

    let res = self.execute(script, vec![]).await
        .map_err(|e| e.to_string())?;

    let message: String = serde_json::from_value(res.json().clone()).map_err(|e| e.to_string())?;

    Ok(message)
}
    //WORKING
    async fn add_to_fav(&self, track_id: &str) -> Result<String, String> {
    let script = format!(
        r#"
        return (async () => {{
            const mk = MusicKit.getInstance();
            const devToken = mk.developerToken;
            const userToken = mk.musicUserToken;
            const trackId = "{}";

            if (!devToken || !userToken) {{
                throw new Error("MusicKit tokens not found. Ensure you are authorized.");
            }}

            // POST to the Apple Music favorites endpoint
            const res = await fetch(`https://amp-api.music.apple.com/v1/me/favorites?ids[songs]=${{trackId}}`, {{
                method: 'POST',
                headers: {{
                    'Authorization': `Bearer ${{devToken}}`,
                    'Music-User-Token': userToken,
                    'Content-Type': 'application/json'
                }}
            }});

            if (!res.ok && res.status !== 202) {{
                const errorDetails = await res.text();
                throw new Error(`Failed to favorite track (${{res.status}}): ${{errorDetails}}`);
            }}

            return `Successfully added track ${{trackId}} to favorites`;
        }})();
        "#,
        track_id
    );

    let res = self.execute(script, vec![]).await
        .map_err(|e| e.to_string())?;

    let message: String = serde_json::from_value(res.json().clone()).map_err(|e| e.to_string())?;

    Ok(message)
}


}

