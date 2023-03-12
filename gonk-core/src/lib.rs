//! The database is split into two parts:
//! The virtual and the physical.
//!
//! The physical database is a file on disk that stores song information.
//! This information includes the artist, album, title, disc number, track number, path and replay gain.
//!
//! The virtual database is an in memory key value store.
//! It is used for quering artists, albums and songs.
//!
//! `Lazy` is my implementation of lazy statics.
//!
//! `Index` is a wrapper over a `Vec<T>` plus an index. Kind of like a circular buffer but the data is usually constant.
//! It's useful for moving up and down the selection of a UI element.
use std::{
    env,
    error::Error,
    ffi::OsString,
    fs,
    mem::size_of,
    ops::Range,
    path::{Path, PathBuf},
    str::from_utf8,
};

pub use crate::{
    db::{Album, Artist, Song},
    // old_db::{Album, Artist, Song},
    playlist::Playlist,
};
pub use flac_decoder::*;
pub use index::*;
pub use lazy::*;

pub mod db;
pub mod flac_decoder;
pub mod index;
pub mod lazy;
pub mod log;
// pub mod old_db;
pub mod playlist;
pub mod profiler;
pub mod settings;
pub mod strsim;
pub mod vdb;

pub fn gonk_path() -> PathBuf {
    let gonk = if cfg!(windows) {
        PathBuf::from(&env::var("APPDATA").unwrap())
    } else {
        PathBuf::from(&env::var("HOME").unwrap()).join(".config")
    }
    .join("gonk");

    if !gonk.exists() {
        fs::create_dir_all(&gonk).unwrap();
    }

    gonk
}

pub fn settings_path() -> PathBuf {
    let mut path = gonk_path();
    path.push("settings.db");
    path
}

pub fn database_path() -> PathBuf {
    let gonk = gonk_path();

    //Backwards compatibility for older versions of gonk
    let old_db = gonk.join("gonk_new.db");
    let db = gonk.join("gonk.db");

    if old_db.exists() {
        fs::rename(old_db, &db).unwrap();
    }

    db
}
