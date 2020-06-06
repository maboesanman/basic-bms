#[macro_use]
extern crate lazy_static;
extern crate rodio;

use sound_provider::bms::{BMS, BMSSoundProvider};
use song::Song;
use rodio::{Sink};
use rodio::source::Source;
use std::env::args;

mod sound_provider;
mod song;

fn main() {
    let path = args().nth(1);
    if path.is_none() {
        println!("no path given");
    }
    let path = path.unwrap();
    let bms = BMS::new(std::path::Path::new(&path));
    for (key, value) in bms.metadata.iter() {
        println!("{}: {}", key, value);
    }
    let bms_sound_provider = BMSSoundProvider::new(bms);
    let song = Song::new(Box::new(bms_sound_provider)).buffered();
    println!("done loading");
    let device = rodio::default_output_device().unwrap();
    let sink = Sink::new(&device);
    sink.append(song);
    sink.sleep_until_end();
}