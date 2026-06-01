const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70,
    0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0,
    0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0,
    0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40,
    0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0,
    0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0,
    0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0,
    0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
];

const DEFAULT_SIZE: usize = 4096;

pub struct Memory {
    data: Vec<u8>,
    pub size: usize,
}

impl Memory {
    pub fn new() -> Self {
        Memory::with_size_and_font(DEFAULT_SIZE, 0x050)
    }

    pub fn with_font_offset(font_offset: u16) -> Self {
        Memory::with_size_and_font(DEFAULT_SIZE, font_offset)
    }

    pub fn with_size_and_font(size: usize, font_offset: u16) -> Self {
        let mut data = vec![0u8; size];
        let start = font_offset as usize;
        let end = (start + FONT.len()).min(size);
        data[start..end].copy_from_slice(&FONT[..end - start]);
        Memory { data, size }
    }

    pub fn load_rom(&mut self, rom: &[u8], offset: u16) {
        let start = offset as usize;
        let end = (start + rom.len()).min(self.size);
        self.data[start..end].copy_from_slice(&rom[..end - start]);
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.data[addr as usize % self.size]
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        self.data[addr as usize % self.size] = val;
    }

    pub fn read_slice(&self, addr: u16, len: u16) -> &[u8] {
        let start = addr as usize % self.size;
        let end = (start + len as usize).min(self.size);
        &self.data[start..end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_loaded() {
        let m = Memory::new();
        assert_eq!(m.read(0x050), 0xF0);
        assert_eq!(m.read(0x09F), 0x80);
    }

    #[test]
    fn test_custom_font_offset() {
        let m = Memory::with_font_offset(0x100);
        assert_eq!(m.read(0x100), 0xF0);
    }

    #[test]
    fn test_read_write() {
        let mut m = Memory::new();
        m.write(0x200, 0xAB);
        assert_eq!(m.read(0x200), 0xAB);
    }

    #[test]
    fn test_load_rom() {
        let mut m = Memory::new();
        let rom = vec![0x00, 0xE0, 0x12, 0x34];
        m.load_rom(&rom, 0x200);
        assert_eq!(m.read(0x200), 0x00);
        assert_eq!(m.read(0x203), 0x34);
    }

    #[test]
    fn test_wraparound() {
        let m = Memory::with_size_and_font(256, 0x050);
        // addr 0x100 wraps to 0 in a 256-byte memory
        m.read(0x100);
    }
}
