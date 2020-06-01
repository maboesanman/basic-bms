use core::time::Duration;
use crate::sound_provider::SoundProvider;

pub struct Song {
    sound_provider: Box<dyn SoundProvider>,
    sample_rate: u32,
}

impl Iterator for Song {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        None
    }
}

impl rodio::Source for Song {
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