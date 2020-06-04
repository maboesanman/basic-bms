// use core::time::Duration;
use std::time::Duration;
use crate::sound_provider::SoundEvent;
use crate::sound_provider::bms::{BMS, BMSSoundProvider};
use std::iter::Peekable;
use rodio::Source;
use std::path::Path;


pub struct Song<'a> {
    sound_provider: Peekable<Box<dyn Iterator<Item = SoundEvent> + Send + 'a>>,
    current_sources: Vec<Box<dyn Source<Item = f32> + Send + 'a>>,
    current_offset: u64,
    sample_rate: u32,
}

impl<'a> Song<'a> {
    pub fn new(sound_provider: Box<dyn Iterator<Item = SoundEvent> + Send + 'a>) -> Song<'a> {
        Song {
            sound_provider: sound_provider.peekable(),
            current_sources: Vec::new(),
            current_offset: 0,
            sample_rate: 44100,
        }
    }
}

impl<'a> Iterator for Song<'a> {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        loop {
            match self.sound_provider.peek() {
                Some(event) => {
                    if event.offset as u64 * self.sample_rate as u64 / 1000 == self.current_offset {
                        let event = self.sound_provider.next().unwrap();
                        self.current_sources.push(Box::new(event.buffer));
                    } else {
                        break
                    }
                },
                None => break
            }
        }
        let mut i = 0;
        let mut sample_result = 0f32;
        // println!("length: {}", self.current_sources.len());
        while i < self.current_sources.len() {
            let current = self.current_sources.get_mut(i).unwrap().next();
            match current {
                Some(sample) => {
                    sample_result += sample / 5f32;
                    i += 1;
                },
                None => {
                    self.current_sources.remove(i);
                }
            }
        }
        self.current_offset += 1;
        Some(sample_result)
    }
}

impl<'a> rodio::Source for Song<'a> {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}