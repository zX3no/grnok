use gonk_database::ServerConfig;
use gonk_types::{Index, Song};
use serde::{Deserialize, Serialize};
use static_init::dynamic;
pub use {client::Client, server::Server};

mod client;
mod server;

#[dynamic]
static CONFIG: ServerConfig = ServerConfig::new();

#[derive(Serialize, Deserialize, Debug)]
pub enum Event {
    AddPath(String),
    Add(Vec<u64>),
    PlayIndex(usize),
    Delete(usize),
    ClearQueue,
    TogglePlayback,
    VolumeUp,
    VolumeDown,
    Prev,
    Next,
    SeekTo(f64),
    SeekBy(f64),
    ShutDown,
    Randomize,

    GetElapsed,
    GetPaused,
    GetVolume,
    GetQueue,
    GetAllArtists,
    GetFirstArtist,
    GetArtist(String),
    // GetAlbums(String),
    // GetSongs(String, String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Artist {
    name: String,
    albums: Vec<Album>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Album {
    name: String,
    songs: Vec<Song>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Elapsed(f64),
    Paused(bool),
    Volume(u16),
    Queue(Queue),
    Update(Update),
    //TODO: this is slow as shit
    Artists(Vec<String>),
    Artist(Artist),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Queue {
    pub songs: Index<Song>,
    pub duration: f64,
}

//when the song changes instead of sending the entire queue
//again just send the new selected song
//durations aren't held in songs anymore so send that too.

//maybe just remove this, probably not faster and over complicated
#[derive(Serialize, Deserialize, Debug)]
pub struct Update {
    pub index: Option<usize>,
    pub duration: f64,
}
