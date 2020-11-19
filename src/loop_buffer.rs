use heapless::{consts::*, FnvIndexMap};
use midi_port::MidiMessage;
pub struct LoopBuffer {
    buffer: FnvIndexMap<u32, MidiMessage, U64>,
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
