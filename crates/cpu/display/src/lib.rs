pub struct Display {
    buffer: Vec<u8>,
    width: u16,
    height: u16,
}

impl Display {
    pub fn new(width: u16, height: u16) -> Self {
        Display {
            buffer: vec![0u8; (width as usize) * (height as usize)],
            width,
            height,
        }
    }

    pub fn width(&self) -> u16 { self.width }
    pub fn height(&self) -> u16 { self.height }

    pub fn clear(&mut self) {
        self.buffer.fill(0);
    }

    pub fn set_pixel(&mut self, x: u8, y: u8, value: u8) {
        if (x as u16) < self.width && (y as u16) < self.height {
            let idx = (y as usize) * (self.width as usize) + (x as usize);
            self.buffer[idx] = value;
        }
    }

    pub fn get_pixel(&self, x: u8, y: u8) -> u8 {
        if (x as u16) < self.width && (y as u16) < self.height {
            let idx = (y as usize) * (self.width as usize) + (x as usize);
            self.buffer[idx]
        } else {
            0
        }
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
    fn test_new_default() {
        let d = Display::new(64, 32);
        assert_eq!(d.width(), 64);
        assert_eq!(d.height(), 32);
        assert_eq!(d.buffer_len(), 2048);
    }

    #[test]
    fn test_clear() {
        let mut d = Display::new(64, 32);
        d.set_pixel(0, 0, 1);
        d.clear();
        assert_eq!(d.get_pixel(0, 0), 0);
    }

    #[test]
    fn test_set_get_pixel() {
        let mut d = Display::new(64, 32);
        d.set_pixel(10, 5, 1);
        assert_eq!(d.get_pixel(10, 5), 1);
        assert_eq!(d.get_pixel(11, 5), 0);
    }

    #[test]
    fn test_out_of_bounds() {
        let mut d = Display::new(64, 32);
        d.set_pixel(100, 100, 1);
        assert_eq!(d.get_pixel(100, 100), 0);
    }

    #[test]
    fn test_get_buffer() {
        let d = Display::new(64, 32);
        assert_eq!(d.get_buffer().len(), 2048);
    }
}
