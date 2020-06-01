
extern crate regex;

use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;
use std::iter::{Peekable};
use std::path::{Path};
use crate::rodio::Source;
use super::{SoundProvider, SoundEvent};

lazy_static! {
    static ref METADATA_REGEX: Regex = Regex::new(r"#(?P<key>[A-Z]+) (?P<value>.*)").unwrap();
    static ref SAMPLE_REGEX: Regex = Regex::new(r"#WAV(?P<id>[0-9A-Z]{2}) (?P<filename>.*)").unwrap();
    static ref DATA_REGEX: Regex = Regex::new(r"(?P<measure>\d{3})(?P<channel>\d{2}):(?P<message>([0-9A-Z]{2})+)").unwrap();
}

#[derive(Clone)]
pub struct BMSProvider {
    pub metadata: HashMap<String, String>,
    sounds: HashMap<u32, Sound>,
    data: Vec<BMSDatum>,
    bpm: u16,
    // root: String,
    // file: String,
}

#[derive(Debug, Clone)]
struct BMSDatum {
    measure: u16,
    channel: u8,
    message: Vec<u32>
}

#[derive(Clone)]
enum Sound {
    Path(String),
    Buffer(rodio::source::Buffered<Box<dyn rodio::Source<Item = i16>>>)
}

// impl std::fmt::Debug for Sound {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Path(p) => p,
//             Buffer(b) => 
//         }
//     }
// }

pub fn load(path: &Path) -> BMSProvider {
    let mut bms = BMSProvider {
        metadata: HashMap::new(),
        sounds: HashMap::new(),
        data: Vec::new(),
        bpm: 0,
        // root: filename
        // file: filename.to_string(),
    };
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        match line {
            Ok(line) => handle_line(&mut bms, &line),
            Err(err) => println!("{}", err),
        }
    }
    bms.bpm = match bms.metadata.get("BPM") {
        Some(bpm) => u16::from_str_radix(bpm, 10).unwrap(),
        None => 130,
    };

    bms.data.sort_by(|a, b| a.measure.cmp(&b.measure));
    bms
}

fn handle_line(bms: &mut BMSProvider, line: &str) {
    match handle_data_line(line) {
        Some(datum) => { bms.data.push(datum); },
        None => match handle_sample_line(line) {
            Some((id, file)) => { bms.sounds.insert(id, Sound::Path(file)); },
            None => match handle_metadata_line(line) {
                Some((key, value)) => { bms.metadata.insert(key, value); },
                None => {}
            }
        }
    };
}

fn handle_metadata_line(line: &str) -> Option<(String, String)> {
    match METADATA_REGEX.captures(line) {
        Some(caps) => Some((caps["key"].to_string(), caps["value"].to_string())),
        None => None
    }
}

fn handle_sample_line(line: &str) -> Option<(u32, String)> {
    match SAMPLE_REGEX.captures(line) {
        Some(caps) => Some((
            u32::from_str_radix(&caps["id"], 36).unwrap(),
            caps["filename"].to_string(),
        )),
        None => None
    }
}

fn handle_data_line(line: &str) -> Option<BMSDatum> {
    match DATA_REGEX.captures(line) {
        Some(caps) => Some(BMSDatum {
            measure: u16::from_str_radix(&caps["measure"], 10).unwrap(),
            channel: u8::from_str_radix(&caps["channel"], 10).unwrap(),
            message: parse_message(&caps["message"]),
        }),
        None => None,
    }
}

fn parse_message(message: &str) -> Vec<u32> {
    let mut out = Vec::new();
    let mut chars = message.chars();
    loop {
        let c1 = chars.next();
        let c2 = chars.next();
        if c1.is_none() || c2.is_none() {
            break out
        }
        let c1 = c1.unwrap().to_digit(36).unwrap();
        let c2 = c2.unwrap().to_digit(36).unwrap();
        out.push(c1 * 36 + c2);
    }
}

struct Measures<'a> {
    data: Peekable<Box<dyn Iterator<Item = &'a BMSDatum> + 'a>>,
}

impl<'a> Iterator for Measures<'a> {
    type Item = Vec<&'a BMSDatum>;
    fn next(&mut self) -> Option<Vec<&'a BMSDatum>> {
        let mut result = Vec::new();
        let mut measure: Option<u16> = None;
        loop {
            let peek_value = self.data.peek();
            match peek_value {
                Some(datum) => {
                    match measure {
                        None => {
                            measure = Some(datum.measure)
                        },
                        Some(current_measure) => {
                            if datum.measure != current_measure {
                                break Some(result);
                            }
                        }
                    }
                    result.push(datum);
                    self.data.next();
                },
                None => {
                    break None
                }
            }
        }
    }
}

impl BMSProvider {
    fn get_measures(&self) -> Measures {
        Measures {
            data: (Box::new(self.data.iter()) as Box<dyn Iterator<Item = &BMSDatum>>).peekable(),
        }
    }
    fn get_sound_buffer(&mut self, buffer_id: u32) -> Box<rodio::source::Buffered<Box<dyn rodio::Source<Item = i16>>>> {
        match self.sounds.get(&buffer_id).unwrap() {
            Sound::Buffer(buffer) => Box::new(*buffer),
            Sound::Path(path) => {
                let file = std::fs::File::open(format!("Megalovania/{}", path)).unwrap();
                let decoder = Box::new(rodio::Decoder::new(BufReader::new(file)).unwrap()) as Box<dyn rodio::Source<Item = i16>>;
                let buffer = decoder.buffered();
                self.sounds.insert(buffer_id, Sound::Buffer(buffer)) ;
                match self.sounds.get(&buffer_id).unwrap() {
                    Sound::Buffer(buffer) => Box::new(*buffer),
                    _ => panic!()
                }
            }
        }
    }
}

struct SoundEvents<'a> {
    bms_provider: &'a mut BMSProvider,
    measures: std::iter::Enumerate<Measures<'a>>,
    current_measure: Option<(u16, Vec<(&'a BMSDatum, usize)>)>,
    // (n, a, b) -> a/b of the way through the nth measure
    last_event_time: (u16, usize, usize),
    measure_sample_length: u32,
}

fn next_event_index(measure: &Vec<(&BMSDatum, usize)>) -> Option<usize> {
    let mut current_a = 1usize;
    let mut current_n = 1usize;
    let mut current_i = std::usize::MAX;
    for next in measure.iter().enumerate() {
        let (next_i, (next, next_a)) = next;
        let next_n = next.message.len();
        if *next_a * current_n < current_a * next_n {
            current_a = *next_a;
            current_n = next_n;
            current_i = next_i;
            if current_a == 0 {
                // current_n != 1 => not last
                // measure.len() > i + 1 => not last
                return Some(current_i);
            }
        }
    }
    match current_i {
        std::usize::MAX => None,
        _ => Some(current_i)
    }
}

impl<'a> Iterator for SoundEvents<'a> {
    type Item = SoundEvent<'a>;
    fn next(&mut self) -> Option<SoundEvent<'a>> {
        loop {
            let mut next_datum_i = None;
            if self.current_measure.is_some() {
                next_datum_i = next_event_index(&self.current_measure.as_ref().unwrap().1);
            }
            match next_datum_i {
                None => {
                    // move to next measure
                    match self.measures.next() {
                        Some((n, measure)) => {
                            self.current_measure = Some((n as u16, measure.iter().map(|&a| (a, 0)).collect()));
                        }
                        None => {
                            break None
                        }
                    }
                }
                Some(i) => {
                    let (n, measure) = &mut self.current_measure.as_mut().unwrap();
                    let event_time = (*n, measure[i].1, measure.len());
                    let point = measure[i].0.message[measure[i].1];
                    measure[i].1 += 1;
                    if point != 0 {
                        let mut offset = self.measure_sample_length * (event_time.0 - self.last_event_time.0) as u32;
                        offset += self.measure_sample_length * (event_time.1 * self.last_event_time.2 - event_time.2 * self.last_event_time.1) as u32 / event_time.2 as u32 / self.last_event_time.2 as u32;
                        self.last_event_time = event_time;
                        break Some(SoundEvent {
                            offset: offset,
                            source: self.bms_provider.get_sound_buffer(point),
                        })
                    }
                }
            }
        }
    }
}

impl SoundProvider for BMSProvider {
    fn get_sound_events<'a>(&'a mut self) -> Box<dyn Iterator<Item = SoundEvent> + 'a> {
        Box::new(SoundEvents {
            bms_provider: self,
            measures: self.get_measures().enumerate(),
            current_measure: None,
            last_event_time: (1, 0, 0),
            measure_sample_length: 4 * 60 * 44100 / self.bpm as u32,
        })
    }
}