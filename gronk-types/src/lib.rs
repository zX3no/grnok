use std::{
    fs::File,
    path::{Path, PathBuf},
};
use symphonia::core::{
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::{MetadataOptions, MetadataRevision, StandardTagKey},
    probe::Hint,
};

#[derive(Debug, Clone)]
pub struct Song {
    pub number: u16,
    pub disc: u16,
    pub name: String,
    pub album: String,
    pub artist: String,
    pub path: PathBuf,
}
impl Song {
    pub fn from(path: &Path) -> Song {
        let mut hint = Hint::new();
        let ext = path.extension().unwrap().to_str().unwrap();
        hint.with_extension(&ext);

        let file = Box::new(File::open(path).unwrap());

        // Create the media source stream using the boxed media source from above.
        let mss = MediaSourceStream::new(file, Default::default());

        // Use the default options for metadata and format readers.
        let format_opts: FormatOptions = Default::default();
        let metadata_opts: MetadataOptions = Default::default();

        let mut probe = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .unwrap();

        let mut song = Song::default();
        song.path = path.to_path_buf();

        let mut get_songs = |metadata: &MetadataRevision| {
            for tag in metadata.tags() {
                if let Some(std_key) = tag.std_key {
                    match std_key {
                        StandardTagKey::AlbumArtist => song.artist = tag.value.to_string(),
                        StandardTagKey::Artist if song.artist.is_empty() => {
                            song.artist = tag.value.to_string()
                        }
                        StandardTagKey::Album => song.album = tag.value.to_string(),
                        StandardTagKey::TrackTitle => song.name = tag.value.to_string(),
                        StandardTagKey::TrackNumber => {
                            song.number = tag.value.to_string().parse::<u16>().unwrap_or(1)
                        }
                        StandardTagKey::DiscNumber => {
                            song.disc = tag.value.to_string().parse::<u16>().unwrap_or(1)
                        }
                        _ => (),
                    }
                }
            }
        };

        if let Some(metadata) = probe.metadata.get() {
            get_songs(metadata.current().unwrap());
        } else if let Some(metadata) = probe.format.metadata().current() {
            get_songs(metadata);
        }

        if song.artist.is_empty() {
            //TODO: unknown artist
            panic!("no artist???");
        }
        song
    }
}
impl PartialEq for Song {
    fn eq(&self, other: &Self) -> bool {
        self.number == other.number
            && self.disc == other.disc
            && self.name == other.name
            && self.album == other.album
            && self.artist == other.artist
            && self.path == other.path
    }
}
impl Default for Song {
    fn default() -> Self {
        Self {
            number: Default::default(),
            disc: Default::default(),
            name: Default::default(),
            album: Default::default(),
            artist: Default::default(),
            path: Default::default(),
        }
    }
}
