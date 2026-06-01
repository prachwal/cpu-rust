use mos6502_bus::Bus;
use mos6502_core::*;
use pia_6520::Pia6821;
use std::collections::VecDeque;
use wasm_bindgen::prelude::*;

// ===== Apple 1 Keyboard / Display helper (wraps generic PIA) =====

struct Apple1Pia {
    pia: Pia6821,
    keyboard: VecDeque<u8>,
    display: Vec<u8>,
}

impl Apple1Pia {
    fn new() -> Self {
        Apple1Pia { pia: Pia6821::new(), keyboard: VecDeque::new(), display: Vec::new() }
    }
    fn push_key(&mut self, ch: u8) { self.keyboard.push_back(ch); }
    fn read(&mut self, addr: u16) -> u8 {
        match addr & 3 {
            0 => {
                let k = self.keyboard.pop_front();
                if let Some(ch) = k { ch | 0x80 } else { 0 }
            }
            1 => { if !self.keyboard.is_empty() { 0x80 } else { 0 } }
            2 => { self.pia.read(addr, 0, 0) }
            3 => { self.pia.read(addr, 0, 0) }
            _ => 0,
        }
    }
    fn write(&mut self, addr: u16, val: u8) {
        match addr & 3 {
            0 | 1 => { self.pia.write(addr, val); }
            2 => { self.display.push(val & 0x7F); }
            3 => { self.pia.write(addr, val); }
            _ => {}
        }
    }
    fn push_display(&mut self, val: u8) {
        self.display.push(val & 0x7F);
    }
}

// ===== Apple 1 Bus =====

struct Apple1Bus {
    ram: Vec<u8>,
    basic_rom: Vec<u8>,
    wozmon_rom: Vec<u8>,
    pia: Apple1Pia,
}

impl Apple1Bus {
    fn new(basic_rom: &[u8], wozmon_rom: &[u8]) -> Self {
        Apple1Bus {
            ram: vec![0; 0x1000],
            basic_rom: basic_rom.to_vec(),
            wozmon_rom: wozmon_rom.to_vec(),
            pia: Apple1Pia::new(),
        }
    }
}

impl Bus for Apple1Bus {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x0FFF => self.ram[addr as usize],
            0xD010..=0xD013 => self.pia.read(addr),
            0xD0F0..=0xD0F3 => 0,
            0xE000..=0xEFFF => self.basic_rom[(addr - 0xE000) as usize],
            0xFF00..=0xFFFF => self.wozmon_rom[(addr - 0xFF00) as usize],
            _ => 0,
        }
    }
    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x0FFF => self.ram[addr as usize] = val,
            0xD010..=0xD013 => { self.pia.write(addr, val); }
            0xD0F0..=0xD0F3 => { self.pia.push_display(val); }
            _ => {}
        }
    }
}

// ===== WASM Exports =====

#[wasm_bindgen]
pub struct Apple1Emulator {
    cpu: Emulator,
    bus: Apple1Bus,
    roms_loaded: bool,
}

#[wasm_bindgen]
impl Apple1Emulator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        let bus = Apple1Bus::new(&[], &[]);
        let cpu = Emulator::new();
        Apple1Emulator { cpu, bus, roms_loaded: false }
    }

    /// Load BASIC and WozMon ROMs — boot from WozMon (0xFF00)
    pub fn load_roms(&mut self, basic: &[u8], wozmon: &[u8]) {
        self.bus = Apple1Bus::new(basic, wozmon);
        self.cpu = Emulator::new();
        // Reset vector from WozMon (0xFFFC-0xFFFD → 0xFF00)
        self.cpu.set_register_pc(0xFF00);
        self.cpu.set_register_sp(0xFF);
        self.roms_loaded = true;
    }

    /// Execute up to `count` instructions. Returns number actually executed.
    pub fn run(&mut self, count: u32) -> u32 {
        if !self.roms_loaded { return 0; }
        let mut n = 0u32;
        for _ in 0..count {
            let c = self.cpu.tick_bus(&mut self.bus);
            if c == 0 { break; }
            n += 1;
        }
        n
    }

    /// Send a keypress (ASCII code) to the Apple 1 keyboard
    pub fn press_key(&mut self, ascii: u8) {
        self.bus.pia.push_key(ascii);
    }

    /// Get display output pointer (raw ASCII bytes)
    pub fn display_ptr(&self) -> *const u8 {
        self.bus.pia.display.as_ptr()
    }

    /// Get display output length
    pub fn display_len(&self) -> usize {
        self.bus.pia.display.len()
    }

    /// Drain and return display output as a JS array
    pub fn take_display(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.bus.pia.display)
    }

    pub fn get_pc(&self) -> u16 { self.cpu.get_register_pc() }
    pub fn get_instructions(&self) -> u64 { self.cpu.get_instruction_count() }
    pub fn get_cycles(&self) -> u64 { self.cpu.get_cycle_count() }
}
