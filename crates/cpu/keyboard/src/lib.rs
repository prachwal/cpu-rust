pub struct Keyboard {
    keys: [bool; 16],
}

impl Keyboard {
    pub fn new() -> Self {
        Keyboard { keys: [false; 16] }
    }

    pub fn press(&mut self, key: u8) {
        if let Some(k) = self.keys.get_mut(key as usize) {
            *k = true;
        }
    }

    pub fn release(&mut self, key: u8) {
        if let Some(k) = self.keys.get_mut(key as usize) {
            *k = false;
        }
    }

    pub fn is_pressed(&self, key: u8) -> bool {
        self.keys.get(key as usize).copied().unwrap_or(false)
    }

    pub fn any_pressed(&self) -> Option<u8> {
        self.keys.iter().position(|&k| k).map(|i| i as u8)
    }
}

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
