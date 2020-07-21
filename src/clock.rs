pub struct Clock {
    loop_length_ms: u32,
    current_count_ms: u32,
}

impl Clock {
    pub fn new(loop_length_ms: u32) -> Clock {
        Clock {
            loop_length_ms,
            current_count_ms: 0,
        }
    }

    pub fn increment(&mut self) {
        self.current_count_ms += 1;
        if self.current_count_ms > self.loop_length_ms {
            self.current_count_ms = 0;
        }
    }

    pub fn get_current_count_ms(&self) -> u32 {
        self.current_count_ms
    }
}
