#[macro_use]
extern crate lazy_static;
extern crate rodio;

// use std::io::BufReader;
// use std::thread;
// use std::time::Duration;
mod sound_provider;
mod song;

fn main() {
    let _bms = sound_provider::bms::load(std::path::Path::new("Megalovania/配置なし.bms"));
    // bms.get_measures();
    // for (key, value) in s.metadata {
    //     println!("{}: {}", key, value);
    // }
    // let device = rodio::default_output_device().unwrap();
    // let device = rodio::devices().unwrap().nth(4).unwrap();
    // println!("{}", rodio::devices().unwrap().count());


    // let file = std::fs::File::open("Megalovania/base_003.wav").unwrap();
    // let beep1 = rodio::play_once(&device, BufReader::new(file)).unwrap();
    // beep1.set_volume(0.2);
    // println!("Started beep1");

    // thread::sleep(Duration::from_millis(1500));

    // let file = std::fs::File::open("Megalovania/base_003.wav").unwrap();
    // rodio::play_once(&device, BufReader::new(file))
    //     .unwrap()
    //     .detach();
    // println!("Started beep2");

    // thread::sleep(Duration::from_millis(1500));
    // let file = std::fs::File::open("Megalovania/base_003.wav").unwrap();
    // let beep3 = rodio::play_once(&device, file).unwrap();
    // println!("Started beep3");

    // thread::sleep(Duration::from_millis(1500));
    // drop(beep1);
    // println!("Stopped beep1");

    // thread::sleep(Duration::from_millis(1500));
    // drop(beep3);
    // println!("Stopped beep3");

    // thread::sleep(Duration::from_millis(1500));
}