use embedded_midi::MidiMessage;
use heapless::{consts::*, FnvIndexMap};
pub struct LoopBuffer {
    buffer: FnvIndexMap<u32, MidiMessage, U512>,
}

impl LoopBuffer {
    pub fn new() -> LoopBuffer {
        LoopBuffer {
            buffer: FnvIndexMap::new(),
        }
    }

    pub fn insert_message(&mut self, time_ms: u32, message: MidiMessage) {
        self.buffer.insert(time_ms, message).unwrap();
    }

    pub fn get_message(&self, time_ms: u32) -> Option<&MidiMessage> {
        self.buffer.get(&time_ms)
    }
}
