//! MOS 6502 CPU Emulator in Rust
//!
//! This crate provides a complete implementation of the MOS 6502 microprocessor.
//! This is Phase 1: Basic structure with CPU, Memory, and Emulator.
//!
//! # Features (Phase 1)
//!
//! - Basic CPU structure with registers (A, X, Y, PC, SP, SR)
//! - 64KB memory with bank switching support
//! - NMOS and CMOS (W65C02) variant support
//! - Stack operations
//! - Interrupt handling (NMI, IRQ, BRK, RTI)
//! - Save/Load state
//! - WASM bindings for web integration
//!
//! # Future Features
//!
//! - Full instruction set emulation (Phase 3)
//! - All 13 addressing modes (Phase 2)
//! - Cycle counting (Phase 7)
//!
//! # Example
//!
//! ```rust
//! use mos6502_core::*;
//!
//! let mut emulator = Emulator::new();
//! let rom = vec![0x00, 0xE0, 0x6A, 0x42];
//! emulator.load_rom(&rom, 0x8000);
//! emulator.reset();
//!
//! // Execute one instruction
//! emulator.tick();
//! ```

pub mod cpu;
pub mod instruction;

pub use cpu_bus::Bus;
pub use mos6502_config::{CpuFamily, CpuQuirks, MachineConfig};
pub use mos6502_memory::Memory;

use crate::cpu::Cpu;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Emulator state for save/load
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulatorState {
    pub cpu: crate::cpu::CpuState,
    pub memory: Vec<u8>,
    pub config: MachineConfig,
}

/// Main emulator structure
///
/// This is the primary interface for using the 6502 emulator.
/// It contains the CPU, memory, and configuration, and provides
/// methods for execution, debugging, and state management.
#[wasm_bindgen]
pub struct Emulator {
    cpu: Cpu,
    memory: Memory,
    config: MachineConfig,
    
    // For cycle-accurate mode
    total_cycles: u64,
}

// Non-WASM methods (generic over Bus)
impl Emulator {
    /// Execute one instruction via generic Bus
    pub fn tick_bus<B: Bus>(&mut self, bus: &mut B) -> u8 {
        if self.cpu.halted || self.cpu.waiting || self.cpu.stopped {
            return 0;
        }
        let cycles = instruction::execute(&mut self.cpu, bus);
        self.cpu.instructions += 1;
        self.total_cycles += cycles as u64;
        cycles
    }

    /// Trigger IRQ using a generic Bus for stack and vectors.
    pub fn trigger_irq_bus<B: Bus>(&mut self, bus: &mut B) -> bool {
        if self.cpu.sr.i() { return false; }
        self.cpu.push_pc_plus_1(bus);
        self.cpu.sr.set_b(false);
        self.cpu.push_sr(bus, false);
        self.cpu.sr.set_i(true);
        self.cpu.pc = bus.read_u16(self.config.irq_vector);
        self.total_cycles += 7;
        self.cpu.instructions += 1;
        true
    }
}

#[wasm_bindgen]
impl Emulator {
    /// Create a new emulator with default configuration (NMOS 6502)
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let config = MachineConfig::default();
        Self::from_config(config)
    }
    
    /// Create a new emulator with NMOS configuration
    pub fn new_nmos() -> Self {
        let config = MachineConfig::nmos();
        Self::from_config(config)
    }
    
    /// Create a new emulator with CMOS configuration
    pub fn new_cmos() -> Self {
        let config = MachineConfig::cmos();
        Self::from_config(config)
    }
    
    /// Create a new emulator with a specific configuration
    pub fn new_with_config(json: &str) -> Result<Emulator, String> {
        let config = MachineConfig::from_json(json)?;
        Ok(Self::from_config(config))
    }
    
    /// Create emulator from configuration
    fn from_config(config: MachineConfig) -> Self {
        Emulator {
            cpu: Cpu::new(config.clone()),
            memory: Memory::new(&config),
            config,
            total_cycles: 0,
        }
    }
    
    // ============================================
    // ROM Loading
    // ============================================
    
    /// Load a ROM into memory at the specified offset
    pub fn load_rom(&mut self, data: &[u8], offset: u16) {
        self.memory.load_rom(data, offset);
    }
    
    /// Load a ROM into memory at the default offset (from config)
    pub fn load_rom_default(&mut self, data: &[u8]) {
        self.load_rom(data, self.config.start_address);
    }
    
    // ============================================
    // Execution
    // ============================================
    
    /// Execute one instruction (one opcode)
    pub fn tick(&mut self) -> u8 {
        if self.cpu.halted || self.cpu.waiting || self.cpu.stopped {
            return 0;
        }

        let cycles = instruction::execute(&mut self.cpu, &mut self.memory);
        self.cpu.instructions += 1;
        self.total_cycles += cycles as u64;
        cycles
    }
    
    /// Execute a single cycle (cycle-accurate mode)
    pub fn step(&mut self) -> u8 {
        let cycles = self.tick();
        cycles
    }
    
    /// Execute multiple cycles
    pub fn run(&mut self, cycles: u64) -> u64 {
        let mut executed = 0u64;
        for _ in 0..cycles {
            let c = self.step() as u64;
            executed += c;
            if c == 0 {
                break; // CPU halted/stopped
            }
        }
        executed
    }

    /// Execute multiple instructions (for WASM batch execution)
    /// Returns number of instructions actually executed.
    pub fn run_instructions(&mut self, count: u32) -> u32 {
        let mut executed = 0u32;
        for _ in 0..count {
            let c = self.tick();
            if c == 0 { break; }
            executed += 1;
        }
        executed
    }
    
    // ============================================
    // Register Access
    // ============================================
    
    /// Get the Accumulator register
    pub fn get_register_a(&self) -> u8 {
        self.cpu.a
    }
    
    /// Set the Accumulator register
    pub fn set_register_a(&mut self, value: u8) {
        self.cpu.a = value;
    }
    
    /// Get the X register
    pub fn get_register_x(&self) -> u8 {
        self.cpu.x
    }
    
    /// Set the X register
    pub fn set_register_x(&mut self, value: u8) {
        self.cpu.x = value;
    }
    
    /// Get the Y register
    pub fn get_register_y(&self) -> u8 {
        self.cpu.y
    }
    
    /// Set the Y register
    pub fn set_register_y(&mut self, value: u8) {
        self.cpu.y = value;
    }
    
    /// Get the Program Counter
    pub fn get_register_pc(&self) -> u16 {
        self.cpu.pc
    }
    
    /// Set the Program Counter
    pub fn set_register_pc(&mut self, value: u16) {
        self.cpu.pc = value;
    }
    
    /// Get the Stack Pointer
    pub fn get_register_sp(&self) -> u8 {
        self.cpu.sp
    }
    
    /// Set the Stack Pointer
    pub fn set_register_sp(&mut self, value: u8) {
        self.cpu.sp = value;
    }
    
    // ============================================
    // Status Register Access
    // ============================================
    
    /// Get the full Status Register value
    pub fn get_status_register(&self) -> u8 {
        self.cpu.sr.value()
    }
    
    /// Set the Status Register
    pub fn set_status_register(&mut self, value: u8) {
        self.cpu.sr.set(value);
    }
    
    /// Get Negative flag (N)
    pub fn get_status_n(&self) -> bool {
        self.cpu.sr.n()
    }
    
    /// Get Overflow flag (V)
    pub fn get_status_v(&self) -> bool {
        self.cpu.sr.v()
    }
    
    /// Get Break flag (B) - Note: not a physical bit, only in pushed SR
    pub fn get_status_b(&self) -> bool {
        self.cpu.sr.b()
    }
    
    /// Get Decimal flag (D)
    pub fn get_status_d(&self) -> bool {
        self.cpu.sr.d()
    }
    
    /// Get Interrupt Disable flag (I)
    pub fn get_status_i(&self) -> bool {
        self.cpu.sr.i()
    }
    
    /// Get Zero flag (Z)
    pub fn get_status_z(&self) -> bool {
        self.cpu.sr.z()
    }
    
    /// Get Carry flag (C)
    pub fn get_status_c(&self) -> bool {
        self.cpu.sr.c()
    }

    /// Get the total number of cycles executed
    pub fn get_cycle_count(&self) -> u64 {
        self.total_cycles
    }

    /// Get the total number of instructions executed
    pub fn get_instruction_count(&self) -> u64 {
        self.cpu.instructions
    }

    // ============================================
    // Memory Access
    // ============================================
    
    /// Read a byte from memory
    pub fn get_memory(&self, addr: u16) -> u8 {
        self.memory.read(addr)
    }
    
    /// Write a byte to memory
    pub fn set_memory(&mut self, addr: u16, value: u8) {
        self.memory.write(addr, value);
    }
    
    /// Get memory as a raw pointer (for WASM)
    pub fn get_memory_ptr(&self) -> *const u8 {
        self.memory.as_slice().as_ptr()
    }
    
    /// Get memory length
    pub fn get_memory_len(&self) -> usize {
        self.memory.len()
    }
    
    /// Read a 16-bit value from memory (little-endian)
    pub fn get_memory_u16(&self, addr: u16) -> u16 {
        self.memory.read_u16(addr)
    }
    
    /// Write a 16-bit value to memory (little-endian)
    pub fn set_memory_u16(&mut self, addr: u16, value: u16) {
        self.memory.write_u16(addr, value);
    }

    /// Send one Apple 1 keyboard character through the PIA keyboard port.
    pub fn apple1_press_key(&mut self, ascii: u8) {
        self.memory.apple1_press_key(ascii);
    }

    /// Take pending Apple 1 display output produced by WozMon.
    pub fn apple1_take_output(&mut self) -> String {
        let bytes = self.memory.apple1_take_output();
        bytes.into_iter().map(char::from).collect()
    }
    
    // ============================================
    // Reset and State Management
    // ============================================
    
    /// Reset the emulator to power-on state
    pub fn reset(&mut self) {
        let reset_vector = self.memory.get_reset_vector();
        self.cpu.reset();
        self.total_cycles = 0;
        self.cpu.pc = reset_vector;
    }
    
    /// Soft reset (keep memory, reset CPU only)
    pub fn soft_reset(&mut self) {
        self.cpu.reset();
        self.total_cycles = 0;
        self.cpu.pc = self.memory.get_reset_vector();
    }

    /// Set the reset vector in memory
    pub fn set_reset_vector(&mut self, addr: u16) {
        self.memory.set_reset_vector(addr);
    }

    /// Get the reset vector from memory
    pub fn get_reset_vector(&self) -> u16 {
        self.memory.get_reset_vector()
    }
    
    /// Save the complete emulator state to a byte vector
    pub fn save_state(&self) -> Vec<u8> {
        let state = EmulatorState {
            cpu: self.cpu.get_state(),
            memory: self.memory.as_slice().to_vec(),
            config: self.config.clone(),
        };
        serde_json::to_vec(&state).unwrap_or_else(|_| Vec::new())
    }
    
    /// Load emulator state from a byte vector
    pub fn load_state(&mut self, state: &[u8]) -> Result<(), String> {
        let state: EmulatorState = serde_json::from_slice(state)
            .map_err(|e| format!("Failed to deserialize state: {}", e))?;
        
        self.cpu.set_state(&state.cpu);
        // For now, only load first bank if bank switching is enabled
        if !state.memory.is_empty() && !self.memory.banks.is_empty() {
            self.memory.banks[0].data.copy_from_slice(&state.memory);
        }
        self.config = state.config;
        
        Ok(())
    }
    
    // ============================================
    // Interrupts
    // ============================================
    
    /// Trigger a Non-Maskable Interrupt (NMI)
    pub fn trigger_nmi(&mut self) {
        // NMI cannot be blocked
        self.cpu.push_pc_plus_1(&mut self.memory);
        self.cpu.sr.set_b(false); // B=0 for hardware interrupt
        self.cpu.push_sr(&mut self.memory, false);
        self.cpu.sr.set_i(true); // Set interrupt disable flag
        self.cpu.pc = self.memory.get_nmi_vector();
        self.total_cycles += 7;
        self.cpu.instructions += 1;
    }
    
    /// Trigger a Maskable Interrupt (IRQ)
    /// Returns true if the interrupt was accepted, false if blocked
    pub fn trigger_irq(&mut self) -> bool {
        // IRQ can be blocked by I flag
        if self.cpu.sr.i() {
            return false;
        }
        
        self.cpu.push_pc_plus_1(&mut self.memory);
        self.cpu.sr.set_b(false); // B=0 for hardware interrupt
        self.cpu.push_sr(&mut self.memory, false);
        self.cpu.sr.set_i(true); // Set interrupt disable flag
        self.cpu.pc = self.memory.get_irq_vector();
        self.total_cycles += 7;
        self.cpu.instructions += 1;
        
        true
    }
    
    /// Trigger a BRK (software interrupt)
    pub fn trigger_brk(&mut self) {
        self.cpu.push_pc_plus_2(&mut self.memory);
        self.cpu.sr.set_b(true); // B=1 for BRK
        self.cpu.push_sr(&mut self.memory, true);
        self.cpu.sr.set_i(true); // Set interrupt disable flag
        self.cpu.pc = self.memory.get_irq_vector();
        self.total_cycles += 7;
        self.cpu.instructions += 1;
    }
    
    // ============================================
    // Debug Functions
    // ============================================
    
    /// Disassemble an instruction at the given address
    pub fn disassemble(&self, addr: u16) -> String {
        let opcode = self.memory.read(addr);
        if let Some(info) = instruction::decode(opcode) {
            // Show illegal opcodes as .byte
            if info.name.starts_with('*') {
                return format!(".byte ${:02X}", opcode);
            }
            match info.mode {
                instruction::AddressingMode::Implied | instruction::AddressingMode::Accumulator =>
                    format!("{}", info.name),
                instruction::AddressingMode::Immediate =>
                    format!("{} #${:02X}", info.name, self.memory.read(addr + 1)),
                instruction::AddressingMode::ZeroPage =>
                    format!("{} ${:02X}", info.name, self.memory.read(addr + 1)),
                instruction::AddressingMode::ZeroPageX =>
                    format!("{} ${:02X},X", info.name, self.memory.read(addr + 1)),
                instruction::AddressingMode::ZeroPageY =>
                    format!("{} ${:02X},Y", info.name, self.memory.read(addr + 1)),
                instruction::AddressingMode::Absolute =>
                    format!("{} ${:04X}", info.name, self.memory.read_u16(addr + 1)),
                instruction::AddressingMode::AbsoluteX =>
                    format!("{} ${:04X},X", info.name, self.memory.read_u16(addr + 1)),
                instruction::AddressingMode::AbsoluteY =>
                    format!("{} ${:04X},Y", info.name, self.memory.read_u16(addr + 1)),
                instruction::AddressingMode::Indirect =>
                    format!("{} (${:04X})", info.name, self.memory.read_u16(addr + 1)),
                instruction::AddressingMode::IndirectX =>
                    format!("{} (${:02X},X)", info.name, self.memory.read(addr + 1)),
                instruction::AddressingMode::IndirectY =>
                    format!("{} (${:02X}),Y", info.name, self.memory.read(addr + 1)),
                instruction::AddressingMode::Relative =>
                    format!("{} ${:04X}", info.name, addr.wrapping_add(2).wrapping_add(self.memory.read(addr + 1) as i8 as u16)),
            }
        } else {
            format!(".byte ${:02X}", opcode)
        }
    }
    
    /// Get information about an opcode
    pub fn get_opcode_info(&self, addr: u16) -> String {
        let opcode = self.memory.read(addr);
        let name = instruction::get_name(opcode);
        serde_json::json!({
            "address": addr,
            "opcode": opcode,
            "name": name,
        }).to_string()
    }
    
    // ============================================
    // Configuration
    // ============================================
    
    /// Get the current configuration as JSON
    pub fn get_config_json(&self) -> String {
        self.config.to_json()
    }
    
    /// Set configuration from JSON
    pub fn set_config(&mut self, json: &str) -> Result<(), String> {
        let config = MachineConfig::from_json(json)?;
        self.config = config.clone();
        // Recreate CPU with new config
        self.cpu = Cpu::new(config);
        Ok(())
    }
    
    /// Get the current variant as a string
    pub fn get_variant(&self) -> String {
        format!("{:?}", self.config.family).to_lowercase()
    }
    
    /// Set the variant
    pub fn set_variant(&mut self, variant: &str) -> Result<(), String> {
        match variant.to_lowercase().as_str() {
            "nmos6502" | "nmos" => { self.config.family = CpuFamily::Nmos6502; self.config.quirks = CpuQuirks::nmos(); Ok(()) }
            "w65c02" | "cmos" => { self.config.family = CpuFamily::W65C02; self.config.quirks = CpuQuirks::cmos(); Ok(()) }
            "ricoh2a03" | "nes" => { self.config.family = CpuFamily::Ricoh2A03; self.config.quirks = CpuQuirks::ricoh2a03(); Ok(()) }
            "r65c02" => { self.config.family = CpuFamily::R65C02; self.config.quirks = CpuQuirks::r65c02(); Ok(()) }
            "nmos6510" | "c64" => { self.config.family = CpuFamily::Nmos6510; Ok(()) }
            _ => Err(format!("Unknown variant: {}", variant)),
        }
    }
}



// Initialize panic hook for WASM
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
