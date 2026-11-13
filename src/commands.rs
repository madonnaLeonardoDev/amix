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
    pub query: String,
    #[arg(short, long)]
    pub result: Option<u32>,
    #[arg(short, long)]
    pub library: bool,
    #[arg(short = 't', long = "type")]
    pub search_type: Option<MediaType>,
}

#[derive(Args, Debug, Clone, SchemaRead, SchemaWrite)]
pub struct RmPlCommand{
    #[arg(
        value_name = "playlist_query",
        group = "playlist_target",
        help = "playlist name or query"
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
    /// Queue index to remove or '*' for all
    #[arg(
        value_name = "INDEX",
        group = "rm_target",
        help = "Queue index (number) or '*' to clear"
    )]
    pub index: Option<String>,

    /// Remove a track from a playlist (-p <TRACK> <PLAYLIST>)
    #[arg(
        short = 'p',
        num_args = 2,
        value_names = &["TRACK", "PLAYLIST"],
        group = "rm_target",
        help = "Track and playlist names"
    )]
    pub p_args: Option<Vec<String>>,
}

#[derive(Args, Debug, Clone, SchemaRead, SchemaWrite)]
#[command(
    // Slot 1: Must be either positional query, --track-id, or --current
    group = ArgGroup::new("slot1_target")
        .required(true)
        .multiple(false),
    // Slot 2: Must be either positional query, --playlist-id, or --fav
    group = ArgGroup::new("slot2_target")
        .required(true)
        .multiple(false),
)]
pub struct MvCommand {
    // ==================== FIRST POSITION ====================
    #[arg(
        value_name = "TRACK_QUERY",
        group = "slot1_target",
        help = "Track name or query string"
    )]
    pub track_query: Option<String>,

    // ==================== SECOND POSITION ====================
    #[arg(
        value_name = "PLAYLIST_QUERY",
        group = "slot2_target",
        help = "Playlist name or query string"
    )]
    pub playlist_query: Option<String>,

    #[arg(
        long = "fav",
        group = "slot2_target",
        help = "Target favorites playlist",
        action = clap::ArgAction::SetTrue
    )]
    pub fav: bool,
}






#[derive(Subcommand, SchemaRead, SchemaWrite, Debug)]
pub enum Commands {

    ///Start the background services that handle the music player
    #[command(visible_alias = "sd")]
    Startd{
        #[arg(short = 'H', long)]
        head:bool
    },

    ///Kill all the services that handle the music player
    #[command(visible_alias = "kd")]
    Killd,

    #[command()]
    Play{
        #[arg(index = 1, value_name = "Query/id")]
        target: Option<String>,

        #[arg(short = 't', long = "type")]
        play_type: Option<MediaType>,

        #[arg(short, long)]
        library: bool,
    },

    #[command()]
    Queue{
        target: Option<String>,

        #[arg(short = 't', long = "type")]
        play_type: Option<MediaType>,

        #[arg(short, long)]
        library: bool,

        #[arg(short, long)]
        next: bool,
    },
    #[command()]
    Ls{
        result: Option<u32>
    },
    #[command()]
    Find(FindSearchArgs),

    #[command()]
    Rm(RmCommand),
    #[command()]
    Rmpl(RmPlCommand),

    #[command()]
    Mkpl{
        name: String,
        desc: Option<String>
    },

    #[command()]
    Mv(MvCommand),

    #[command()]
    Shuffle,

    #[command(visible_alias = "auto")]
    Autoplay,

    #[command()]
    Current,

    #[command(visible_alias = "nx")]
    Next,

    #[command(visible_alias = "pv")]
    Previous,

    #[command()]
    Volume{
        value: Option<u32>,
    },

    #[command()]
    Display{
        #[arg(short, long)]
        lyrics: bool
    }

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
        track_target_search: SearchArgs,
        playlist_target_search: SearchArgs
    },
    RmPlaylist(SearchArgs),
    //WORKING
    MkPlaylist{
        name: String,
        desc: String
    },

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
                        t.push(MediaType::Album);
                        t.push(MediaType::Song);
                        t.push(MediaType::Artist);
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
                            track_target_search: SearchArgs { target: args[0].to_owned(), search_type: vec![MediaType::Song], is_target_library: true, limit: Some(1) },
                             playlist_target_search: SearchArgs { target: args[1].to_owned(), search_type: vec![MediaType::Playlist], is_target_library: true, limit: Some(1) } })
                    },
                    (None, None) => return Err("Either rm -p [INDEX / *] or rm -p [Query / Id] [Query / Id]".to_string()),
                    (Some(_), Some(_)) => return Err("Either rm -p [INDEX / *] or rm -p [Query / Id] [Query / Id]".to_string())

                }
            }

            Commands::Rmpl(command) => {
                        let target = &command.playlist_query;
                        return Ok(PlayerCommand::RmPlaylist(SearchArgs 
                            { target: target.to_string(), search_type: vec![MediaType::Playlist], is_target_library: true , limit: Some(1)}));
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