#[macro_use]
extern crate lazy_static;
extern crate rodio;

use std::io::BufReader;
use std::fs::File;
// use std::thread;
// use std::time::Duration;

use std::{thread, time};
use sound_provider::bms::{BMS, BMSSoundProvider};
use song::Song;
use rodio::source::{Buffered};
use rodio::{Source, Decoder};

mod sound_provider;
mod song;

fn main() {
    let bms = BMS::new(std::path::Path::new("Megalovania/配置なし.bms"));
    for (key, value) in bms.metadata.iter() {
        println!("{}: {}", key, value);
    }
    let mut bms_sound_provider = BMSSoundProvider::new(bms);
    let file = File::open("Megalovania/rg_1_002.wav").unwrap();
    // let decoder = Decoder::new(BufReader::new(file)).unwrap();
    // let buffer = decoder.buffered();
    // let cloned_buffer = buffer.clone();
    // for event in bms_sound_provider {

    // }
    
    let song = Song::new(Box::new(bms_sound_provider)).buffered();
    // for sample in song.clone() {
    // }
    let device = rodio::default_output_device().unwrap();
    // rodio::play_raw(&device, decoder);
    // rodio::play_once(&device, file);
    rodio::play_raw(&device, song);
    thread::sleep(time::Duration::from_millis(1000000));
}