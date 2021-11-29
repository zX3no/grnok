#![cfg_attr(test, deny(missing_docs))]
pub use cpal::{
    self, traits::DeviceTrait, Device, Devices, DevicesError, InputDevices, OutputDevices,
    SupportedStreamConfig,
};

mod conversions;
mod sink;
mod stream;

pub mod buffer;
pub mod decoder;
pub mod dynamic_mixer;
pub mod queue;
pub mod source;

pub use crate::conversions::Sample;
pub use crate::decoder::Decoder;
pub use crate::sink::Sink;
pub use crate::source::Source;
pub use crate::stream::{OutputStream, OutputStreamHandle, PlayError, StreamError};

use std::path::Path;
use std::time::Duration;
use std::{fs::File, io::BufReader};

pub struct Player {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Sink,
    total_duration: Option<Duration>,
    volume: f32,
}
impl Player {
    pub fn new() -> Self {
        let (_stream, handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&handle).unwrap();
        let volume = 0.01;
        sink.set_volume(volume);

        Self {
            _stream,
            handle,
            sink,
            total_duration: None,
            volume,
        }
    }
    pub fn set_volume(&self, v: f32) {
        //TODO: clamping
        self.sink.set_volume(self.volume + v);
    }
    pub fn sleep_until_end(&self) {
        self.sink.sleep_until_end();
    }
    pub fn play(&mut self, path: &Path) {
        self.stop();
        let file = File::open(path).unwrap();
        let decoder = Decoder::new(BufReader::new(file)).unwrap();
        self.total_duration = decoder.total_duration();
        self.sink.append(decoder);
    }
    pub fn stop(&mut self) {
        // self.sink.stop();
        self.sink.drop();
        self.sink = Sink::try_new(&self.handle).unwrap();
        self.sink.set_volume(self.volume);
    }
    pub fn elapsed(&self) -> Duration {
        //TODO: change to option duration
        self.sink.elapsed()
    }
    pub fn duration(&self) -> Option<Duration> {
        self.total_duration
    }
    pub fn toggle_playback(&self) {
        self.sink.toggle_playback();
    }
    pub fn seek(&self, amount: Duration) {
        self.sink.seek(amount);
    }
    pub fn seeker(&self) -> f64 {
        if let Some(duration) = self.duration() {
            let elapsed = self.elapsed();
            elapsed.as_secs_f64() / duration.as_secs_f64()
        } else {
            0.0
        }
    }
    pub fn is_playing(&self) -> bool {
        self.elapsed() != Duration::from_secs(0)
    }
}
