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
    async fn play_next(&self, item_type: &MediaType, id: &str) -> Result<String, String>;
    async fn set_queue(&self, item_type: &MediaType, id: &str) -> Result<String, String>;
    async fn play(&self) -> Result<(), String>; 
    async fn pause(&self) -> Result<(), String>; 
    async fn next_track(&self) -> Result<(), String>; 
    async fn previous_track(&self) -> Result<(), String>;
    async fn shuffle_toggle(&self) -> Result<i64, String>;
    async fn list_library_playlists(&self) -> Result<Vec<Playlist>, String>; 
    async fn get_current_track(&self) -> Result<Option<Track>, String>; 
    async fn clear_queue(&self) -> Result<(), String>; 
    async fn remove_queue(&self, index: u32) -> Result<(), String>;
    async fn list_queue(&self) -> Result<Vec<Track>, String>;
    async fn volume(&self, val: Option<u32>) -> Result<i32, String>;
}

impl MusicDriver for WebDriver {
 async fn search(&self, query: &str, item_type: Vec<MediaType>, is_library: bool, limit: u32) -> Result<SearchResult, String> {
    let mut media_type_string = String::new();
    let mut types_json_array = String::from("[");
    
    for (i, item) in item_type.iter().enumerate() {
        if i > 0 {
            media_type_string.push_str(",");
            types_json_array.push_str(",");
        }
        media_type_string.push_str(item.as_str());
        types_json_array.push_str(&format!("\"{}\"", item.as_str()));
    }
    types_json_array.push_str("]");

    let script: String = if is_library {
        format!(
            r#"
            return (async () => {{
                try {{
                    const mk = MusicKit.getInstance();
                    if (!mk || !mk.isAuthorized) {{
                        await mk?.authorize();
                    }}

                    const searchResponse = await mk.api.music(`v1/me/library/search`, {{
                        term: "{}",
                        types: "library-{}",
                        limit: {}
                    }});

                    const resultsObj = searchResponse?.data?.results || {{}};
                    const requestedTypes = {};

                    for (const rawType of requestedTypes) {{
                        const key = `library-${{rawType}}`;
                        const items = resultsObj[key]?.data;
                        if (items && items.length > 0) {{
                            const data = items[0];
                            const attr = data?.attributes || {{}};

                            if (rawType === "songs") {{
                                return {{
                                    type: "Track",
                                    id: data.id || null,
                                    title: attr.name || null,
                                    artist: attr.artistName || null,
                                    album: attr.albumName || null,
                                    duration_ms: attr.durationInMillis || null
                                }};
                            }} else if (rawType === "albums") {{
                                return {{
                                    type: "Album",
                                    id: data.id || null,
                                    name: attr.name || null,
                                    artist: attr.artistName || null,
                                    track_count: attr.trackCount || null
                                }};
                            }} else if (rawType === "playlists") {{
                                return {{
                                    type: "Playlist",
                                    id: data.id || null,
                                    name: attr.name || null,
                                    desc: attr.description?.standard || null
                                }};
                            }} else if (rawType === "artists") {{
                                return {{
                                    type: "Artist",
                                    id: data.id || null,
                                    name: attr.name || null
                                }};
                            }}
                        }}
                    }}

                    return null;
                }} catch (err) {{
                    console.error("Library Search Error:", err);
                    return null;
                }}
            }})();
            "#,
            query,
            media_type_string,
            limit,
            types_json_array,
        )
    } else {
        format!(
            r#"
            return (async () => {{
                try {{
                    const mk = MusicKit.getInstance();
                    if (!mk || !mk.isAuthorized) {{
                        await mk?.authorize();
                    }}

                    const storefront = mk.storefrontId || mk.storefront || 'us';

                    const searchResponse = await mk.api.music(`v1/catalog/${{storefront}}/search`, {{
                        term: "{}",
                        types: "{}",
                        limit: {}
                    }});

                    const resultsObj = searchResponse?.data?.results || {{}};
                    const requestedTypes = {};

                    for (const rawType of requestedTypes) {{
                        const items = resultsObj[rawType]?.data;
                        if (items && items.length > 0) {{
                            const data = items[0];
                            const attr = data?.attributes || {{}};

                            if (rawType === "songs") {{
                                return {{
                                    type: "Track",
                                    id: data.id || null,
                                    title: attr.name || null,
                                    artist: attr.artistName || null,
                                    album: attr.albumName || null,
                                    duration_ms: attr.durationInMillis || null
                                }};
                            }} else if (rawType === "albums") {{
                                return {{
                                    type: "Album",
                                    id: data.id || null,
                                    name: attr.name || null,
                                    artist: attr.artistName || null,
                                    track_count: attr.trackCount || null
                                }};
                            }} else if (rawType === "playlists") {{
                                return {{
                                    type: "Playlist",
                                    id: data.id || null,
                                    name: attr.name || null,
                                    desc: attr.description?.standard || null
                                }};
                            }} else if (rawType === "artists") {{
                                return {{
                                    type: "Artist",
                                    id: data.id || null,
                                    name: attr.name || null
                                }};
                            }}
                        }}
                    }}

                    return null;
                }} catch (err) {{
                    console.error("Catalog Search Error:", err);
                    return null;
                }}
            }})();
            "#,
            query,
            media_type_string,
            limit,
            types_json_array,
        )
    };

    let res = self.execute(script, vec![]).await
        .map_err(|e| e.to_string())?;

    let search_result: Option<SearchResult> = serde_json::from_value(res.json().clone())
        .map_err(|e| format!("Deserialization error: {}", e))?;

    search_result.ok_or_else(|| "No Matches Found".to_string())
}
    async fn play_next(&self, item_type: &MediaType, id: &str) -> Result<String, String> {
    let script = format!(r#"
    return (async () => {{
        const mk = MusicKit.getInstance();
        if (!mk || !mk.isAuthorized ) {{
        await mk?.authorize();
    }}
    try {{ 
    await mk.playNext({{ {}: '{}'}});
    return true;
    }}
    catch (e) {{
        return false;
    }}
    }})()
    "#, item_type.as_str(), id);

    let res = self.execute(script, vec![]).await
        .map_err(|e| e.to_string())?;


    let is_success: bool = serde_json::from_value(res.json().clone())
        .map_err(|e| e.to_string())?;

    if is_success {
        return Ok(id.to_string())
    }

    Err("Could Not play next, maybe wrong id".to_string())
        
    }
    async fn set_queue(&self, item_type: &MediaType, id: &str) -> Result<String, String> {
    let script = format!(r#"
    return (async () => {{
        const mk = MusicKit.getInstance();
        if (!mk || !mk.isAuthorized ) {{
        await mk?.authorize();
    }}
    try {{ 
    await mk.setQueue({{ {}: '{}'}});
    return true;
    }}
    catch (e) {{
        return false;
    }}
    }})()
    "#, item_type.as_str(), id);

    let res = self.execute(script, vec![]).await
        .map_err(|e| e.to_string())?;


    let is_success: bool = serde_json::from_value(res.json().clone())
        .map_err(|e| e.to_string())?;

    if is_success {
        return Ok(id.to_string())
    }

    Err("Could Not play, maybe wrong id".to_string())
        
    }
    async fn play(&self) -> Result<(), String> {
    let script = "MusicKit.getInstance().play();";
    self.execute(script, vec![]).await.map_err(|e| e.to_string())?;
    Ok(())
    }   
    async fn pause(&self) -> Result<(), String> {
    let script = "MusicKit.getInstance().pause();";
    self.execute(script, vec![]).await.map_err(|e| e.to_string())?;
    Ok(())
}
    async fn next_track(&self) -> Result<(), String> {
    let script = "MusicKit.getInstance().skipToNextItem();";
    self.execute(script, vec![]).await.map_err(|e| e.to_string())?;
    Ok(())
}
    async fn previous_track(&self) -> Result<(), String> {
    let script = "MusicKit.getInstance().skipToPreviousItem();";
    self.execute(script, vec![]).await.map_err(|e| e.to_string())?;
    Ok(())
}
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
    async fn list_library_playlists(&self) -> Result<Vec<Playlist>, String> {
    let script = r#"
        return (async () => {
            try {
                const mk = MusicKit.getInstance();
                if (!mk || !mk.isAuthorized) {
                    await mk?.authorize();
                }

                const response = await mk.api.music('v1/me/library/playlists', { limit: 100 });
                const rawItems = response?.data?.data || [];

                // 2. Return the FLATTENED mapped array, NOT 'rawItems'
                return rawItems.map(p => ({
                    id: p.id,
                    name: p.attributes?.name || null,
                    desc: p.attributes?.description?.standard || null,
                }));
            } catch (err) {
                console.error("MusicKit error:", err);
                return [];
            }
        })();
    "#;

    let res = self
        .execute(script, vec![])
        .await
        .map_err(|e| format!("WebDriver script execution failed: {}", e))?;

    let json_val = res.json();

    // 3. Explicit error reporting instead of silent fallback
    serde_json::from_value::<Vec<Playlist>>(json_val.clone()).map_err(|e| {
        format!(
            "Deserialization failed for JSON `{}`: {}",
            json_val, e
        )
    })
}
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
    async fn clear_queue(&self) -> Result<(), String> {
    let script = r#"
    MusicKit.getInstance().clearQueue()
    "#;

    self.execute(script, vec![]).await.map_err(|e| e.to_string())?;
    
    Ok(())
}
    async fn remove_queue(&self, index: u32) -> Result<(), String> {
    let script = format!(r#"
    MusicKit.getInstance().queue.remove({})
    "#, index);
    self.execute(script, vec![]).await.map_err(|e| e.to_string())?;
    
    Ok(())
}
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
}

