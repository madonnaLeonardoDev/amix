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
    #[arg(short, long)]
    pub result: Option<u32>
}

#[derive(Args, Debug, Clone)]
pub struct LsSearchArgs {
    pub query: String,
    #[arg(short, long)]
    pub result: Option<u32>,
    #[arg(short, long)]
    pub library: Option<u32>,
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

#[derive(Subcommand, Debug, Clone)]
pub enum RmQueue {
    Queue{
        #[arg(short, long)]
        index: Option<u32>
    }
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
    // --- Track Slot ---
    #[arg(
        long = "track-id",
        value_name = "TRACK_ID",
        group = "track_target",
        help = "Track ID"
    )]
    pub track_id: Option<String>,

    #[arg(
        value_name = "TRACK_QUERY",
        group = "track_target",
        help = "Track name or query"
    )]
    pub track_query: Option<String>,

    // --- Playlist Slot ---
    #[arg(
        long = "playlist-id",
        value_name = "PLAYLIST_ID",
        group = "playlist_target",
        help = "Playlist ID"
    )]
    pub playlist_id: Option<String>,

    #[arg(
        value_name = "PLAYLIST_QUERY",
        group = "playlist_target",
        help = "Playlist name or query"
    )]
    pub playlist_query: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct RmCommand {
    #[command(subcommand)]
    pub subcommand: Option<RmQueue>,

    #[command(flatten)]
    pub search_args: RmFromPlaylist,
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
    pub track_query: Option<String>,
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
    Search(SearchArgs),
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
                    let t = Vec::new();
                    t.push(MediaType::Song);
                    t.push(MediaType::Radio);
                    t.push(MediaType::Album);
                    t.push(MediaType::Playlist);
                    t
                };

                let t = target.unwrap();

                let target:(String, bool)= if let Some(id) = t.id {
                    (id, true)
                } else {
                    (t.query.unwrap(), false)
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
                    let t = Vec::new();
                    t.push(MediaType::Song);
                    t.push(MediaType::Radio);
                    t.push(MediaType::Album);
                    t.push(MediaType::Playlist);
                    t
                };

                let t = target.unwrap();

                let target:(String, bool)= if let Some(id) = t.id {
                    (id, true)
                } else {
                    (t.query.unwrap(), false)
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
                if command.subcommand.is_some(){
                    //LS SUBCOMMANDS MATCHING                    
                } else {
                    //SEARCH
                }

                Ok(())
            }
        }
    }
}