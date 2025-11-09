// Random number generator (LFSR - Linear Feedback Shift Register)
// Tiny deterministic RNG - same sequence each run unless you change seed

#[derive(Clone, Copy)]
pub struct Lfsr {
    s: u16,
}

impl Lfsr {
    pub fn new(seed: u16) -> Self {
        Self { s: if seed == 0 { 0xACE1 } else { seed } }
    }
    
    pub fn next(&mut self) -> u16 {
        let lsb = self.s & 1;
        self.s >>= 1;
        if lsb != 0 {
            self.s ^= 0xB400;
        }
        self.s
    }
    
    pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
        let span = (hi - lo + 1) as u16;
        lo + (self.next() % span) as i32
    }
}

