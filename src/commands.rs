use clap::{Parser, Subcommand, ValueEnum, Args, ArgGroup};
use thirtyfour::{WebDriver};
use wincode_derive::{SchemaRead, SchemaWrite};

use crate::browser_interface::{MusicDriver, SearchResult};

#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq, SchemaRead, SchemaWrite)]
pub enum MediaType {
    #[value(alias = "al")]
    Album,
    #[value(alias = "p")]
    Playlist,
    #[value(alias = "s")]
    Song,
    #[value(alias = "ar")]
    Artist
}

impl MediaType {
    pub fn as_str<'a>(&self) -> &'a str {
        match self{
            Self::Album => {
                "albums"
            },
            Self::Playlist => {
                "playlists"
            },
            Self::Artist => {
                "artists"
            },
            Self::Song => {
                "songs"
            }
        }
    }
}

#[derive(Args, Debug, Clone, SchemaRead, SchemaWrite)]
pub struct FindSearchArgs {
    /// Text to search for
    pub query: String,

    /// Number of results to show
    #[arg(short, long)]
    pub result: Option<u32>,

    /// Search your library instead of the Apple Music catalog
    #[arg(short, long)]
    pub library: bool,

    /// Only look for one kind of item (song, album, artist or playlist)
    #[arg(short = 't', long = "type")]
    pub search_type: Option<MediaType>,
}

#[derive(Args, Debug, Clone, SchemaRead, SchemaWrite)]
pub struct RmPlCommand {
    #[arg(
        value_name = "playlist_query",
        group = "playlist_target",
        help = "Name of the playlist to remove (or a query that matches it)"
    )]
    pub playlist_query: String,
}

#[derive(Args, Debug, Clone, SchemaRead, SchemaWrite)]
#[command(
    group = ArgGroup::new("rm_target")
        .required(true)
        .multiple(false),
)]
pub struct RmCommand {
    #[arg(
        value_name = "INDEX",
        group = "rm_target",
        help = "Queue position to remove, or '*' to clear the whole queue"
    )]
    pub index: Option<String>,

    #[arg(
        short = 'p',
        num_args = 2,
        value_names = &["TRACK", "PLAYLIST"],
        group = "rm_target",
        help = "Remove a track from a playlist, given the track and playlist names"
    )]
    pub p_args: Option<Vec<String>>,
}

/// Add a track to a playlist (or move it there)
#[derive(Args, Debug, Clone, SchemaRead, SchemaWrite)]
#[command(
    group = ArgGroup::new("slot1_target")
        .required(true)
        .multiple(false),
    group = ArgGroup::new("slot2_target")
        .required(true)
        .multiple(false),
)]
pub struct MvCommand {
    #[arg(
        value_name = "TRACK_QUERY",
        group = "slot1_target",
        help = "Track name or query string"
    )]
    pub track_query: Option<String>,

    #[arg(
        value_name = "PLAYLIST_QUERY",
        group = "slot2_target",
        help = "Name of the destination playlist (or a query that matches it)"
    )]
    pub playlist_query: Option<String>,

    #[arg(
        long = "fav",
        group = "slot2_target",
        help = "Use your favorites playlist as the destination",
        action = clap::ArgAction::SetTrue
    )]
    pub fav: bool,
}

#[derive(Subcommand, SchemaRead, SchemaWrite, Debug)]
pub enum Commands {
    /// Start the background services that handle the music player
    #[command(visible_alias = "sd")]
    Startd {
        /// Open the Apple Music web player in a visible (headed) browser window
        #[arg(short = 'H', long)]
        head: bool,
    },

    /// Kill all the services that handle the music player
    #[command(visible_alias = "kd")]
    Killd,

    /// Play a song, album, artist or playlist, or toggle play/pause if no target is given
    #[command()]
    Play {
        /// Search query or ID of what to play
        #[arg(index = 1, value_name = "Query/id")]
        target: Option<String>,

        /// Kind of item to play (song, album, artist or playlist)
        #[arg(short = 't', long = "type")]
        play_type: Option<MediaType>,

        /// Look in your library instead of the Apple Music catalog
        #[arg(short, long)]
        library: bool,
    },

    /// Add an item to the queue
    #[command()]
    Queue {
        /// Search query or ID of the item to queue
        target: Option<String>,

        /// Kind of item to queue (song, album, artist or playlist)
        #[arg(short = 't', long = "type")]
        play_type: Option<MediaType>,

        /// Look in your library instead of the Apple Music catalog
        #[arg(short, long)]
        library: bool,

        /// Play it next instead of adding it to the end of the queue
        #[arg(short, long)]
        next: bool,
    },

    /// List the tracks in the playing queue
    #[command()]
    Ls {
        /// Number of tracks to show
        result: Option<u32>,
    },

    /// Search for music
    #[command()]
    Find(FindSearchArgs),

    /// Remove a track from the queue or from a playlist
    #[command()]
    Rm(RmCommand),

    /// Remove a playlist from your library
    #[command()]
    Rmpl(RmPlCommand),

    /// Create a new, empty playlist
    #[command()]
    Mkpl {
        /// Name of the new playlist
        name: String,
        /// Optional description for the playlist
        desc: Option<String>,
    },

    /// Add a track to a playlist, or move it there
    #[command()]
    Mv(MvCommand),

    /// Shuffle the queue
    #[command()]
    Shuffle,

    /// Toggle autoplay
    #[command(visible_alias = "auto")]
    Autoplay,

    /// Show the track that is currently playing
    #[command()]
    Current,

    /// Skip to the next track
    #[command(visible_alias = "nx")]
    Next,

    /// Go back to the previous track
    #[command(visible_alias = "pv")]
    Previous,

    /// Show the current volume, or set it if a value is given
    #[command()]
    Volume {
        /// New volume level from 0 to 100; omit to print the current volume
        value: Option<u32>,
    },

    /// Show the artwork and a player dock in the terminal (needs a terminal that can render images)
    #[command()]
    Display {
        /// Also show the lyrics of the current track
        #[arg(short, long)]
        lyrics: bool,
    },
}

#[derive(SchemaRead, SchemaWrite)]
pub struct SearchArgs{
    pub target: String,
    pub search_type: Vec<MediaType>,
    pub is_target_library: bool,
    pub limit: Option<u32>
}

impl SearchArgs {
    pub async fn resolve_search(self, driver: &WebDriver) -> Result<SearchResult, String> {
        let limit = if let Some(l) = self.limit{
            l
        } else {
            10
        };
        let res = driver.search(&self.target, self.search_type, self.is_target_library, limit).await?;
        Ok(res)
    } 
}

#[derive(SchemaRead, SchemaWrite)]
pub enum PlayerCommand{
    //WORKING
    PlayStop,
    //WORKING
    NextTrack,
    //WORKING
    PrevTrack,
    //WORKING
    ShuffleToggle,
    //WORKING
    AutoPlayToggle,
    //WORKING
    Volume(Option<u32>),
    //WORKING
    SetQueue(SearchArgs),
    //WORKING
    QueueAdd{
        search: SearchArgs,
        is_next: bool
    },
    //WORKING
    Search(SearchArgs),
    //WORKING
    ListQueueTracks(Option<u32>),
    //WORKING
    RmQueue(Option<u32>),

    RmFromPlaylist{
        track_target_search: String,
        playlist_target_search: String
    },
    //WORKING
    RmPlaylist(String),
    //WORKING
    MkPlaylist{
        name: String,
        desc: String
    },
    //WORKING
    MvToPlaylist{
        track_target: Option<String>,
        is_track_target_current: bool,
        playlist_target: Option<String>,
        is_playlist_target_fav: bool
    },
    //WORKING
    Current,

    //INTERNAL COMMANDS

    DisplaModeDaemon,
    GetNextTrackMeta
}
impl Commands {
    pub fn into_player_cmd(&self) -> Result<PlayerCommand, String> {
        match self {
            Commands::Play { target, play_type, library } => {
                if target.is_none(){
                    return Ok(PlayerCommand::PlayStop)
                }
                let search_type = if let Some(t) = play_type.to_owned() {
                    vec![t]
                } else {
                    let mut t = Vec::new();
                    t.push(MediaType::Song);
                    t.push(MediaType::Album);
                    t.push(MediaType::Artist);
                    t.push(MediaType::Playlist);
                    t
                };
                Ok(PlayerCommand::SetQueue(
                    SearchArgs { target: target.clone().unwrap(), search_type: search_type, is_target_library: *library, limit: None }
                ))
            },

            Commands::Queue { target, play_type, library , next} => {
                if target.is_none() && !library {
                    return Ok(PlayerCommand::PlayStop)
                };
                let search_type = if let Some(t) = play_type.as_ref() {
                    vec![t.to_owned()]
                } else {
                    let mut t = Vec::new();
                    t.push(MediaType::Song);
                    t.push(MediaType::Artist);
                    t.push(MediaType::Album);
                    t.push(MediaType::Playlist);
                    t
                };
                Ok(PlayerCommand::QueueAdd{
                    search: SearchArgs { target: target.clone().unwrap(), search_type: search_type, is_target_library: *library, limit:Some(1) },
                    is_next: *next
                })
            },
            Commands::Find(search_args) => {
                    let query = &search_args.query;
                    let result = search_args.result;
                    let is_library = search_args.library;
                    let search_type = if let Some(t) = search_args.search_type {
                        vec![t]
                    } else {
                    let mut t = Vec::new();
                        t.push(MediaType::Song);
                        t.push(MediaType::Artist);
                        t.push(MediaType::Album);
                        t.push(MediaType::Playlist);
                        t
                    };

                    return Ok(PlayerCommand::Search (SearchArgs { target: query.to_string(), search_type, is_target_library: is_library , limit: result}))
            },
            Commands::Ls { result } => {
                Ok(PlayerCommand::ListQueueTracks(*result))
            },
            Commands::Rm(command) => {
                match (command.index.as_ref(), command.p_args.as_ref()) {
                    (Some(i), None) => {
                        let index = if i == "%" {
                            None
                        } else {
                            let index = match i.parse::<u32>() {
                                Ok(i) => i,
                                Err(_) => return Err("Index argument has to be a valid number or '*' to select all".to_string())
                            };
                            Some(index)
                        };
                        return Ok(PlayerCommand::RmQueue(index))
                    },
                    (None, Some(args)) => {
                        return Ok(PlayerCommand::RmFromPlaylist { 
                            track_target_search: args[0].to_owned(),
                             playlist_target_search: args[1].to_owned()
                        })
                    },
                    (None, None) => return Err("Either rm -p [INDEX / *] or rm -p [Query / Id] [Query / Id]".to_string()),
                    (Some(_), Some(_)) => return Err("Either rm -p [INDEX / *] or rm -p [Query / Id] [Query / Id]".to_string())

                }
            }

            Commands::Rmpl(command) => {
                        let target = command.playlist_query.to_owned();
                        return Ok(PlayerCommand::RmPlaylist(target));
            }

            Commands::Mkpl{name, desc} => {
                return Ok(PlayerCommand::MkPlaylist { name: name.to_string(), desc: desc.to_owned().unwrap_or("".to_string()) })
            }

            Commands::Mv(command) => {
                let (track_target, is_track_target_current): (Option<String>, bool) = 
                    match command.track_query.as_ref(){
                        Some(q) => {
                            if q == "@" {
                                (None, true)
                            } else {
                                (Some(q.to_string()), false)
                            }
                            
                        },
                        None => {
                            return Err("Either current track [@] or query".to_string());
                        }
                    };

                    let (playlist_target, is_playlist_target_fav): (Option<String>, bool) = 
                        match (command.playlist_query.as_ref(), command.fav){
                            (Some(q), false) => {
                                (Some(q.to_string()), false)
                            },
                            (None, true) => {
                                (None, true)
                            },
                            (Some(_), true) => {
                                return Err("Either fav playlist or query".to_string());
                            },
                            (None, false) => {
                                return Err("Either fav playlist or query".to_string());
                            }
                        };
                return  Ok(PlayerCommand::MvToPlaylist { track_target: track_target , is_track_target_current, playlist_target: playlist_target, is_playlist_target_fav })
            }
        
            Commands::Volume { value } => {
                return Ok(PlayerCommand::Volume(*value))
            }

            Commands::Autoplay => {
                return Ok(PlayerCommand::AutoPlayToggle)
            }

            Commands::Shuffle => {
                return Ok(PlayerCommand::ShuffleToggle)
            },
            Commands::Current => {
                return Ok(PlayerCommand::Current);
            },
            Commands::Next => {
                return Ok(PlayerCommand::NextTrack);
            },
            Commands::Previous => {
                return Ok(PlayerCommand::PrevTrack)
            }
            
            _=> {
                return Err("Command not supported".to_string())
            }
        }
    }
}