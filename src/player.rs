use rodio::Sink;
use rodio::{Decoder, OutputStream};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

pub struct Command {
    playing: bool,
}
impl Command {
    pub fn new() -> Self {
        Self { playing: false }
    }
}
#[derive(Clone)]
pub enum Event {
    Play,
    Pause,
    Stop,
    Volume(f32),
    Empty,
}

pub struct Player {
    pub now_playing: String,
    playing: bool,
    pub event: Arc<RwLock<Event>>,
}
impl Player {
    pub fn new() -> Self {
        Self {
            now_playing: String::new(),
            playing: false,
            event: Arc::new(RwLock::new(Event::Empty)),
        }
    }
    pub fn play(&mut self, path: &PathBuf) {
        self.now_playing = path.file_name().unwrap().to_string_lossy().to_string();
        self.playing = true;

        let path = path.clone();
        let event = self.event.clone();

        thread::spawn(move || {
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let file = BufReader::new(File::open(path).unwrap());
            let source = Decoder::new(file).unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();

            sink.append(source);
            sink.set_volume(0.01);
            loop {
                match *event.read().unwrap() {
                    Event::Play => sink.play(),
                    Event::Pause => sink.pause(),
                    Event::Stop => sink.stop(),
                    Event::Volume(_) => sink.set_volume(0.001),
                    Event::Empty => (),
                }
            }
        });
    }
    pub fn toggle_playback(&mut self) {
        if self.playing {
            *self.event.write().unwrap() = Event::Pause;
        } else {
            *self.event.write().unwrap() = Event::Play;
        }
        self.playing = !self.playing;
    }
    pub fn stop(&mut self) {
        *self.event.write().unwrap() = Event::Stop;
        self.playing = false;
    }
}
