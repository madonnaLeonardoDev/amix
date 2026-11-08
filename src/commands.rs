use clap::{Parser, Subcommand, ValueEnum, Args, ArgGroup};
use thirtyfour::WebDriver;
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
pub struct LsQueueArgs {
    #[arg(short, long)]
    pub result: Option<u32>,
}

#[derive(Args, Debug, Clone, SchemaRead, SchemaWrite)]
pub struct LsSearchArgs {
    pub query: String,
    #[arg(short, long)]
    pub result: Option<u32>,
    #[arg(short, long)]
    pub library: bool,
    #[arg(short = 't', long = "type")]
    pub search_type: Option<MediaType>,
}

#[derive(Subcommand, Debug, Clone, SchemaRead, SchemaWrite)]
pub enum LsSubcommands {
    /// List current playback queue
    Queue(LsQueueArgs),

    /// List tracks in a specific playlist
    Playlist
}

#[derive(Args, Debug, Clone, SchemaRead, SchemaWrite)]
pub struct LsCommand {
    #[command(subcommand)]
    pub subcommand: Option<LsSubcommands>,

    #[command(flatten)]
    pub search_args: LsSearchArgs,
}

#[derive(Args, Debug, Clone, SchemaRead, SchemaWrite)]
pub struct RmQueue {
    pub queue: u32,
}

#[derive(Args, Debug, Clone, SchemaRead, SchemaWrite)]
#[command(
    group = ArgGroup::new("track_target")
        .required(true)
        .multiple(false),
    group = ArgGroup::new("playlist_target")
        .required(true)
        .multiple(false),
)]
pub struct RmFromPlaylist {
    #[arg(value_name = "TRACK_QUERY", group = "track_target", help = "Track name or query")]
    pub track_query: String,

    #[arg(value_name = "PLAYLIST_QUERY", group = "playlist_target", help = "Playlist name or query")]
    pub playlist_query: String,
}

/// The two mutually exclusive modes for the rm command
#[derive(Args, Debug, Clone, SchemaRead, SchemaWrite)]
#[group(required = true, multiple = false)]
pub struct RmCommand {
    #[command(flatten)]
    pub queue_mode: Option<RmQueue>,

    #[command(flatten)]
    pub playlist_mode: Option<RmFromPlaylist>,
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
pub struct MkPlCommand{
    #[arg(
        value_name = "playlist_name",
        help = "playlist name"
    )]
    pub playlist_name: String,
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

    #[arg(
        long = "current",
        group = "slot1_target",
        help = "Target the currently playing track",
        action = clap::ArgAction::SetTrue
    )]
    pub current: bool,

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
    Ls(LsCommand),

    #[command()]
    Rm(RmCommand),

    #[command()]
    Rmpl(RmPlCommand),

    #[command()]
    Mkpl(MkPlCommand),

    #[command()]
    Mv(MvCommand),

    #[command()]
    Shuffle,

    #[command(visible_alias = "auto")]
    Autoplay,

    #[command()]
    Current,

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
    PlayStop,
    NextTrack,
    PrevTrack,
    ShuffleToggle,
    AutoPlayToggle,
    Volume(Option<u32>),
    SetQueue(SearchArgs),
    QueueNext(SearchArgs),
    QueueAdd(SearchArgs),
    Search(SearchArgs),
    ListPlaylists,
    ListQueueTracks(Option<u32>),
    RmQueue(Option<u32>),
    RmFromPlaylist{
        track_target_search: SearchArgs,
        playlist_target_search: SearchArgs
    },
    RmPlaylist(SearchArgs),
    MkPlaylist(String),
    MvToPlaylist{
        track_target: Option<SearchArgs>,
        is_track_target_current: bool,
        playlist_target: Option<SearchArgs>,
        is_playlist_target_fav: bool
    },
    Current,

    //INTERNAL COMMANDS

    DisplaModeDaemon,
    GetNextTrackMeta
}
impl Commands {
    pub fn into_player_cmd(&self) -> Result<PlayerCommand, String> {
        match self {
            Commands::Play { target, play_type, library } => {
                eprint!("DEBUG: {:?}", self);
                if target.is_none(){
                    eprint!("TAEGET IS NONE");
                    return Ok(PlayerCommand::PlayStop)
                }
                let search_type = if let Some(t) = play_type.to_owned() {
                    vec![t]
                } else {
                    let mut t = Vec::new();
                    t.push(MediaType::Song);
                    t.push(MediaType::Artist);
                    t.push(MediaType::Album);
                    t.push(MediaType::Playlist);
                    t
                };
                eprint!("SetQueue sent");
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
                if *next {
                    return Ok(PlayerCommand::QueueNext(
                    SearchArgs { target: target.clone().unwrap(), search_type: search_type, is_target_library: *library, limit: Some(1) }
                    ))
                }
                Ok(PlayerCommand::QueueAdd(
                    SearchArgs { target: target.clone().unwrap(), search_type: search_type, is_target_library: *library, limit:Some(1) }
                ))
            },

            Commands::Ls(command) => {
                if let Some(subcommand) = command.subcommand.as_ref() {
                    match subcommand {
                        LsSubcommands::Playlist => {
                            return Ok(PlayerCommand::ListPlaylists);
                        }
                        LsSubcommands::Queue(queue_args) => {
                           return Ok(PlayerCommand::ListQueueTracks(queue_args.result))
                        }
                    }



                } else {
                    let query = &command.search_args.query;
                    let result = command.search_args.result;
                    let is_library = command.search_args.library;
                    let search_type = if let Some(t) = command.search_args.search_type {
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
                }
            }

            Commands::Rm(command) => {
                match (command.playlist_mode.clone(), command.queue_mode.clone()) {
                    (Some(rm_from_pl), None) => {
                        let track_target_search: String = rm_from_pl.track_query;

                        let playlist_target_search: String = rm_from_pl.playlist_query;



                        return Ok(PlayerCommand::RmFromPlaylist { track_target_search: SearchArgs {
                                target: track_target_search, search_type: vec![MediaType::Song], is_target_library: true , limit: Some(1)}, 

                             playlist_target_search: SearchArgs {
                                target: playlist_target_search,  search_type: vec![MediaType::Playlist], is_target_library: true, limit: Some(1)} })
                    },
                    (None, Some(rmq)) => {
                        return Ok(PlayerCommand::RmQueue(Some(rmq.queue)));
                    },
                    (None, None) => {
                        return Ok(PlayerCommand::RmQueue(None));
                    },
                    (Some(_), Some(_)) => {
                        return Err("Either rm queue or rm from pl".to_string())
                    }
                }
            }

            Commands::Rmpl(command) => {
                        let target = &command.playlist_query;
                        return Ok(PlayerCommand::RmPlaylist(SearchArgs 
                            { target: target.to_string(), search_type: vec![MediaType::Playlist], is_target_library: true , limit: Some(1)}));
            }

            Commands::Mkpl(command) => {
                return Ok(PlayerCommand::MkPlaylist(command.playlist_name.clone()))
            }

            Commands::Mv(command) => {
                let (track_target, is_track_target_current): (Option<String>, bool) = 
                    match (command.track_query.as_ref(), command.current){
                        (Some(q), false) => {
                            (Some(q.to_string()), false)
                        },
                        (None, true) => {
                            (None, true)
                        },
                        (Some(_), true) => {
                            return Err("Either current track or query".to_string());
                        },
                        (None, false) => {
                            return Err("Either current track or query".to_string());
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

                let track_sc =  if let Some(target) = track_target {
                    Some(SearchArgs { target, search_type: vec![MediaType::Song], is_target_library: false , limit: Some(1)})
                } else {
                    None
                };
                let playlist_sc =  if let Some(target) = playlist_target {
                    Some(SearchArgs { target, search_type: vec![MediaType::Playlist], is_target_library: true , limit: Some(1)})
                } else {
                    None
                };
                return  Ok(PlayerCommand::MvToPlaylist { track_target: track_sc , is_track_target_current, playlist_target: playlist_sc, is_playlist_target_fav })
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
            }
            
            _=> {
                return Err("Command not supported".to_string())
            }
        }
    }
}