use cpu_bus::Bus;

/// Universal flat memory — no platform-specific features.
pub struct Memory {
    data: Vec<u8>,
    size: usize,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        Memory { data: vec![0; size], size }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.data[(addr as usize) % self.size]
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        self.data[(addr as usize) % self.size] = value;
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
        let lo = self.read(addr) as u16;
        let hi = self.read(addr.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }

    pub fn write_u16(&mut self, addr: u16, value: u16) {
        self.write(addr, (value & 0xFF) as u8);
        self.write(addr.wrapping_add(1), ((value >> 8) & 0xFF) as u8);
    }

    pub fn load(&mut self, data: &[u8], offset: u16) {
        for (i, &byte) in data.iter().enumerate() {
            let addr = offset.wrapping_add(i as u16);
            self.write(addr, byte);
        }
    }

    pub fn as_slice(&self) -> &[u8] { &self.data }
    pub fn as_mut_slice(&mut self) -> &mut [u8] { &mut self.data }
    pub fn len(&self) -> usize { self.size }
    pub fn clear(&mut self) { self.data.fill(0); }
}

impl Bus for Memory {
    fn read(&mut self, addr: u16) -> u8 { Memory::read(self, addr) }
    fn write(&mut self, addr: u16, value: u8) { Memory::write(self, addr, value) }
    fn read_u16(&mut self, addr: u16) -> u16 { Memory::read_u16(self, addr) }
    fn write_u16(&mut self, addr: u16, value: u16) { Memory::write_u16(self, addr, value) }
}

/// Trait for types that can be read/written like memory.
pub trait MemoryAccess {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);
    fn read_u16(&self, addr: u16) -> u16;
    fn write_u16(&mut self, addr: u16, value: u16);
}

impl MemoryAccess for Memory {
    fn read(&self, addr: u16) -> u8 { Memory::read(self, addr) }
    fn write(&mut self, addr: u16, value: u8) { Memory::write(self, addr, value) }
    fn read_u16(&self, addr: u16) -> u16 { Memory::read_u16(self, addr) }
    fn write_u16(&mut self, addr: u16, value: u16) { Memory::write_u16(self, addr, value) }
}

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
