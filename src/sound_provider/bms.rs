
extern crate regex;

use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;
use std::iter::{Peekable};
use std::path::{Path};
use rodio::source::{Buffered};
use rodio::{Source, Decoder};
use super::{SoundEvent};

lazy_static! {
    static ref METADATA_REGEX: Regex = Regex::new(r"#(?P<key>[A-Z]+) (?P<value>.*)").unwrap();
    static ref SAMPLE_REGEX: Regex = Regex::new(r"#WAV(?P<id>[0-9A-Z]{2}) (?P<filename>.*)").unwrap();
    static ref DATA_REGEX: Regex = Regex::new(r"(?P<measure>\d{3})(?P<channel>\d{2}):(?P<message>([0-9A-Z]{2})+)").unwrap();
}

#[derive(Clone)]
pub struct BMS {
    pub metadata: HashMap<String, String>,
    sounds: HashMap<u32, String>,
    data: Vec<BMSDatum>,
    bpm: u16,
    // root: String,
    // file: String,
    
}

#[derive(Clone)]
struct BMSDatum {
    measure: u16,
    _channel: u8,
    message: Vec<u32>
}

impl BMS {
    pub fn new(path: &Path) -> BMS {
        let mut bms = BMS {
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
                Ok(line) => bms.handle_line(&line),
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

    fn handle_line(&mut self, line: &'_ str) {
        match BMS::handle_data_line(line) {
            Some(datum) => { self.data.push(datum); },
            None => match BMS::handle_sample_line(line) {
                Some((id, path)) => {
                    self.sounds.insert(id, path);
                },
                None => if let Some((key, value)) = BMS::handle_metadata_line(line) {
                    self.metadata.insert(key, value);
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
                _channel: u8::from_str_radix(&caps["channel"], 10).unwrap(),
                message: BMS::parse_message(&caps["message"]),
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
}

pub struct BMSSoundProvider {
    bms: BMS,
    sources: HashMap<String, LazySource>,
    measures: std::iter::Enumerate<Measures>,
    current_measure: Option<(usize, Vec<(BMSDatum, usize)>)>,
    // (n, a, b) -> a/b of the way through the nth measure
    // last_event_time: (u16, usize, usize),
    // in milliseconds
    measure_length: u32,
}

enum LazySource {
    Path(String),
    Buffer(Buffered<Box<dyn Source<Item = f32> + Send>>),
}

impl<'a> BMSSoundProvider {
    pub fn new(bms: BMS) -> BMSSoundProvider {
        let mut sources = HashMap::new();
        for (_, path) in bms.sounds.iter() {
            let path = path.clone();
            sources.insert(path.clone(), LazySource::Path(path.clone()));
        }

        BMSSoundProvider {
            bms: bms.clone(),
            sources,
            measures: Measures::new(Box::new(bms.data.into_iter())).enumerate(),
            current_measure: None,
            // last_event_time: (0, 0, 1),
            measure_length: 4 * 60 * 1000 / bms.bpm as u32,
        }
    }
    
    fn get_source(&mut self, id: u32) -> Option<Buffered<Box<dyn Source<Item = f32> + Send>>> {
        match self.bms.sounds.get(&id) {
            Some(path) => match self.sources.get(path) {
                Some(lazy_source) => match lazy_source {
                    LazySource::Path(path) => {
                        let file = std::fs::File::open(format!("Megalovania/{}", path));
                        match file {
                            Ok(file) => {
                                let decoder = Box::new(Decoder::new(BufReader::new(file)).unwrap().convert_samples()) as Box<dyn Source<Item = f32> + Send>;
                                let uniform = Box::new(rodio::source::UniformSourceIterator::new(decoder, 1, 44100)) as Box<dyn Source<Item = f32> + Send>;
                                let buffer = uniform.buffered();
                                let cloned_buffer = buffer.clone();
                                self.sources.insert(path.clone(), LazySource::Buffer(buffer));
                                Some(cloned_buffer)
                            },
                            Err(err) => {
                                println!("{}", err);
                                None
                            }
                        }
                    },
                    LazySource::Buffer(buffer) => Some(buffer.clone())
                }
                None => None,
            }
            None => None,
        }
    }
    fn next_event_index(&self) -> Option<usize> {
        let mut current_a = 1usize;
        let mut current_n = 1usize;
        let mut current_i = std::usize::MAX;
        match self.current_measure.as_ref() {
            None => return None,
            Some(measure) => {
                for next in measure.1.iter().enumerate() {
                    let (next_i, (next, next_a)) = next;
                    let next_n = next.message.len();
                    if *next_a * current_n < current_a * next_n {
                        current_a = *next_a;
                        current_n = next_n;
                        current_i = next_i;
                        if current_a == 0 {
                            return Some(current_i);
                        }
                    }
                }
            }
        }
        
        match current_i {
            std::usize::MAX => None,
            _ => Some(current_i)
        }
    }
}

impl Iterator for BMSSoundProvider {
    type Item = SoundEvent;
    fn next(&mut self) -> Option<SoundEvent> {
        loop {
            // advance measure if missing. complete iteration if no more measures
            if self.current_measure.is_none() {
                match self.measures.next() {
                    Some(next_measure) => {
                        let measure_number = next_measure.0;
                        let status = next_measure.1.into_iter().map(|datum| (datum, 0)).collect();
                        self.current_measure = Some((measure_number, status));
                        continue
                    },
                    None => break None
                }
            }

            // get the index of the message to advance through
            let next_datum_i = self.next_event_index();

            // if there are no more events in this measure, we will load the next measure next loop
            if next_datum_i.is_none() {
                self.current_measure = None;
                continue
            }
            let next_datum_i = next_datum_i.unwrap();

            // borrow a mutable reference to the current measure
            let current_measure = self.current_measure.as_mut().unwrap();

            
            let n = current_measure.0;
            let (bms_datum, a) = current_measure.1.get_mut(next_datum_i).unwrap();
            let b = bms_datum.message.len();
            let point = *bms_datum.message.get(a.clone()).unwrap();
            let m = self.measure_length as usize;
            let offset = ((m * n * b + m * *a) / b) as u32;
            *a += 1;
            if point != 0u32 {
                match self.get_source(point) {
                    Some(source) => {
                        break Some(SoundEvent {
                            offset,
                            buffer: source,
                        })
                    }
                    None => {}
                }
            }
        }
    }
}

struct Measures {
    data: Peekable<Box<dyn Iterator<Item = BMSDatum> + Send>>,
}

impl Measures {
    fn new(iter: Box<dyn Iterator<Item = BMSDatum> + Send>) -> Measures {
        Measures {
            data: iter.peekable()
        }
    }
}

impl Iterator for Measures {
    type Item = Vec<BMSDatum>;
    fn next(&mut self) -> Option<Vec<BMSDatum>> {
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
                    result.push(self.data.next().unwrap());
                },
                None => {
                    break None
                }
            }
        }
    }
}



