use cpu_bus::Bus;
use mos6502_core::*;
use std::collections::VecDeque;

// ===== Simple PIA 6821 for Apple 1 =====

struct Pia6821 {
    keyboard: VecDeque<u8>,
    display: Vec<u8>,
}

impl Pia6821 {
    fn new() -> Self { Pia6821 { keyboard: VecDeque::new(), display: Vec::new() } }
    fn push_key(&mut self, ch: u8) { self.keyboard.push_back(ch); }
    fn read(&mut self, addr: u16) -> u8 {
        let val = match addr & 3 {
            0 => {
                let k = self.keyboard.pop_front();
                if let Some(ch) = k {
                    if ch >= 0x20 && ch <= 0x7E || ch == 0x0D || ch == 0x0A {
                        self.display.push(ch);
                    }
                }
                k.unwrap_or(0)
            }
            1 => { if !self.keyboard.is_empty() { 0x80 } else { 0 } }
            2 => 0,
            3 => 0,
            _ => 0,
        };
        val
    }
    fn write(&mut self, addr: u16, val: u8) {
        if (addr & 3) == 2 { self.display.push(val); }
    }
    fn take_display(&mut self) -> Vec<u8> { std::mem::take(&mut self.display) }
}

// ===== Apple 1 Bus =====

struct Apple1Bus {
    ram: Vec<u8>,
    basic_rom: Vec<u8>,
    wozmon_rom: Vec<u8>,
    pia: Pia6821,
    output: Vec<u8>,
    accum: Vec<u8>,
    accumulate: bool,
}

impl Apple1Bus {
    fn new(basic_rom: &[u8], wozmon_rom: &[u8]) -> Self {
        Apple1Bus {
            ram: vec![0; 0x1000],
            basic_rom: basic_rom.to_vec(),
            wozmon_rom: wozmon_rom.to_vec(),
            pia: Pia6821::new(),
            output: Vec::new(),
            accum: Vec::new(),
            accumulate: false,
        }
    }
    fn output_text(&mut self) -> String {
        let mut bytes: Vec<u8> = std::mem::take(&mut self.output);
        bytes.extend(std::mem::take(&mut self.pia.display));
        if self.accumulate { self.accum.extend(&bytes); }
        String::from_utf8_lossy(&bytes).to_string()
    }
    fn accumulated(&self) -> String {
        String::from_utf8_lossy(&self.accum).to_string()
    }
    fn press_key(&mut self, ch: u8) { self.pia.push_key(ch); }
    fn press_text(&mut self, s: &str) { for b in s.bytes() { self.pia.push_key(b); } }
}

impl Bus for Apple1Bus {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x0FFF => self.ram[addr as usize],
            0xD010..=0xD013 => self.pia.read(addr),
            0xE000..=0xEFFF => self.basic_rom[(addr - 0xE000) as usize],
            0xFF00..=0xFFFF => self.wozmon_rom[(addr - 0xFF00) as usize],
            _ => 0,
        }
    }
    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x0FFF => self.ram[addr as usize] = val,
            0xD010..=0xD013 => { self.pia.write(addr, val); self.output.push(val); }
            _ => {}
        }
    }
}

// ===== Helpers =====

fn load_rom_bytes(path: &str) -> Vec<u8> {
    let bytes = std::fs::read(path).unwrap_or_else(|_| {
        std::fs::read(format!("crates/6502/core/{}", path))
            .unwrap_or_else(|_| panic!("cannot open {}", path))
    });
    bytes
}

fn setup_apple1() -> (Emulator, Apple1Bus) {
    let basic = load_rom_bytes("tests/roms/apple-1/basic.bin");
    let wozmon = load_rom_bytes("tests/roms/apple-1/wozmon.bin");
    let mut bus = Apple1Bus::new(&basic, &wozmon);
    bus.write(0xFFFC, 0x00); bus.write(0xFFFD, 0xE0);
    let emu = Emulator::new();
    (emu, bus)
}

// ===== Tests =====

#[test]
fn test_basic_starts() {
    let (mut emu, mut bus) = setup_apple1();
    emu.set_register_pc(0xE000);
    emu.set_register_sp(0xFF);
    assert_eq!(bus.read(0xE000), 0x4C, "BASIC starts with JMP");
    assert_eq!(emu.tick_bus(&mut bus), 3);
    assert_eq!(emu.get_register_pc(), 0xE2B0);
}

#[test]
fn test_basic_keyboard_io() {
    let (mut emu, mut bus) = setup_apple1();
    emu.set_register_pc(0xE000);
    emu.set_register_sp(0xFF);

    bus.press_key(0x41);
    assert_eq!(bus.read(0xD011), 0x80, "KBDCR set");
    assert_eq!(bus.read(0xD010), 0x41, "KBD read");
    assert_eq!(bus.read(0xD011), 0x00, "KBDCR clear");
}

#[test]
fn test_basic_prompt_and_print() {
    let (mut emu, mut bus) = setup_apple1();
    emu.set_register_pc(0xE000);
    emu.set_register_sp(0xFF);

    // Accumulate output across ticks
    bus.accumulate = true;
    let mut seen_prompt = false;
    for tick in 0..100000 {
        let pc = emu.get_register_pc();
        if pc == 0xE003 || pc == 0xE006 { bus.press_key(0x0D); }
        emu.tick_bus(&mut bus);
        let _ = bus.output_text();
        let acc = bus.accumulated();
        if acc.contains('>') {
            seen_prompt = true;
            println!("Prompt found at tick {} ({} chars)", tick, acc.len());
            break;
        }
        if tick % 25000 == 24999 {
            println!("Tick {}: output {} chars, last 50: {:?}", tick, acc.len(),
                if acc.len() > 50 { &acc[acc.len()-50..] } else { &acc });
        }
    }

    if !seen_prompt {
        let all = bus.accumulated();
        panic!("No prompt. Output: {:?}", all);
    }

    // Send PRINT command
    bus.press_text("PRINT 1 + 1\r");

    // Process command
    let mut got_result = false;
    for _ in 0..10000 {
        let pc = emu.get_register_pc();
        if pc == 0xE003 || pc == 0xE006 { bus.press_key(0x0D); }
        emu.tick_bus(&mut bus);
        let _ = bus.output_text();
        let all = bus.accumulated();
        let last_200 = if all.len() > 200 { &all[all.len()-200..] } else { &all };
        if last_200.contains('2') && !last_200.contains("PRINT") {
            got_result = true;
            break;
        }
    }

    if !got_result {
        let all = bus.accumulated();
        println!("FINAL ACCUMULATED ({}): {:?}", all.len(), all);
    }
    assert!(got_result, "Expected PRINT 1+1 result");
}
