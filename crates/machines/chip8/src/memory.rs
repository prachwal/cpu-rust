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
#[path = "tests/memory.rs"]
mod tests;
