use cpu_bus::Bus;

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

const MEM_SIZE: usize = 4096;

pub struct Memory {
    inner: cpu_memory::Memory,
}

impl Memory {
    pub fn new() -> Self {
        Memory::with_font_offset(0x050)
    }

    pub fn with_font_offset(font_offset: u16) -> Self {
        let mut inner = cpu_memory::Memory::new(MEM_SIZE);
        inner.load(&FONT, font_offset);
        Memory { inner }
    }

    pub fn load_rom(&mut self, rom: &[u8], offset: u16) {
        self.inner.load(rom, offset);
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.inner.read(addr)
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        self.inner.write(addr, val);
    }

    pub fn read_slice(&self, addr: u16, len: u16) -> &[u8] {
        let data = self.inner.as_slice();
        let start = addr as usize % MEM_SIZE;
        let end = (start + len as usize).min(MEM_SIZE);
        &data[start..end]
    }
}

impl Bus for Memory {
    fn read(&mut self, addr: u16) -> u8 {
        self.inner.read(addr)
    }
    fn write(&mut self, addr: u16, value: u8) {
        self.inner.write(addr, value);
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
        let m = Memory::with_font_offset(0x050);
        m.read(0x1000);
    }
}
