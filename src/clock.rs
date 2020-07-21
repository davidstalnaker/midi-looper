pub struct Clock {
    pub count: u32,
}

impl Clock {
    pub fn new() -> Clock {
        Clock { count: 0 }
    }

    pub fn increment(&mut self) {
        self.count += 1;
    }
}
