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
mod tests {
    use super::*;

    #[test]
    fn test_press_release() {
        let mut kb = Keyboard::new();
        assert!(!kb.is_pressed(0xA));
        kb.press(0xA);
        assert!(kb.is_pressed(0xA));
        kb.release(0xA);
        assert!(!kb.is_pressed(0xA));
    }

    #[test]
    fn test_any_pressed() {
        let mut kb = Keyboard::new();
        assert_eq!(kb.any_pressed(), None);
        kb.press(0x5);
        assert_eq!(kb.any_pressed(), Some(0x5));
    }

    #[test]
    fn test_out_of_range_ignored() {
        let mut kb = Keyboard::new();
        kb.press(0xFF);
        assert!(!kb.is_pressed(0xFF));
    }
}
