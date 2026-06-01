mod config;
mod memory;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct Emulator {
    cpu: chip8_cpu::cpu::Cpu,
    memory: memory::Memory,
    display: cpu_display::Display,
    keyboard: cpu_keyboard::Keyboard,
    config: config::MachineConfig,
}

fn from_config(cfg: config::MachineConfig) -> Emulator {
    let (w, h) = cfg.display_size();
    Emulator {
        cpu: chip8_cpu::cpu::Cpu::new(),
        memory: memory::Memory::with_font_offset(cfg.memory.font_offset),
        display: cpu_display::Display::new(w, h),
        keyboard: cpu_keyboard::Keyboard::new(),
        config: cfg,
    }
}

#[wasm_bindgen]
impl Emulator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        from_config(config::MachineConfig::default())
    }

    pub fn new_schip() -> Self {
        from_config(config::MachineConfig::schip())
    }

    pub fn new_xochip() -> Self {
        from_config(config::MachineConfig::xochip())
    }

    pub fn new_with_json(json: &str) -> Result<Emulator, String> {
        let cfg = config::MachineConfig::from_json(json)?;
        Ok(from_config(cfg))
    }

    pub fn load_config(&mut self, json: &str) -> Result<(), String> {
        let cfg = config::MachineConfig::from_json(json)?;
        let (w, h) = cfg.display_size();
        self.config = cfg;
        self.cpu = chip8_cpu::cpu::Cpu::new();
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
        if self.cpu.halted { return; }

        let opcode = (self.memory.read(self.cpu.pc) as u16) << 8
            | self.memory.read(self.cpu.pc + 1) as u16;
        self.cpu.pc += 2;

        let (shift_vy, memory_inc_i, vf_reset) = self.config.quirks();
        let quirks = chip8_cpu::instruction::Quirks { shift_vy, memory_inc_i, vf_reset };

        chip8_cpu::instruction::execute(
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

    pub fn get_register_pc(&self) -> u16 { self.cpu.pc }
    pub fn get_register_i(&self) -> u16 { self.cpu.i }
    pub fn get_register_v(&self, reg: u8) -> u8 { self.cpu.v[reg as usize] }
    pub fn get_delay(&self) -> u8 { self.cpu.delay }
    pub fn get_sound_remaining(&self) -> u8 { self.cpu.sound }
    pub fn paused(&self) -> bool { self.cpu.halted }

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
#[path = "tests/lib.rs"]
mod tests;
