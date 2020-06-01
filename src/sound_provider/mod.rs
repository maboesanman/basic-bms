pub mod bms;

pub struct SoundEvent<'a> {
    // number of samples since last sound to play this sound
    offset: u32,
    // source to play at the time
    source: Box<dyn rodio::Source<Item = i16> + 'a>
}

pub trait SoundProvider {
    fn get_sound_events<'a>(&'a mut self) -> Box<dyn Iterator<Item = SoundEvent> + 'a>;
}
