use mos6502_bus::Bus;
use mos6502_config::MachineConfig;

#[derive(Debug, Clone)]
pub struct MemoryBank {
    pub data: Vec<u8>,
    pub size: usize,
}

impl MemoryBank {
    pub fn new(size: usize) -> Self {
        MemoryBank {
            data: vec![0; size],
            size,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        let wrapped_addr = addr as usize % self.size;
        self.data[wrapped_addr]
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        let wrapped_addr = addr as usize % self.size;
        self.data[wrapped_addr] = value;
    }
}

#[derive(Debug, Clone)]
struct Apple1Pia {
    keyboard_data: u8,
    keyboard_ready: bool,
    keyboard_control: u8,
    display_control: u8,
    display_configured: bool,
    display_output: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Memory {
    pub banks: Vec<MemoryBank>,
    pub current_bank: usize,
    pub size: usize,
    config: MachineConfig,
    apple1_pia: Option<Apple1Pia>,
}

impl Memory {
    pub fn new(config: &MachineConfig) -> Self {
        if config.memory.bank_switching && config.memory.num_banks > 1 {
            let bank_size = config.memory.size / config.memory.num_banks;
            let banks = (0..config.memory.num_banks)
                .map(|_| MemoryBank::new(bank_size))
                .collect();

            Memory {
                banks,
                current_bank: 0,
                size: config.memory.size,
                config: config.clone(),
                apple1_pia: Self::apple1_pia_for(config),
            }
        } else {
            Memory {
                banks: vec![MemoryBank::new(config.memory.size)],
                current_bank: 0,
                size: config.memory.size,
                config: config.clone(),
                apple1_pia: Self::apple1_pia_for(config),
            }
        }
    }

    fn apple1_pia_for(config: &MachineConfig) -> Option<Apple1Pia> {
        if config.label != "Apple 1" {
            return None;
        }

        Some(Apple1Pia {
            keyboard_data: 0,
            keyboard_ready: false,
            keyboard_control: 0,
            display_control: 0,
            display_configured: false,
            display_output: Vec::new(),
        })
    }

    pub fn with_defaults() -> Self {
        Self::new(&MachineConfig::default())
    }

    pub fn read(&self, addr: u16) -> u8 {
        if let Some(pia) = &self.apple1_pia {
            match addr {
                0xD010 => return pia.keyboard_data,
                0xD011 => {
                    let ready = if pia.keyboard_ready { 0x80 } else { 0x00 };
                    return (pia.keyboard_control & 0x7F) | ready;
                }
                0xD012 => return 0x00,
                0xD013 => return pia.display_control,
                _ => {}
            }
        }

        if self.config.memory.bank_switching && self.banks.len() > 1 {
            self.banks[self.current_bank].read(addr)
        } else {
            self.banks[0].read(addr)
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        if let Some(pia) = &mut self.apple1_pia {
            match addr {
                0xD010 => {
                    pia.keyboard_data = value;
                    pia.keyboard_ready = true;
                    return;
                }
                0xD011 => {
                    pia.keyboard_control = value;
                    return;
                }
                0xD012 => {
                    if pia.display_configured {
                        pia.display_output.push(value & 0x7F);
                    }
                    return;
                }
                0xD013 => {
                    pia.display_control = value;
                    pia.display_configured = true;
                    return;
                }
                _ => {}
            }
        }

        if self.config.memory.bank_switching && self.banks.len() > 1 {
            self.banks[self.current_bank].write(addr, value)
        } else {
            self.banks[0].write(addr, value)
        }
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

    pub fn load_rom(&mut self, data: &[u8], offset: u16) {
        for (i, &byte) in data.iter().enumerate() {
            let addr = offset.wrapping_add(i as u16);
            self.write(addr, byte);
        }
    }

    pub fn load_rom_bank(&mut self, bank: usize, data: &[u8], offset: u16) -> Result<(), String> {
        if bank >= self.banks.len() {
            return Err(format!("Bank {} out of range (max: {})", bank, self.banks.len() - 1));
        }

        for (i, &byte) in data.iter().enumerate() {
            let addr = offset.wrapping_add(i as u16);
            self.banks[bank].write(addr, byte);
        }

        Ok(())
    }

    pub fn set_bank(&mut self, bank: usize) -> Result<(), String> {
        if bank >= self.banks.len() {
            return Err(format!("Bank {} out of range", bank));
        }
        self.current_bank = bank;
        Ok(())
    }

    pub fn get_bank(&self) -> usize { self.current_bank }
    pub fn num_banks(&self) -> usize { self.banks.len() }
    pub fn bank_size(&self) -> usize { self.banks[self.current_bank].size }
    pub fn as_slice(&self) -> &[u8] { &self.banks[0].data }
    pub fn len(&self) -> usize { self.size }

    pub fn is_zero_page(addr: u16) -> bool { addr <= 0xFF }
    pub fn is_stack_page(addr: u16) -> bool { addr >= 0x0100 && addr <= 0x01FF }

    pub fn get_reset_vector(&self) -> u16 { self.read_u16(self.config.reset_vector) }
    pub fn set_reset_vector(&mut self, addr: u16) { self.write_u16(self.config.reset_vector, addr); }
    pub fn get_nmi_vector(&self) -> u16 { self.read_u16(self.config.nmi_vector) }
    pub fn set_nmi_vector(&mut self, addr: u16) { self.write_u16(self.config.nmi_vector, addr); }
    pub fn get_irq_vector(&self) -> u16 { self.read_u16(self.config.irq_vector) }
    pub fn set_irq_vector(&mut self, addr: u16) { self.write_u16(self.config.irq_vector, addr); }

    pub fn clear(&mut self) {
        for bank in &mut self.banks {
            bank.data.fill(0);
        }
        self.apple1_pia = Self::apple1_pia_for(&self.config);
    }

    pub fn copy_from(&mut self, other: &Memory) {
        for (i, bank) in self.banks.iter_mut().enumerate() {
            if i < other.banks.len() {
                bank.data.copy_from_slice(&other.banks[i].data);
            }
        }
        self.apple1_pia = other.apple1_pia.clone();
    }

    pub fn apple1_press_key(&mut self, ascii: u8) {
        if let Some(pia) = &mut self.apple1_pia {
            pia.keyboard_data = ascii | 0x80;
            pia.keyboard_ready = true;
        }
    }

    pub fn apple1_clear_key_ready(&mut self) {
        if let Some(pia) = &mut self.apple1_pia {
            pia.keyboard_ready = false;
        }
    }

    pub fn apple1_take_output(&mut self) -> Vec<u8> {
        if let Some(pia) = &mut self.apple1_pia {
            return pia.display_output.drain(..).collect();
        }
        Vec::new()
    }
}

impl Bus for Memory {
    fn read(&mut self, addr: u16) -> u8 {
        let value = Memory::read(self, addr);
        if addr == 0xD010 {
            self.apple1_clear_key_ready();
        }
        value
    }
    fn write(&mut self, addr: u16, value: u8) {
        Memory::write(self, addr, value)
    }
    fn read_u16(&mut self, addr: u16) -> u16 {
        Memory::read_u16(self, addr)
    }
    fn write_u16(&mut self, addr: u16, value: u16) {
        Memory::write_u16(self, addr, value)
    }
}

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
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation() {
        let config = MachineConfig::default();
        let mem = Memory::new(&config);
        assert_eq!(mem.len(), 65536);
    }

    #[test]
    fn test_basic_read_write() {
        let config = MachineConfig::default();
        let mut mem = Memory::new(&config);

        mem.write(0x1234, 0xAB);
        assert_eq!(mem.read(0x1234), 0xAB);
    }

    #[test]
    fn test_read_write_u16() {
        let config = MachineConfig::default();
        let mut mem = Memory::new(&config);

        mem.write_u16(0x1000, 0x1234);
        assert_eq!(mem.read(0x1000), 0x34);
        assert_eq!(mem.read(0x1001), 0x12);
        assert_eq!(mem.read_u16(0x1000), 0x1234);
    }

    #[test]
    fn test_load_rom() {
        let config = MachineConfig::default();
        let mut mem = Memory::new(&config);

        let rom = vec![0x00, 0xE0, 0x6A, 0x42];
        mem.load_rom(&rom, 0x8000);

        assert_eq!(mem.read(0x8000), 0x00);
        assert_eq!(mem.read(0x8001), 0xE0);
        assert_eq!(mem.read(0x8002), 0x6A);
        assert_eq!(mem.read(0x8003), 0x42);
    }

    #[test]
    fn test_zero_page_detection() {
        assert!(Memory::is_zero_page(0x0000));
        assert!(Memory::is_zero_page(0x00FF));
        assert!(!Memory::is_zero_page(0x0100));
    }

    #[test]
    fn test_stack_page_detection() {
        assert!(Memory::is_stack_page(0x0100));
        assert!(Memory::is_stack_page(0x01FF));
        assert!(!Memory::is_stack_page(0x00FF));
        assert!(!Memory::is_stack_page(0x0200));
    }

    #[test]
    fn test_vectors() {
        let config = MachineConfig::default();
        let mut mem = Memory::new(&config);

        mem.set_reset_vector(0x8000);
        assert_eq!(mem.get_reset_vector(), 0x8000);

        mem.set_nmi_vector(0x9000);
        assert_eq!(mem.get_nmi_vector(), 0x9000);

        mem.set_irq_vector(0xA000);
        assert_eq!(mem.get_irq_vector(), 0xA000);
    }

    #[test]
    fn test_bank_switching() {
        let mut config = MachineConfig::nmos6502();
        config.memory.bank_switching = true;
        config.memory.num_banks = 2;

        let mut mem = Memory::new(&config);
        assert_eq!(mem.num_banks(), 2);

        mem.set_bank(0).unwrap();
        mem.write(0x1000, 0xAA);

        mem.set_bank(1).unwrap();
        mem.write(0x1000, 0xBB);

        mem.set_bank(0).unwrap();
        assert_eq!(mem.read(0x1000), 0xAA);

        mem.set_bank(1).unwrap();
        assert_eq!(mem.read(0x1000), 0xBB);
    }

    #[test]
    fn test_load_rom_bank() {
        let mut config = MachineConfig::nmos6502();
        config.memory.bank_switching = true;
        config.memory.num_banks = 2;

        let mut mem = Memory::new(&config);

        let rom = vec![0x01, 0x02, 0x03];
        mem.load_rom_bank(1, &rom, 0x8000).unwrap();

        mem.set_bank(1).unwrap();
        assert_eq!(mem.read(0x8000), 0x01);
        assert_eq!(mem.read(0x8001), 0x02);
        assert_eq!(mem.read(0x8002), 0x03);
    }

    #[test]
    fn test_clear() {
        let config = MachineConfig::default();
        let mut mem = Memory::new(&config);

        mem.write(0x1000, 0xFF);
        mem.clear();
        assert_eq!(mem.read(0x1000), 0x00);
    }
}
