use clap::{Parser, Subcommand, ValueEnum, Args, ArgGroup};
use wincode_derive::{SchemaRead, SchemaWrite};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaType {
    #[value(alias = "a")]
    Album,
    #[value(alias = "p")]
    Playlist,
    #[value(alias = "s")]
    Song,
    #[value(alias = "r")]
    Radio
}

#[derive(Args, Debug, SchemaRead, SchemaWrite)]
#[command(group(
    ArgGroup::new("play_method")
        .required(true)
        .args(["query", "id"]),
))]
#[derive(Clone)]
pub struct PlayTarget {
    pub query: Option<String>,

    #[arg(short = 'i', long = "id")]
    pub id: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct LsQueueArgs {
    #[arg(short, long)]
    pub result: Option<u32>,
}

#[derive(Args, Debug, Clone)]
pub struct LsPlaylistArgs {
    #[command(flatten)]
    pub target: PlayTarget,
    #[arg(short, long)]
    pub result: Option<u32>
}

#[derive(Args, Debug, Clone)]
pub struct LsSearchArgs {
    pub query: String,
    #[arg(short, long)]
    pub result: Option<u32>,
    #[arg(short, long)]
    pub library: bool,
    #[arg(short = 't', long = "type")]
    pub search_type: Option<MediaType>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum LsSubcommands {
    /// List current playback queue
    Queue(LsQueueArgs),

    /// List tracks in a specific playlist
    Playlist(LsPlaylistArgs)
}

#[derive(Args, Debug, Clone)]
pub struct LsCommand {
    #[command(subcommand)]
    pub subcommand: Option<LsSubcommands>,

    #[command(flatten)]
    pub search_args: LsSearchArgs,
}

#[derive(Args, Debug, Clone)]
pub struct RmQueue {
    pub queue: u32,
}

#[derive(Args, Debug, Clone)]
#[command(
    group = ArgGroup::new("track_target")
        .required(true)
        .multiple(false),
    group = ArgGroup::new("playlist_target")
        .required(true)
        .multiple(false),
)]
pub struct RmFromPlaylist {
    #[arg(long = "track-id", value_name = "TRACK_ID", group = "track_target", help = "Track ID")]
    pub track_id: Option<String>,

    #[arg(value_name = "TRACK_QUERY", group = "track_target", help = "Track name or query")]
    pub track_query: Option<String>,

    #[arg(long = "playlist-id", value_name = "PLAYLIST_ID", group = "playlist_target", help = "Playlist ID")]
    pub playlist_id: Option<String>,

    #[arg(value_name = "PLAYLIST_QUERY", group = "playlist_target", help = "Playlist name or query")]
    pub playlist_query: Option<String>,
}

/// The two mutually exclusive modes for the rm command
#[derive(Args, Debug, Clone)]
#[group(required = true, multiple = false)]
pub struct RmCommand {
    #[command(flatten)]
    pub queue_mode: Option<RmQueue>,

    #[command(flatten)]
    pub playlist_mode: Option<RmFromPlaylist>,
}

#[derive(Args, Debug, Clone)]
pub struct RmPlCommand{
        #[arg(
        short = 'i',
        long = "id",
        value_name = "PLAYLIST_ID",
        group = "playlist_target",
        help = "Playlist ID"
    )]
    pub playlist_id: Option<String>,

    #[arg(
        value_name = "playlist_query",
        group = "playlist_target",
        help = "playlist name or query"
    )]
    pub playlist_query: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct MkPlCommand{
    #[arg(
        value_name = "playlist_name",
        help = "playlist name"
    )]
    pub playlist_name: String,
}

#[derive(Args, Debug, Clone)]
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
        long = "track-id",
        value_name = "ID",
        group = "slot1_target",
        help = "Track ID"
    )]
    pub track_id: Option<String>,

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
        long = "playlist-id",
        value_name = "ID",
        group = "slot2_target",
        help = "Playlist ID"
    )]
    pub playlist_id: Option<String>,

    #[arg(
        long = "fav",
        group = "slot2_target",
        help = "Target favorites playlist",
        action = clap::ArgAction::SetTrue
    )]
    pub fav: bool,
}






#[derive(Subcommand)]
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
        #[command(flatten)]
        target: Option<PlayTarget>,

        #[arg(short = 't', long = "type")]
        play_type: Option<MediaType>,

        #[arg(short, long)]
        library: bool,
    },

    #[command()]
    Queue{
        #[command(flatten)]
        target: Option<PlayTarget>,

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
    Volume{
        value: Option<u32>,
    },


}

struct SearchArgs{
    target: String,
    is_target_id: bool,
    search_type: Vec<MediaType>,
    is_target_library: bool,
}



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
    Search{
        search: SearchArgs,
        result: Option<u32>
    },
    ListPlaylistTracks{
        search: SearchArgs,
        result: Option<u32>
    },
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
    }
}





impl Commands {
    pub fn into_player_cmd(&self) -> Result<PlayerCommand, String> {
        match self {
            Commands::Play { target, play_type, library } => {
                if target.is_none() && !library {
                    return Ok(PlayerCommand::PlayStop)
                }
                let search_type = if let Some(t) = play_type.to_owned() {
                    vec![t]
                } else {
                    let mut t = Vec::new();
                    t.push(MediaType::Song);
                    t.push(MediaType::Radio);
                    t.push(MediaType::Album);
                    t.push(MediaType::Playlist);
                    t
                };

                let t = target.as_ref().unwrap();

                let target:(String, bool)= if let Some(id) = &t.id {
                    (id.to_string(), true)
                } else {
                    (t.query.as_ref().unwrap().to_string(), false)
                };

                Ok(PlayerCommand::SetQueue(
                    SearchArgs { target: target.0, is_target_id: target.1, search_type: search_type, is_target_library: *library }
                ))
            },

            Commands::Queue { target, play_type, library , next} => {
                if target.is_none() && !library {
                    return Ok(PlayerCommand::PlayStop)
                }
                let search_type = if let Some(t) = play_type.to_owned() {
                    vec![t]
                } else {
                    let mut t = Vec::new();
                    t.push(MediaType::Song);
                    t.push(MediaType::Radio);
                    t.push(MediaType::Album);
                    t.push(MediaType::Playlist);
                    t
                };

                let t = target.as_ref().unwrap();

                let target:(String, bool)= if let Some(id) = t.id.as_ref() {
                    (id.to_string(), true)
                } else {
                    (t.query.as_ref().unwrap().to_string(), false)
                };
                if *next {
                    return Ok(PlayerCommand::QueueNext(
                    SearchArgs { target: target.0, is_target_id: target.1, search_type: search_type, is_target_library: *library }
                    ))
                }
                Ok(PlayerCommand::QueueAdd(
                    SearchArgs { target: target.0, is_target_id: target.1, search_type: search_type, is_target_library: *library }
                ))
            },

            Commands::Ls(command) => {
                if let Some(subcommand) = command.subcommand.clone() {
                    match subcommand {
                        LsSubcommands::Playlist(pl_args) => {
                            let result = pl_args.result;
                            let (target, is_target_id) = match (pl_args.target.query, pl_args.target.id) {
                                (Some(q), None) => (q, false),
                                (None, Some(id)) => (id, true),
                                (Some(_), Some(_)) => return Err("Either query or id".to_string()),
                                (None, None) => return Err("Pick a target either by query or id (--id)".to_string())
                            };

                            return Ok(PlayerCommand::ListPlaylistTracks { search: SearchArgs { target, is_target_id, search_type: vec![MediaType::Playlist], is_target_library: true }, result });
                        }
                        LsSubcommands::Queue(queue_args) => {
                           return Ok(PlayerCommand::ListQueueTracks(queue_args.result))
                        }
                    }



                } else {
                    let query = command.search_args.query.clone();
                    let result = command.search_args.result;
                    let is_library = command.search_args.library;
                    let search_type = if let Some(t) = command.search_args.search_type {
                        vec![t]
                    } else {
                        let mut t = Vec::new();
                        t.push(MediaType::Album);
                        t.push(MediaType::Song);
                        t.push(MediaType::Radio);
                        t.push(MediaType::Playlist);
                        t
                    };

                    return Ok(PlayerCommand::Search { search: SearchArgs { target: query, is_target_id: false, search_type, is_target_library: is_library }, result })
                }
            }

            Commands::Rm(command) => {
                match (command.playlist_mode.clone(), command.queue_mode.clone()) {
                    (Some(rm_from_pl), None) => {
                        let track_target_search: (String, bool) = match (rm_from_pl.track_query.as_ref(), rm_from_pl.track_query.as_ref()) {
                          (Some(q), None) => {
                            (q.to_string(), false)
                          },
                          (None, Some(id)) => {
                            (id.to_string(), true)
                          },
                          (Some(_), Some(_)) => {
                            return Err("Either query or id".to_string())  
                          } ,
                          (None, None) =>{
                            return Err("Either query or id".to_string())
                          }
                        };

                        let playlist_target_search: (String, bool) = match (rm_from_pl.playlist_query, rm_from_pl.playlist_id) {
                          (Some(q), None) => {
                            (q, false)
                          },
                          (None, Some(id)) => {
                            (id, true)
                          },
                          (Some(_), Some(_)) => {
                            return Err("Either query or id".to_string())  
                          } ,
                          (None, None) =>{
                            return Err("Either query or id".to_string())
                          }
                        };



                        return Ok(PlayerCommand::RmFromPlaylist { track_target_search: SearchArgs {
                                target: track_target_search.0, is_target_id: track_target_search.1, search_type: vec![MediaType::Song], is_target_library: true }, 

                             playlist_target_search: SearchArgs {
                                target: playlist_target_search.0, is_target_id: playlist_target_search.1, search_type: vec![MediaType::Playlist], is_target_library: true } })
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
                        let (target, is_id): (String, bool) = match (command.playlist_query.as_ref(), command.playlist_id.as_ref()) {
                          (Some(q), None) => {
                            (q.to_string(), false)
                          },
                          (None, Some(id)) => {
                            (id.to_string(), true)
                          },
                          (Some(_), Some(_)) => {
                            return Err("Either query or id".to_string())  
                          } ,
                          (None, None) =>{
                            return Err("Either query or id".to_string())
                          }
                        };
                        return Ok(PlayerCommand::RmPlaylist(SearchArgs 
                            { target: target, is_target_id: is_id, search_type: vec![MediaType::Playlist], is_target_library: true }));
            }

            Commands::Mkpl(command) => {
                return Ok(PlayerCommand::MkPlaylist(command.playlist_name.clone()))
            }

            Commands::Mv(command) => {
                let (track_target, is_track_target_id, is_track_target_current): (Option<String>, bool, bool) = 
                    match (command.track_query.as_ref(), command.track_id.as_ref()) {
                        (Some(q), None) => {
                            if command.current {
                                return Err("Either current track or query".to_string());
                            }
                            (Some(q.to_string()), false, false)
                        },
                        (None, Some(id)) => {
                            if command.current {
                                return Err("Either current track or id".to_string());
                            }
                            (Some(id.to_string()), true, false)
                        },
                        (None, None) => {
                            if !command.current {
                                return Err("Either current track or query/id".to_string());
                            }
                            (None, false, true)
                        },
                        (Some(_), Some(_)) => {
                            return Err("Either query or id or current".to_string())
                        }
                    };

                let (playlist_target, is_playlist_target_id, is_playlist_target_fav): (Option<String>, bool, bool) =
                match (command.playlist_query.as_ref(), command.playlist_id.as_ref()) {
                        (Some(q), None) => {
                            if command.fav {
                                return Err("Either fav playlist or query".to_string());
                            }
                            (Some(q.to_string()), false, false)
                        },
                        (None, Some(id)) => {
                            if command.fav {
                                return Err("Either fav playlist or id".to_string());
                            }
                            (Some(id.to_string()), true, false)
                        },
                        (None, None) => {
                            if !command.fav {
                                return Err("Either fav playlist or query/id".to_string());
                            }
                            (None, false, true)
                        },
                        (Some(_), Some(_)) => {
                            return Err("Either query or id or fav".to_string())
                        }
                };

                let track_sc =  if let Some(target) = track_target {
                    Some(SearchArgs { target, is_target_id: is_track_target_id, search_type: vec![MediaType::Song], is_target_library: false })
                } else {
                    None
                };
                let playlist_sc =  if let Some(target) = playlist_target {
                    Some(SearchArgs { target, is_target_id: is_playlist_target_id, search_type: vec![MediaType::Playlist], is_target_library: true })
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
            }
            
            _=> {
                return Err("Command not supported".to_string())
            }
        }
    }
}