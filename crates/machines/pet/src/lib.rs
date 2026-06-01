//! PET 2001 Emulator Core (Rust → WASM)
//!
//! Uses external: `pia-6520` (keyboard PIA #1, IEEE-488 PIA #2),
//!               `via-6522` (user port, timers, CB1 VBLANK)
//!
//! ## Memory Map
//!   $0000-$7FFF  RAM
//!   $8000-$83E7  Screen RAM (40×25)
//!   $C000-$CFFF  BASIC ROM (high)
//!   $D000-$DFFF  BASIC ROM (low)
//!   $E000-$E7FF  Editor ROM
//!   $E800-$EFFF  I/O
//!     $E810-$E81F  PIA #1 (keyboard matrix)
//!     $E820-$E82F  PIA #2 (IEEE-488)
//!     $E840-$E84F  VIA 6522
//!   $F000-$FFFF  Kernal ROM
//!
//! ## VIA connections
//!   CB1  — Vertical Blank (60Hz) — cursor blink / keyboard scan

use cpu_bus::Bus;
use mos6502_core::*;
use pia_6520::Pia6821;
use via_6522::Via6522;
use wasm_bindgen::prelude::*;

// =========================================================================
// Keyboard Matrix — 10 rows × 8 columns
// =========================================================================

struct KeyboardMatrix {
    rows: [u8; 10],
    selected_row: u8,
}

impl KeyboardMatrix {
    fn new() -> Self { KeyboardMatrix { rows: [0xFF; 10], selected_row: 0 } }
    fn press(&mut self, row: usize, col: usize) {
        if row < 10 && col < 8 { self.rows[row] &= !(1u8 << col); }
    }
    fn release(&mut self, row: usize, col: usize) {
        if row < 10 && col < 8 { self.rows[row] |= 1u8 << col; }
    }
    fn column_data(&self) -> u8 {
        self.rows.get(self.selected_row as usize).copied().unwrap_or(0xFF)
    }
}

// =========================================================================
// PET 2001 Bus
// =========================================================================

struct PetBus {
    ram: Vec<u8>,
    basic_c000: Vec<u8>,
    basic_d000: Vec<u8>,
    editor: Vec<u8>,
    kernal: Vec<u8>,
    screen: Vec<u8>,
    kbd: KeyboardMatrix,
    pia1: Pia6821,
    pia2: Pia6821,
    via: Via6522,
    vb_counter: u32,
    tick_counter: u64,
}

impl PetBus {
    fn new(basic_c000: &[u8], basic_d000: &[u8], editor: &[u8], kernal: &[u8]) -> Self {
        let mut b1 = vec![0u8; 0x1000];
        b1[..basic_c000.len()].copy_from_slice(basic_c000);
        let mut b2 = vec![0u8; 0x1000];
        b2[..basic_d000.len()].copy_from_slice(basic_d000);
        let mut ed = vec![0u8; 0x1000];
        ed[..editor.len()].copy_from_slice(editor);
        let mut kr = vec![0u8; 0x1000];
        kr[..kernal.len()].copy_from_slice(kernal);
        PetBus {
            ram: vec![0u8; 0x8000],
            basic_c000: b1, basic_d000: b2, editor: ed, kernal: kr,
            screen: vec![0x20u8; 1000],
            kbd: KeyboardMatrix::new(),
            pia1: Pia6821::new(), pia2: Pia6821::new(),
            via: Via6522::new(),
            vb_counter: 0, tick_counter: 0,
        }
    }
}

impl Bus for PetBus {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.ram[addr as usize],
            0x8000..=0x83E7 => self.screen[(addr - 0x8000) as usize],
            0x83E8..=0x8FFF | 0x9000..=0xBFFF => 0,
            0xC000..=0xCFFF => self.basic_c000[(addr - 0xC000) as usize],
            0xD000..=0xDFFF => self.basic_d000[(addr - 0xD000) as usize],
            0xE000..=0xE7FF => self.editor[(addr - 0xE000) as usize],
            0xE800..=0xEFFF => self.io_read(addr),
            0xF000..=0xFFFF => self.kernal[(addr - 0xF000) as usize],
        }
    }
    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x7FFF => self.ram[addr as usize] = val,
            0x8000..=0x83E7 => self.screen[(addr - 0x8000) as usize] = val,
            0x83E8..=0x8FFF | 0x9000..=0xBFFF => {},
            0xC000..=0xCFFF | 0xD000..=0xDFFF => {},
            0xE000..=0xE7FF => {},
            0xE800..=0xEFFF => self.io_write(addr, val),
            0xF000..=0xFFFF => {},
        }
    }
}

impl PetBus {
    fn io_read(&mut self, addr: u16) -> u8 {
        match addr {
            0xE810..=0xE813 => {
                // PIA1: keyboard matrix
                self.pia1.read(addr, 0xFF, self.kbd.column_data())
            }
            0xE820..=0xE823 => {
                // PIA2: IEEE-488 stub
                self.pia2.read(addr, 0, 0)
            }
            0xE840..=0xE84F => {
                self.via.read(addr)
            }
            _ => 0,
        }
    }
    fn io_write(&mut self, addr: u16, val: u8) {
        match addr {
            0xE810..=0xE813 => {
                self.pia1.write(addr, val);
                // Port A output bits 0-3 = row select
                self.kbd.selected_row = self.pia1.output_a() & 0x0F;
            }
            0xE820..=0xE823 => {
                self.pia2.write(addr, val);
            }
            0xE840..=0xE84F => {
                self.via.write(addr, val);
            }
            _ => {}
        }
    }
}

// =========================================================================
// WASM Exports
// =========================================================================

#[wasm_bindgen]
pub struct Pet2001 {
    cpu: Emulator,
    bus: PetBus,
    roms_loaded: bool,
    chargen: Vec<u8>,
}

#[wasm_bindgen]
impl Pet2001 {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        Pet2001 { cpu: Emulator::new(), bus: PetBus::new(&[], &[], &[], &[]),
                  roms_loaded: false, chargen: build_chargen() }
    }

    pub fn load_roms(&mut self, basic_c000: &[u8], basic_d000: &[u8], editor: &[u8], kernal: &[u8]) {
        self.bus = PetBus::new(basic_c000, basic_d000, editor, kernal);
        self.cpu = Emulator::new();
        self.cpu.set_register_sp(0xFF);
        let rv = (self.bus.kernal[0xFFC] as u16) | ((self.bus.kernal[0xFFD] as u16) << 8);
        self.cpu.set_register_pc(rv);
        // IRQ vector in internal memory for trigger_irq()
        self.cpu.set_memory(0xFFFE, self.bus.kernal[0xFFE]);
        self.cpu.set_memory(0xFFFF, self.bus.kernal[0xFFF]);
        // PIA1: configure DDRA = 0x0F (Port A bits 0-3 = row select outputs)
        self.bus.pia1.ddra = 0x0F;
        self.roms_loaded = true;
    }

    pub fn run(&mut self, count: u32) -> u32 {
        if !self.roms_loaded { return 0; }
        let mut n = 0u32;
        for _ in 0..count {
            // VIA tick (1 cycle per instruction)
            self.bus.via.tick(1);
            // Vertical blank: CB1 edge ~60Hz
            self.bus.vb_counter += 1;
            if self.bus.vb_counter >= 16666 {
                self.bus.vb_counter = 0;
                self.bus.via.trigger_cb1();
            }
            // Execute one instruction
            let c = self.cpu.tick_bus(&mut self.bus);
            if c == 0 { break; }
            // 60Hz tick — simulates VBLANK hardware cursor blink
            if n > 0 && n % 5555 == 0 {
                self.bus.ram[0x99] = self.bus.ram[0x99].wrapping_add(1); // jiffy
                self.bus.ram[0x9E] = 1; // cursor enable (unstick \$E29D wait loop)
                let a8 = self.bus.ram[0xA8];
                if a8 == 0 {
                    self.bus.ram[0xA8] = 20; // reset blink timer (20 frames ≈ 300ms)
                    self.bus.ram[0xAA] ^= 1; // toggle cursor blink flag
                } else {
                    self.bus.ram[0xA8] = a8 - 1;
                }
                self.bus.ram[0x8F] = self.bus.ram[0x8F].wrapping_add(1); // cursor counter
                if self.bus.ram[0x8F] >= 20 {
                    self.bus.ram[0x8F] = 0;
                }
            }
            // IRQ delivery
            if self.bus.via.irq {
                if self.cpu.trigger_irq_bus(&mut self.bus) { self.bus.via.irq = false; }
            }
            n += 1;
        }
        n
    }

    pub fn press_key(&mut self, row: u8, col: u8) {
        self.bus.kbd.press(row as usize, col as usize);
    }
    pub fn release_key(&mut self, row: u8, col: u8) {
        self.bus.kbd.release(row as usize, col as usize);
    }

    pub fn screen_ptr(&self) -> *const u8 { self.bus.screen.as_ptr() }
    pub fn screen_len(&self) -> usize { self.bus.screen.len() }
    pub fn ram_ptr(&self) -> *const u8 { self.bus.ram.as_ptr() }
    pub fn ram_len(&self) -> usize { self.bus.ram.len() }
    pub fn chargen_ptr(&self) -> *const u8 { self.chargen.as_ptr() }
    pub fn chargen_len(&self) -> usize { self.chargen.len() }
    pub fn get_pc(&self) -> u16 { self.cpu.get_register_pc() }
    pub fn get_instructions(&self) -> u64 { self.cpu.get_instruction_count() }
    pub fn get_cycles(&self) -> u64 { self.cpu.get_cycle_count() }
}

// =========================================================================
// Inline Character Generator ROM (2048 bytes, 256 chars × 8 bytes)
// =========================================================================

fn build_chargen() -> Vec<u8> {
    let mut data = vec![0u8; 2048];
    let mut set = |ch: usize, bytes: &[u8; 8]| {
        let off = ch * 8;
        for i in 0..8 { data[off + i] = bytes[i]; }
    };
    set(0x20, &[0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00]);
    set(0x21, &[0x18,0x18,0x18,0x18,0x18,0x00,0x18,0x00]);
    set(0x22, &[0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00]);
    set(0x23, &[0x24,0x24,0x7E,0x24,0x7E,0x24,0x24,0x00]);
    set(0x24, &[0x08,0x3E,0x48,0x3C,0x0A,0x7C,0x08,0x00]);
    set(0x25, &[0x62,0x64,0x08,0x10,0x26,0x46,0x00,0x00]);
    set(0x26, &[0x38,0x44,0x4C,0x38,0x4A,0x44,0x3A,0x00]);
    set(0x27, &[0x18,0x18,0x30,0x00,0x00,0x00,0x00,0x00]);
    set(0x28, &[0x0C,0x18,0x30,0x30,0x30,0x18,0x0C,0x00]);
    set(0x29, &[0x30,0x18,0x0C,0x0C,0x0C,0x18,0x30,0x00]);
    set(0x2A, &[0x00,0x18,0x7E,0x18,0x7E,0x18,0x00,0x00]);
    set(0x2B, &[0x00,0x18,0x18,0x7E,0x18,0x18,0x00,0x00]);
    set(0x2C, &[0x00,0x00,0x00,0x00,0x18,0x18,0x30,0x00]);
    set(0x2D, &[0x00,0x00,0x00,0x7E,0x00,0x00,0x00,0x00]);
    set(0x2E, &[0x00,0x00,0x00,0x00,0x18,0x18,0x00,0x00]);
    set(0x2F, &[0x02,0x06,0x0C,0x18,0x30,0x60,0x40,0x00]);
    set(0x30, &[0x3C,0x66,0x6E,0x7E,0x76,0x66,0x3C,0x00]);
    set(0x31, &[0x18,0x38,0x18,0x18,0x18,0x18,0x7E,0x00]);
    set(0x32, &[0x3C,0x66,0x06,0x0C,0x30,0x60,0x7E,0x00]);
    set(0x33, &[0x3C,0x66,0x06,0x1C,0x06,0x66,0x3C,0x00]);
    set(0x34, &[0x0C,0x1C,0x3C,0x6C,0x7E,0x0C,0x0C,0x00]);
    set(0x35, &[0x7E,0x60,0x7C,0x06,0x06,0x66,0x3C,0x00]);
    set(0x36, &[0x3C,0x66,0x60,0x7C,0x66,0x66,0x3C,0x00]);
    set(0x37, &[0x7E,0x06,0x0C,0x18,0x30,0x30,0x30,0x00]);
    set(0x38, &[0x3C,0x66,0x66,0x3C,0x66,0x66,0x3C,0x00]);
    set(0x39, &[0x3C,0x66,0x66,0x3E,0x06,0x66,0x3C,0x00]);
    set(0x3A, &[0x00,0x18,0x18,0x00,0x18,0x18,0x00,0x00]);
    set(0x3B, &[0x00,0x18,0x18,0x00,0x18,0x18,0x30,0x00]);
    set(0x3C, &[0x06,0x0C,0x18,0x30,0x18,0x0C,0x06,0x00]);
    set(0x3D, &[0x00,0x00,0x7E,0x00,0x7E,0x00,0x00,0x00]);
    set(0x3E, &[0x60,0x30,0x18,0x0C,0x18,0x30,0x60,0x00]);
    set(0x3F, &[0x3C,0x66,0x06,0x0C,0x18,0x00,0x18,0x00]);
    set(0x40, &[0x3C,0x66,0x6E,0x6E,0x60,0x66,0x3C,0x00]);
    set(0x5B, &[0x7E,0x60,0x60,0x60,0x60,0x60,0x7E,0x00]);
    set(0x5C, &[0x40,0x60,0x30,0x18,0x0C,0x06,0x02,0x00]);
    set(0x5D, &[0x7E,0x06,0x06,0x06,0x06,0x06,0x7E,0x00]);
    set(0x5E, &[0x18,0x3C,0x66,0x42,0x00,0x00,0x00,0x00]);
    set(0x5F, &[0x00,0x00,0x00,0x00,0x00,0x00,0x00,0xFF]);
    // === PET 2001 chargen layout: ===
    // 0x01-0x1A = A-Z uppercase (NOT 0x41-0x5A!)
    // 0x41-0x5A = same A-Z (for shifted/lowercase mode)
    // 0x60-0x7E = same glyphs as 0x40-0x5E (lowercase in shifted mode)
    let letter_data: [[u8; 8]; 26] = [
        [0x3C,0x66,0x66,0x7E,0x66,0x66,0x66,0x00], // A
        [0x7C,0x66,0x66,0x7C,0x66,0x66,0x7C,0x00], // B
        [0x3C,0x66,0x60,0x60,0x60,0x66,0x3C,0x00], // C
        [0x7C,0x66,0x66,0x66,0x66,0x66,0x7C,0x00], // D
        [0x7E,0x60,0x60,0x7C,0x60,0x60,0x7E,0x00], // E
        [0x7E,0x60,0x60,0x7C,0x60,0x60,0x60,0x00], // F
        [0x3C,0x66,0x60,0x6E,0x66,0x66,0x3C,0x00], // G
        [0x66,0x66,0x66,0x7E,0x66,0x66,0x66,0x00], // H
        [0x7E,0x18,0x18,0x18,0x18,0x18,0x7E,0x00], // I
        [0x1E,0x06,0x06,0x06,0x06,0x66,0x3C,0x00], // J
        [0x66,0x6C,0x78,0x70,0x78,0x6C,0x66,0x00], // K
        [0x60,0x60,0x60,0x60,0x60,0x60,0x7E,0x00], // L
        [0x63,0x77,0x7F,0x6B,0x63,0x63,0x63,0x00], // M
        [0x66,0x76,0x7E,0x6E,0x66,0x66,0x66,0x00], // N
        [0x3C,0x66,0x66,0x66,0x66,0x66,0x3C,0x00], // O
        [0x7C,0x66,0x66,0x7C,0x60,0x60,0x60,0x00], // P
        [0x3C,0x66,0x66,0x66,0x6E,0x3C,0x06,0x00], // Q
        [0x7C,0x66,0x66,0x7C,0x78,0x6C,0x66,0x00], // R
        [0x3C,0x66,0x60,0x3C,0x06,0x66,0x3C,0x00], // S
        [0x7E,0x18,0x18,0x18,0x18,0x18,0x18,0x00], // T
        [0x66,0x66,0x66,0x66,0x66,0x66,0x3C,0x00], // U
        [0x66,0x66,0x66,0x66,0x66,0x3C,0x18,0x00], // V
        [0x63,0x63,0x63,0x6B,0x7F,0x77,0x63,0x00], // W
        [0x66,0x66,0x3C,0x18,0x3C,0x66,0x66,0x00], // X
        [0x66,0x66,0x66,0x3C,0x18,0x18,0x18,0x00], // Y
        [0x7E,0x06,0x0C,0x18,0x30,0x60,0x7E,0x00], // Z
    ];
    for (i, glyph) in letter_data.iter().enumerate() {
        let idx = (i + 1) * 8;        // 0x01-0x1A
        for j in 0..8 { data[idx + j] = glyph[j]; }
        let idx2 = (i + 0x41) * 8;    // 0x41-0x5A (for lowercase mode)
        for j in 0..8 { data[idx2 + j] = glyph[j]; }
        let idx3 = (i + 0x61) * 8;    // 0x61-0x7A (lowercase slot)
        for j in 0..8 { data[idx3 + j] = glyph[j]; }
    }
    // 0x7F blank
    for i in 0..8 { data[0x7F * 8 + i] = 0; }
    // 0x80-0xFF = duplicate 0x00-0x7F
    for ch in 0x80..=0xFF {
        let src = (ch - 0x80) * 8;
        let dst = ch * 8;
        for i in 0..8 { data[dst + i] = data[src + i]; }
    }
    data
}
