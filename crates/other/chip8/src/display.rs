pub struct Display {
    pub buffer: Vec<u8>,
    pub width: u16,
    pub height: u16,
}

impl Display {
    pub fn new() -> Self {
        Display::with_size(64, 32)
    }

    pub fn with_size(width: u16, height: u16) -> Self {
        Display {
            buffer: vec![0u8; (width as usize) * (height as usize)],
            width,
            height,
        }
    }

    pub fn clear(&mut self) {
        for pixel in &mut self.buffer {
            *pixel = 0;
        }
    }

    pub fn draw(&mut self, x: u8, y: u8, sprite: &[u8]) -> bool {
        let mut collision = false;
        let w = self.width;
        let h = self.height;

        for (row, &byte) in sprite.iter().enumerate() {
            if row as u16 >= h {
                break;
            }
            let py = (y.wrapping_add(row as u8)) % (h as u8);
            let base = (py as usize) * (w as usize);

            for bit in 0..8 {
                if byte & (0x80 >> bit) == 0 {
                    continue;
                }
                let px = (x.wrapping_add(bit)) % (w as u8);
                let idx = base + (px as usize);

                if self.buffer[idx] != 0 {
                    collision = true;
                }
                self.buffer[idx] ^= 1;
            }
        }

        collision
    }

    pub fn get_pixel(&self, x: u8, y: u8) -> bool {
        if x >= self.width as u8 || y >= self.height as u8 {
            return false;
        }
        let idx = (y as usize) * (self.width as usize) + (x as usize);
        self.buffer[idx] != 0
    }

    pub fn get_buffer(&self) -> Vec<u8> {
        self.buffer.clone()
    }

    pub fn buffer_ptr(&self) -> *const u8 {
        self.buffer.as_ptr()
    }

    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clear() {
        let mut d = Display::new();
        d.buffer[0] = 1;
        d.clear();
        assert_eq!(d.buffer[0], 0);
    }

    #[test]
    fn test_draw_no_collision() {
        let mut d = Display::new();
        let sprite = [0xF0, 0x90, 0x90, 0x90, 0xF0];
        let coll = d.draw(0, 0, &sprite);
        assert!(!coll);
        assert!(d.get_pixel(0, 0));
        assert!(!d.get_pixel(4, 0));
    }

    #[test]
    fn test_draw_collision() {
        let mut d = Display::new();
        let sprite = [0xFF];
        d.draw(0, 0, &sprite);
        let coll = d.draw(0, 0, &sprite);
        assert!(coll);
        assert!(!d.get_pixel(0, 0));
    }

    #[test]
    fn test_with_size() {
        let d = Display::with_size(128, 64);
        assert_eq!(d.buffer.len(), 8192);
    }

    #[test]
    fn test_get_buffer_size() {
        let d = Display::new();
        assert_eq!(d.get_buffer().len(), 2048);
    }

    #[test]
    fn test_draw_on_wider_display() {
        let mut d = Display::with_size(128, 64);
        let sprite = [0xFF];
        let coll = d.draw(100, 30, &sprite);
        assert!(!coll);
        assert!(d.get_pixel(100, 30));
        assert!(d.get_pixel(107, 30));
    }
}
