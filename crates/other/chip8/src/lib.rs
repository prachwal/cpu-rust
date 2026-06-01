mod config;
mod cpu;
mod instruction;
mod memory;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct Emulator {
    cpu: cpu::Cpu,
    memory: memory::Memory,
    display: cpu_display::Display,
    keyboard: cpu_keyboard::Keyboard,
    config: config::MachineConfig,
}

#[wasm_bindgen]
impl Emulator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let cfg = config::MachineConfig::default();
        Self::from_config(cfg)
    }

    fn from_config(cfg: config::MachineConfig) -> Self {
        let (w, h) = cfg.display_size();
        Emulator {
            cpu: cpu::Cpu::new(),
            memory: memory::Memory::with_font_offset(cfg.memory.font_offset),
            display: cpu_display::Display::new(w, h),
            keyboard: cpu_keyboard::Keyboard::new(),
            config: cfg,
        }
    }

    pub fn new_with_json(json: &str) -> Result<Emulator, String> {
        let cfg = config::MachineConfig::from_json(json)?;
        Ok(Self::from_config(cfg))
    }

    pub fn load_config(&mut self, json: &str) -> Result<(), String> {
        let cfg = config::MachineConfig::from_json(json)?;
        let (w, h) = cfg.display_size();
        self.config = cfg;
        self.cpu = cpu::Cpu::new();
        self.memory = memory::Memory::with_font_offset(self.config.memory.font_offset);
        self.display = cpu_display::Display::new(w, h);
        self.keyboard = cpu_keyboard::Keyboard::new();
        Ok(())
    }

    pub fn get_config_json(&self) -> String {
        self.config.to_json()
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        let offset = self.config.memory.rom_offset;
        self.memory.load_rom(data, offset);
    }

    pub fn tick(&mut self) {
        if self.cpu.halted {
            return;
        }

        let opcode = (self.memory.read(self.cpu.pc) as u16) << 8
            | self.memory.read(self.cpu.pc + 1) as u16;
        self.cpu.pc += 2;

        let (shift_vy, memory_inc_i, vf_reset) = self.config.quirks();
        let quirks = instruction::Quirks { shift_vy, memory_inc_i, vf_reset };

        instruction::execute(
            opcode,
            &mut self.cpu,
            &mut self.memory,
            &mut self.display,
            &self.keyboard,
            &quirks,
        );
    }

    pub fn tick_timers(&mut self) {
        self.cpu.tick_timers();
    }

    pub fn get_display(&self) -> Vec<u8> {
        self.display.get_buffer()
    }

    pub fn get_display_ptr(&self) -> *const u8 {
        self.display.buffer_ptr()
    }

    pub fn get_display_len(&self) -> usize {
        self.display.buffer_len()
    }

    pub fn get_instructions_per_second(&self) -> u32 {
        self.config.instructions_per_second()
    }

    pub fn get_display_width(&self) -> u16 {
        self.config.display.width
    }

    pub fn get_display_height(&self) -> u16 {
        self.config.display.height
    }

    pub fn get_pixel_width(&self) -> u16 {
        self.config.display.pixel_width
    }

    pub fn get_pixel_height(&self) -> u16 {
        self.config.display.pixel_height
    }

    pub fn get_sound(&self) -> bool {
        self.cpu.sound > 0
    }

    pub fn key_down(&mut self, key: u8) {
        self.keyboard.press(key);
    }

    pub fn key_up(&mut self, key: u8) {
        self.keyboard.release(key);
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.memory = memory::Memory::with_font_offset(self.config.memory.font_offset);
        self.display = cpu_display::Display::new(
            self.config.display.width,
            self.config.display.height,
        );
    }
}

#[cfg(test)]
mod integration {
    use super::*;

    #[test]
    fn test_tick_executes_instruction() {
        let mut emu = Emulator::new();
        // 0x6A42: LD VA, 0x42
        emu.memory.write(0x200, 0x6A);
        emu.memory.write(0x201, 0x42);
        emu.tick();
        assert_eq!(emu.cpu.v[0xA], 0x42);
        assert_eq!(emu.cpu.pc, 0x202);
    }

    #[test]
    fn test_load_rom_sets_memory() {
        let mut emu = Emulator::new();
        let rom = vec![0x00, 0xE0, 0x6A, 0x42];
        emu.load_rom(&rom);
        assert_eq!(emu.memory.read(0x200), 0x00);
        assert_eq!(emu.memory.read(0x203), 0x42);
    }

    #[test]
    fn test_keyboard_roundtrip() {
        let mut emu = Emulator::new();
        emu.key_down(0xF);
        assert!(emu.keyboard.is_pressed(0xF));
        emu.key_up(0xF);
        assert!(!emu.keyboard.is_pressed(0xF));
    }

    #[test]
    fn test_tick_timers() {
        let mut emu = Emulator::new();
        emu.cpu.delay = 3;
        emu.cpu.sound = 2;
        emu.tick_timers();
        assert_eq!(emu.cpu.delay, 2);
        assert_eq!(emu.cpu.sound, 1);
    }

    #[test]
    fn test_get_sound() {
        let mut emu = Emulator::new();
        assert!(!emu.get_sound());
        emu.cpu.sound = 1;
        assert!(emu.get_sound());
    }

    #[test]
    fn test_display_buffer_size() {
        let emu = Emulator::new();
        assert_eq!(emu.get_display().len(), 2048);
    }

    #[test]
    fn test_reset() {
        let mut emu = Emulator::new();
        emu.cpu.pc = 0x500;
        emu.cpu.v[0] = 42;
        emu.reset();
        assert_eq!(emu.cpu.pc, 0x200);
        assert_eq!(emu.cpu.v[0], 0);
    }
}
