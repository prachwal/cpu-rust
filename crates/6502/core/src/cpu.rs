//! CPU module for MOS 6502 emulator
//!
//! Implements the 6502 processor core with all registers, flags,
//! and basic operations.

use mos6502_bus::Bus;
use mos6502_config::{CpuFamily, MachineConfig, RmwBehavior};
#[cfg(test)]
use mos6502_memory::Memory;
use serde::{Deserialize, Serialize};

/// Status Register (SR/P) - 8 bits with 7 flags
///
/// Bit layout: NV-BDIZC
/// - Bit 7 (N): Negative flag
/// - Bit 6 (V): Overflow flag
/// - Bit 5: Unused (always 1 when pushed to stack)
/// - Bit 4 (B): Break flag (set by BRK/PHP, cleared by PLP/RTI)
/// - Bit 3 (D): Decimal mode flag
/// - Bit 2 (I): Interrupt disable flag
/// - Bit 1 (Z): Zero flag
/// - Bit 0 (C): Carry flag
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusRegister(u8);

impl StatusRegister {
    /// Create a new status register with default values
    /// After reset: I=1, D=0, others=0, unused=1
    pub fn new() -> Self {
        // N=0, V=0, unused=1, B=0, D=0, I=1, Z=0, C=0
        // Binary: 0010_0100 = 0x24
        StatusRegister(0x24)
    }
    
    /// Create a status register with all flags cleared
    pub fn empty() -> Self {
        // All flags cleared, unused bit set to 1
        // Binary: 0010_0000 = 0x20
        StatusRegister(0x20)
    }
    
    /// Get raw value
    pub fn value(&self) -> u8 {
        self.0
    }
    
    /// Set raw value
    pub fn set(&mut self, value: u8) {
        // Always keep unused bit (bit 5) as 1
        self.0 = (value & 0xDF) | 0x20;
    }
    
    // Individual flag getters
    
    /// Negative flag (bit 7) - set if result is negative (bit 7 = 1)
    pub fn n(&self) -> bool {
        (self.0 & 0x80) != 0
    }
    
    /// Overflow flag (bit 6) - set if signed arithmetic overflow
    pub fn v(&self) -> bool {
        (self.0 & 0x40) != 0
    }
    
    /// Unused bit (bit 5) - always 1 when pushed to stack
    pub fn unused(&self) -> bool {
        (self.0 & 0x20) != 0
    }
    
    /// Break flag (bit 4) - set by BRK/PHP, cleared by PLP/RTI
    /// Note: This is not a physical bit, only appears in pushed SR
    pub fn b(&self) -> bool {
        (self.0 & 0x10) != 0
    }
    
    /// Decimal mode flag (bit 3) - enables BCD mode for ADC/SBC
    pub fn d(&self) -> bool {
        (self.0 & 0x08) != 0
    }
    
    /// Interrupt disable flag (bit 2) - blocks maskable interrupts
    pub fn i(&self) -> bool {
        (self.0 & 0x04) != 0
    }
    
    /// Zero flag (bit 1) - set if result is zero
    pub fn z(&self) -> bool {
        (self.0 & 0x02) != 0
    }
    
    /// Carry flag (bit 0) - set if carry/borrow occurred
    pub fn c(&self) -> bool {
        (self.0 & 0x01) != 0
    }
    
    // Individual flag setters
    
    /// Set Negative flag
    pub fn set_n(&mut self, value: bool) {
        if value {
            self.0 |= 0x80;
        } else {
            self.0 &= !0x80;
        }
    }
    
    /// Set Overflow flag
    pub fn set_v(&mut self, value: bool) {
        if value {
            self.0 |= 0x40;
        } else {
            self.0 &= !0x40;
        }
    }
    
    /// Set Break flag (not a physical bit, only used when pushing)
    pub fn set_b(&mut self, value: bool) {
        if value {
            self.0 |= 0x10;
        } else {
            self.0 &= !0x10;
        }
    }
    
    /// Set Decimal mode flag
    pub fn set_d(&mut self, value: bool) {
        if value {
            self.0 |= 0x08;
        } else {
            self.0 &= !0x08;
        }
    }
    
    /// Set Interrupt disable flag
    pub fn set_i(&mut self, value: bool) {
        if value {
            self.0 |= 0x04;
        } else {
            self.0 &= !0x04;
        }
    }
    
    /// Set Zero flag
    pub fn set_z(&mut self, value: bool) {
        if value {
            self.0 |= 0x02;
        } else {
            self.0 &= !0x02;
        }
    }
    
    /// Set Carry flag
    pub fn set_c(&mut self, value: bool) {
        if value {
            self.0 |= 0x01;
        } else {
            self.0 &= !0x01;
        }
    }
    
    /// Update N and Z flags based on a value
    pub fn update_nz(&mut self, value: u8) {
        self.set_n(value & 0x80 != 0);
        self.set_z(value == 0);
    }
    
    /// Update all flags after an arithmetic operation (ADC/SBC)
    /// 
    /// Parameters:
    /// - result: The 8-bit result
    /// - carry: Whether a carry occurred
    /// - overflow: Whether a signed overflow occurred
    pub fn update_arithmetic(&mut self, result: u8, carry: bool, overflow: bool) {
        self.set_n(result & 0x80 != 0);
        self.set_z(result == 0);
        self.set_c(carry);
        self.set_v(overflow);
    }
    
    /// Update flags after a logical operation (AND/ORA/EOR)
    pub fn update_logical(&mut self, result: u8) {
        self.set_n(result & 0x80 != 0);
        self.set_z(result == 0);
    }
    
    /// Update flags after a shift/rotate operation
    pub fn update_shift(&mut self, result: u8, carry_out: bool) {
        self.set_n(result & 0x80 != 0);
        self.set_z(result == 0);
        self.set_c(carry_out);
    }
    
    /// Update flags after a comparison (CMP/CPX/CPY)
    /// 
    /// Parameters:
    /// - result: The result of (reg - mem)
    /// - carry: Whether reg >= mem
    pub fn update_comparison(&mut self, result: u8, carry: bool) {
        self.set_n(result & 0x80 != 0);
        self.set_z(result == 0);
        self.set_c(carry);
    }
    
    /// Get value for pushing to stack (with B=1 for BRK/PHP)
    pub fn push_value(&self) -> u8 {
        // Set unused=1, keep all other bits as-is
        self.0 | 0x20
    }
    
    /// Create a status register from a pulled value (B is ignored)
    pub fn from_pulled(value: u8) -> Self {
        // Ignore B flag and unused bit when pulling
        // Clear B and unused, then set unused=1
        let value = (value & 0xEF) | 0x20; // B=0, unused=1
        StatusRegister(value)
    }
    
    /// Reset to power-on state
    pub fn reset(&mut self) {
        // After reset: I=1, D=0, others=0, unused=1
        self.0 = 0x24; // 0010_0100
    }
}

impl std::fmt::Display for StatusRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "NV-BDIZC: N={} V={} B={} D={} I={} Z={} C={}",
            self.n() as u8,
            self.v() as u8,
            self.b() as u8,
            self.d() as u8,
            self.i() as u8,
            self.z() as u8,
            self.c() as u8
        )
    }
}

impl serde::Serialize for StatusRegister {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(self.0)
    }
}

impl<'de> serde::Deserialize<'de> for StatusRegister {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        Ok(StatusRegister(value))
    }
}

/// CPU state for save/load functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuState {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: u16,
    pub sp: u8,
    pub sr: u8,
    pub cycles: u64,
    pub instructions: u64,
    pub halted: bool,
    pub waiting: bool,
    pub stopped: bool,
}

/// Main CPU structure
#[derive(Debug, Clone)]
pub struct Cpu {
    // Registers
    pub a: u8,            // Accumulator
    pub x: u8,            // Index register X
    pub y: u8,            // Index register Y
    pub pc: u16,         // Program Counter
    pub sp: u8,          // Stack Pointer
    
    // Status Register
    pub sr: StatusRegister,
    
    // Timing
    pub cycles: u64,      // Total cycles executed
    pub instructions: u64, // Total instructions executed
    
    // State flags
    pub halted: bool,      // CPU is halted (KIL/JAM)
    pub waiting: bool,     // CPU is waiting for interrupt (WAI)
    pub stopped: bool,     // CPU is stopped (STP)
    
    // Configuration reference
    pub config: MachineConfig,
}

impl Cpu {
    /// Create a new CPU with the given configuration
    pub fn new(config: MachineConfig) -> Self {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            pc: config.start_address,
            sp: 0xFF, // Stack pointer starts at $01FF
            sr: StatusRegister::new(),
            cycles: 0,
            instructions: 0,
            halted: false,
            waiting: false,
            stopped: false,
            config,
        }
    }
    
    /// Create a new CPU with default configuration
    pub fn with_defaults() -> Self {
        Self::new(MachineConfig::default())
    }
    
    /// Reset the CPU to power-on state
    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.pc = self.config.start_address;
        self.sp = 0xFF;
        self.sr.reset(); // I=1, D=0
        self.cycles = 0;
        self.instructions = 0;
        self.halted = false;
        self.waiting = false;
        self.stopped = false;
    }
    
    /// Get current state for save/load
    pub fn get_state(&self) -> CpuState {
        CpuState {
            a: self.a,
            x: self.x,
            y: self.y,
            pc: self.pc,
            sp: self.sp,
            sr: self.sr.value(),
            cycles: self.cycles,
            instructions: self.instructions,
            halted: self.halted,
            waiting: self.waiting,
            stopped: self.stopped,
        }
    }
    
    /// Set state from saved data
    pub fn set_state(&mut self, state: &CpuState) {
        self.a = state.a;
        self.x = state.x;
        self.y = state.y;
        self.pc = state.pc;
        self.sp = state.sp;
        self.sr.set(state.sr);
        self.cycles = state.cycles;
        self.instructions = state.instructions;
        self.halted = state.halted;
        self.waiting = state.waiting;
        self.stopped = state.stopped;
    }
    
    // Stack operations
    
    /// Push a byte onto the stack
    /// Stack grows downward from $01FF to $0100
    pub fn push_stack(&mut self, value: u8, memory: &mut impl Bus) {
        let addr = 0x0100 | (self.sp as u16);
        memory.write(addr, value);
        self.sp = self.sp.wrapping_sub(1);
    }
    
    /// Pull a byte from the stack
    pub fn pull_stack(&mut self, memory: &mut impl Bus) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let addr = 0x0100 | (self.sp as u16);
        memory.read(addr)
    }
    
    /// Push the Program Counter onto the stack (2 bytes, little-endian)
    pub fn push_pc(&mut self, memory: &mut impl Bus) {
        // Push PC+1 for most cases, PC+2 for BRK
        let pc_to_push = self.pc;
        let pc_low = (pc_to_push & 0xFF) as u8;
        let pc_high = ((pc_to_push >> 8) & 0xFF) as u8;
        
        self.push_stack(pc_high, memory);
        self.push_stack(pc_low, memory);
    }
    
    /// Push PC+1 (for NMI, IRQ)
    pub fn push_pc_plus_1(&mut self, memory: &mut impl Bus) {
        let pc_to_push = self.pc.wrapping_add(1);
        let pc_low = (pc_to_push & 0xFF) as u8;
        let pc_high = ((pc_to_push >> 8) & 0xFF) as u8;
        
        self.push_stack(pc_high, memory);
        self.push_stack(pc_low, memory);
    }
    
    /// Push PC+2 (for BRK)
    pub fn push_pc_plus_2(&mut self, memory: &mut impl Bus) {
        let pc_to_push = self.pc.wrapping_add(2);
        let pc_low = (pc_to_push & 0xFF) as u8;
        let pc_high = ((pc_to_push >> 8) & 0xFF) as u8;
        
        self.push_stack(pc_high, memory);
        self.push_stack(pc_low, memory);
    }
    
    /// Pull the Program Counter from the stack
    pub fn pull_pc(&mut self, memory: &mut impl Bus) {
        let pc_low = self.pull_stack(memory);
        let pc_high = self.pull_stack(memory);
        self.pc = ((pc_high as u16) << 8) | (pc_low as u16);
    }
    
    /// Push the Status Register onto the stack
    /// For BRK and PHP: B=1
    /// For IRQ/NMI: B=0
    pub fn push_sr(&mut self, memory: &mut impl Bus, brk_mode: bool) {
        if brk_mode {
            // BRK or PHP: B=1
            self.sr.set_b(true);
        } else {
            // IRQ or NMI: B=0
            self.sr.set_b(false);
        }
        let sr_value = self.sr.push_value();
        self.push_stack(sr_value, memory);
    }
    
    /// Pull the Status Register from the stack
    pub fn pull_sr(&mut self, memory: &mut impl Bus) {
        let sr_value = self.pull_stack(memory);
        self.sr = StatusRegister::from_pulled(sr_value);
    }
    
    // Helper methods for addressing modes
    
    /// Calculate effective address for zero page addressing
    pub fn zero_page_addr(&self, base: u8) -> u16 {
        base as u16
    }
    
    /// Calculate effective address for zero page,X addressing
    pub fn zero_page_x_addr(&self, base: u8) -> u16 {
        base.wrapping_add(self.x) as u16
    }
    
    /// Calculate effective address for zero page,Y addressing
    pub fn zero_page_y_addr(&self, base: u8) -> u16 {
        base.wrapping_add(self.y) as u16
    }
    
    /// Calculate effective address for absolute addressing
    pub fn absolute_addr(&self, lo: u8, hi: u8) -> u16 {
        ((hi as u16) << 8) | (lo as u16)
    }
    
    /// Calculate effective address for absolute,X addressing with page boundary check
    pub fn absolute_x_addr(&self, base: u16) -> (u16, bool) {
        let addr = base.wrapping_add(self.x as u16);
        let page_crossed = (base & 0xFF00) != (addr & 0xFF00);
        (addr, page_crossed)
    }
    
    /// Calculate effective address for absolute,Y addressing with page boundary check
    pub fn absolute_y_addr(&self, base: u16) -> (u16, bool) {
        let addr = base.wrapping_add(self.y as u16);
        let page_crossed = (base & 0xFF00) != (addr & 0xFF00);
        (addr, page_crossed)
    }
    
    /// Calculate effective address for indirect addressing (JMP only)
    /// Takes into account the NMOS JMP indirect bug
    pub fn indirect_addr(&self, ptr: u16, memory: &mut impl Bus) -> u16 {
        if self.config.has_jmp_indirect_bug() && (ptr & 0xFF) == 0xFF {
            // NMOS bug: when low byte is FF, high byte is read from same page
            let lo = memory.read(ptr);
            let hi = memory.read(ptr & 0xFF00); // Bug: should be ptr + 1
            ((hi as u16) << 8) | (lo as u16)
        } else {
            // Correct behavior: read from ptr and ptr+1
            memory.read_u16(ptr)
        }
    }
    
    /// Calculate effective address for (indirect,X) addressing
    pub fn indirect_x_addr(&self, base: u8, memory: &mut impl Bus) -> u16 {
        let ptr = base.wrapping_add(self.x) as u16;
        memory.read_u16(ptr)
    }
    
    /// Calculate effective address for (indirect),Y addressing with page boundary check
    pub fn indirect_y_addr(&self, base: u8, memory: &mut impl Bus) -> (u16, bool) {
        let ptr = base as u16;
        let target = memory.read_u16(ptr);
        let addr = target.wrapping_add(self.y as u16);
        let page_crossed = (target & 0xFF00) != (addr & 0xFF00);
        (addr, page_crossed)
    }
    
    /// Calculate relative address for branch instructions
    pub fn relative_addr(&self, offset: i8) -> u16 {
        // Sign extend the offset and add to PC
        self.pc.wrapping_add(offset as u16)
    }
    
    // Utility methods
    
    /// Check if adding an offset would cross a page boundary
    pub fn would_cross_page(base: u16, offset: u8) -> bool {
        let addr = base.wrapping_add(offset as u16);
        (base & 0xFF00) != (addr & 0xFF00)
    }
    
    /// Get the variant
    pub fn variant(&self) -> CpuFamily {
        self.config.family
    }
    
    /// Get the RMW behavior
    pub fn rmw_behavior(&self) -> RmwBehavior {
        self.config.quirks.rmw
    }
}

// Implement PartialEq for testing
impl PartialEq for Cpu {
    fn eq(&self, other: &Self) -> bool {
        self.a == other.a &&
        self.x == other.x &&
        self.y == other.y &&
        self.pc == other.pc &&
        self.sp == other.sp &&
        self.sr == other.sr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_cpu() -> (Cpu, Memory) {
        let config = MachineConfig::default();
        let cpu = Cpu::new(config.clone());
        let memory = Memory::new(&config);
        (cpu, memory)
    }

    #[test]
    fn test_new() {
        let (cpu, _) = setup_cpu();
        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.y, 0);
        assert_eq!(cpu.pc, 0x8000);
        assert_eq!(cpu.sp, 0xFF);
        assert!(!cpu.sr.n());
        assert!(!cpu.sr.v());
        assert!(!cpu.sr.z());
        assert!(!cpu.sr.c());
        assert!(cpu.sr.i()); // I=1 after reset
        assert!(!cpu.sr.d()); // D=0 after reset
    }

    #[test]
    fn test_reset() {
        let (mut cpu, _) = setup_cpu();
        cpu.a = 0xFF;
        cpu.x = 0xFF;
        cpu.y = 0xFF;
        cpu.pc = 0x0000;
        cpu.sp = 0x00;
        cpu.sr.set_n(true);
        cpu.sr.set_z(true);
        
        cpu.reset();
        
        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.y, 0);
        assert_eq!(cpu.pc, 0x8000);
        assert_eq!(cpu.sp, 0xFF);
        assert!(!cpu.sr.n());
        assert!(!cpu.sr.z());
    }

    #[test]
    fn test_push_pull_stack() {
        let (mut cpu, mut memory) = setup_cpu();
        
        cpu.push_stack(0x42, &mut memory);
        assert_eq!(cpu.sp, 0xFE);
        
        cpu.push_stack(0x99, &mut memory);
        assert_eq!(cpu.sp, 0xFD);
        
        let val1 = cpu.pull_stack(&mut memory);
        assert_eq!(val1, 0x99);
        assert_eq!(cpu.sp, 0xFE);
        
        let val2 = cpu.pull_stack(&mut memory);
        assert_eq!(val2, 0x42);
        assert_eq!(cpu.sp, 0xFF);
    }

    #[test]
    fn test_push_pull_pc() {
        let (mut cpu, mut memory) = setup_cpu();
        cpu.pc = 0x1234;
        
        cpu.push_pc(&mut memory);
        
        cpu.pc = 0x0000; // Clear PC
        cpu.pull_pc(&mut memory);
        
        assert_eq!(cpu.pc, 0x1234);
    }

    #[test]
    fn test_push_sr() {
        let (mut cpu, mut memory) = setup_cpu();
        cpu.sr.set_n(true);
        cpu.sr.set_z(true);
        cpu.sr.set_c(false);
        
        // Push as BRK (B=1)
        cpu.push_sr(&mut memory, true);
        
        let sr_value = memory.read(0x01FF);
        assert!(sr_value & 0x80 != 0); // N=1
        assert!(sr_value & 0x02 != 0); // Z=1
        assert!(sr_value & 0x10 != 0); // B=1
        assert!(sr_value & 0x20 != 0); // unused=1
        
        // Push as IRQ (B=0)
        cpu.sp = 0xFF; // Reset SP
        cpu.push_sr(&mut memory, false);
        
        let sr_value = memory.read(0x01FF);
        assert!(sr_value & 0x80 != 0); // N=1
        assert!(sr_value & 0x02 != 0); // Z=1
        assert!(sr_value & 0x10 == 0); // B=0
        assert!(sr_value & 0x20 != 0); // unused=1
    }

    #[test]
    fn test_pull_sr() {
        let (mut cpu, mut memory) = setup_cpu();
        
        // Push a known SR value
        memory.write(0x01FF, 0xFF); // All flags set
        cpu.sp = 0xFE; // Point to the value
        
        cpu.pull_sr(&mut memory);
        
        assert!(cpu.sr.n());
        assert!(cpu.sr.v());
        assert!(cpu.sr.z());
        assert!(cpu.sr.c());
        assert!(cpu.sr.i());
        assert!(cpu.sr.d());
        assert!(!cpu.sr.b()); // B is ignored when pulling
    }

    #[test]
    fn test_addressing_modes() {
        let (mut cpu, _) = setup_cpu();
        
        // Zero page
        assert_eq!(cpu.zero_page_addr(0x42), 0x0042);
        
        // Zero page,X
        cpu.x = 0x05;
        assert_eq!(cpu.zero_page_x_addr(0x42), 0x0047);
        
        // Zero page,Y
        cpu.y = 0x03;
        assert_eq!(cpu.zero_page_y_addr(0x42), 0x0045);
        
        // Absolute
        assert_eq!(cpu.absolute_addr(0x34, 0x12), 0x1234);
        
        // Absolute,X with page boundary
        cpu.x = 0x01;
        let (addr, crossed) = cpu.absolute_x_addr(0x12FF);
        assert_eq!(addr, 0x1300);
        assert!(crossed);
        
        // Absolute,X without page boundary
        let (addr, crossed) = cpu.absolute_x_addr(0x1200);
        assert_eq!(addr, 0x1201);
        assert!(!crossed);
        
        // Relative
        let addr = cpu.relative_addr(0x10);
        assert_eq!(addr, 0x8010);
        
        let addr = cpu.relative_addr(-0x10 as i8);
        assert_eq!(addr, 0x7FF0);
    }

    #[test]
    fn test_page_boundary_detection() {
        assert!(Cpu::would_cross_page(0x12FF, 0x01));
        assert!(!Cpu::would_cross_page(0x1200, 0xFF));
        assert!(!Cpu::would_cross_page(0x1200, 0x0F));
    }

    #[test]
    fn test_status_register_display() {
        let mut sr = StatusRegister::new();
        sr.set_n(true);
        sr.set_z(false);
        sr.set_c(true);
        
        let display = format!("{}", sr);
        assert!(display.contains("N=1"));
        assert!(display.contains("Z=0"));
        assert!(display.contains("C=1"));
    }

    #[test]
    fn test_status_register_push_value() {
        let mut sr = StatusRegister::new();
        sr.set_n(true);
        sr.set_c(true);
        
        let push_value = sr.push_value();
        // N=1, V=0, B=0, D=0, I=1, Z=0, C=1, unused=1
        // Binary: 1010_0101 = 0xA5
        assert_eq!(push_value, 0xA5);
    }

    #[test]
    fn test_variant_config() {
        let nmos_config = MachineConfig::nmos();
        let cpu = Cpu::new(nmos_config);
        assert_eq!(cpu.variant(), CpuFamily::Nmos6502);
        
        let cmos_config = MachineConfig::cmos();
        let cpu = Cpu::new(cmos_config);
        assert_eq!(cpu.variant(), CpuFamily::W65C02);
    }

    #[test]
    fn test_state_save_load() {
        let (mut cpu, _) = setup_cpu();
        cpu.a = 0x42;
        cpu.x = 0x13;
        cpu.y = 0x55;
        cpu.pc = 0xABCD;
        cpu.sp = 0xFE;
        cpu.cycles = 100;
        cpu.instructions = 50;
        
        let state = cpu.get_state();
        
        let (mut cpu2, _) = setup_cpu();
        cpu2.set_state(&state);
        
        assert_eq!(cpu2.a, cpu.a);
        assert_eq!(cpu2.x, cpu.x);
        assert_eq!(cpu2.y, cpu.y);
        assert_eq!(cpu2.pc, cpu.pc);
        assert_eq!(cpu2.sp, cpu.sp);
        assert_eq!(cpu2.cycles, cpu.cycles);
        assert_eq!(cpu2.instructions, cpu.instructions);
    }
}
