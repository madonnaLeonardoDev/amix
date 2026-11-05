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
    #[command(flatten)]
    pub query: PlayTarget,
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
pub struct RmFromPlaylist {
    //TO IMPLEMENT STRUCTURE -id / Query + -id / Query
}

#[derive(Args, Debug, Clone)]
pub struct RmCommand {
    #[command(subcommand)]
    pub subcommand: Option<RmQueue>,

    #[command(flatten)]
    pub search_args: RmFromPlaylist,
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
        query: PlayTarget,

        #[arg(short, long)]
        id: Option<String>,

        #[arg(short = 't', long = "type")]
        play_type: Option<String>,

        #[arg(short, long)]
        library: bool,
    },

    #[command()]
    Queue{
        #[command(flatten)]
        query: PlayTarget,

        #[arg(short, long)]
        id: Option<String>,

        #[arg(short = 't', long = "type")]
        play_type: Option<String>,

        #[arg(short, long)]
        library: bool,

        #[arg(short, long)]
        next: bool,
    },
    Ls(LsCommand),

    #[command()]
    Shuffle,

    #[command(visible_alias = "auto")]
    Autoplay,

    #[command()]
    Volume{
        value: Option<u32>
    },


}   