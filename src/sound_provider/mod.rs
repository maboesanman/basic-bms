pub mod bms;

use rodio::Source;
use rodio::source::Buffered;

pub struct SoundEvent {
    // number of milliseconds to wait before playing this sound (from the beginning)
    pub offset: u32,
    // source to play at the time
    pub buffer: Buffered<Box<dyn Source<Item = f32> + Send>>
}
