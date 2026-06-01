use mos6502_bus::Bus;
use crate::cpu::Cpu;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressingMode {
    Implied,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
    Relative,
}

#[derive(Debug, Clone, Copy)]
pub struct OpcodeInfo {
    pub opcode: u8,
    pub name: &'static str,
    pub mode: AddressingMode,
    pub bytes: u8,
    pub cycles: u8,
}

// Fetch value via addressing mode. Returns (value, address, page_crossed).
// `pc` is the address of the operand (the byte after the opcode).
fn fetch_operand(cpu: &Cpu, memory: &mut impl Bus, mode: AddressingMode, pc: u16) -> (u8, u16, bool) {
    use AddressingMode::*;
    match mode {
        Implied | Accumulator => (cpu.a, 0, false),
        Immediate => (memory.read(pc), 0, false),
        ZeroPage => {
            let addr = memory.read(pc) as u16;
            (memory.read(addr), addr, false)
        }
        ZeroPageX => {
            let addr = memory.read(pc).wrapping_add(cpu.x) as u16;
            (memory.read(addr), addr, false)
        }
        ZeroPageY => {
            let addr = memory.read(pc).wrapping_add(cpu.y) as u16;
            (memory.read(addr), addr, false)
        }
        Absolute => {
            let addr = memory.read_u16(pc);
            (memory.read(addr), addr, false)
        }
        AbsoluteX => {
            let base = memory.read_u16(pc);
            let addr = base.wrapping_add(cpu.x as u16);
            let cross = (base & 0xFF00) != (addr & 0xFF00);
            (memory.read(addr), addr, cross)
        }
        AbsoluteY => {
            let base = memory.read_u16(pc);
            let addr = base.wrapping_add(cpu.y as u16);
            let cross = (base & 0xFF00) != (addr & 0xFF00);
            (memory.read(addr), addr, cross)
        }
        Indirect => (0, memory.read_u16(pc), false),
        IndirectX => {
            let base = memory.read(pc).wrapping_add(cpu.x);
            let lo = memory.read(base as u16) as u16;
            let hi = memory.read(base.wrapping_add(1) as u16) as u16;
            let addr = (hi << 8) | lo;
            (memory.read(addr), addr, false)
        }
        IndirectY => {
            let base = memory.read(pc);
            let lo = memory.read(base as u16) as u16;
            let hi = memory.read(base.wrapping_add(1) as u16) as u16;
            let ptr = (hi << 8) | lo;
            let addr = ptr.wrapping_add(cpu.y as u16);
            let cross = (ptr & 0xFF00) != (addr & 0xFF00);
            (memory.read(addr), addr, cross)
        }
        Relative => {
            let offset = memory.read(pc) as i8;
            (offset as u8, cpu.pc.wrapping_add(offset as u16), false)
        }
    }
}

// Fetch 16-bit address for JMP indirect (handles NMOS bug)
fn fetch_indirect_target(cpu: &Cpu, memory: &mut impl Bus, pc: u16) -> u16 {
    let ptr = memory.read_u16(pc);
    if cpu.has_jmp_indirect_bug() && (ptr & 0xFF) == 0xFF {
        let lo = memory.read(ptr);
        let hi = memory.read(ptr & 0xFF00);
        ((hi as u16) << 8) | (lo as u16)
    } else {
        memory.read_u16(ptr)
    }
}

// Helper to get the effective address for STA/STX/STY/INC/DEC etc (write variants)
fn fetch_address(cpu: &Cpu, memory: &mut impl Bus, mode: AddressingMode, pc: u16) -> u16 {
    use AddressingMode::*;
    match mode {
        ZeroPage => memory.read(pc) as u16,
        ZeroPageX => memory.read(pc).wrapping_add(cpu.x) as u16,
        ZeroPageY => memory.read(pc).wrapping_add(cpu.y) as u16,
        Absolute => memory.read_u16(pc),
        AbsoluteX => memory.read_u16(pc).wrapping_add(cpu.x as u16),
        AbsoluteY => memory.read_u16(pc).wrapping_add(cpu.y as u16),
        IndirectX => {
            let base = memory.read(pc).wrapping_add(cpu.x);
            let lo = memory.read(base as u16) as u16;
            let hi = memory.read(base.wrapping_add(1) as u16) as u16;
            (hi << 8) | lo
        }
        IndirectY => {
            let base = memory.read(pc);
            let lo = memory.read(base as u16) as u16;
            let hi = memory.read(base.wrapping_add(1) as u16) as u16;
            let ptr = (hi << 8) | lo;
            ptr.wrapping_add(cpu.y as u16)
        }
        _ => 0,
    }
}

impl Cpu {
    pub fn has_jmp_indirect_bug(&self) -> bool {
        self.config.has_jmp_indirect_bug()
    }
}

macro_rules! update_nz { ($cpu:expr, $val:expr) => { $cpu.sr.set_n($val & 0x80 != 0); $cpu.sr.set_z($val == 0); } }

fn exec_lda(cpu: &mut Cpu, val: u8) { cpu.a = val; update_nz!(cpu, cpu.a); }
fn exec_ldx(cpu: &mut Cpu, val: u8) { cpu.x = val; update_nz!(cpu, cpu.x); }
fn exec_ldy(cpu: &mut Cpu, val: u8) { cpu.y = val; update_nz!(cpu, cpu.y); }
fn exec_sta(cpu: &mut Cpu, memory: &mut impl Bus, addr: u16) { memory.write(addr, cpu.a); }
fn exec_stx(cpu: &mut Cpu, memory: &mut impl Bus, addr: u16) { memory.write(addr, cpu.x); }
fn exec_sty(cpu: &mut Cpu, memory: &mut impl Bus, addr: u16) { memory.write(addr, cpu.y); }

fn exec_tax(cpu: &mut Cpu) { cpu.x = cpu.a; update_nz!(cpu, cpu.x); }
fn exec_tay(cpu: &mut Cpu) { cpu.y = cpu.a; update_nz!(cpu, cpu.y); }
fn exec_txa(cpu: &mut Cpu) { cpu.a = cpu.x; update_nz!(cpu, cpu.a); }
fn exec_tya(cpu: &mut Cpu) { cpu.a = cpu.y; update_nz!(cpu, cpu.a); }
fn exec_tsx(cpu: &mut Cpu) { cpu.x = cpu.sp; update_nz!(cpu, cpu.x); }
fn exec_txs(cpu: &mut Cpu) { cpu.sp = cpu.x; }

fn exec_pha(cpu: &mut Cpu, memory: &mut impl Bus) {
    cpu.push_stack(cpu.a, memory);
}
fn exec_pla(cpu: &mut Cpu, memory: &mut impl Bus) {
    cpu.a = cpu.pull_stack(memory);
    update_nz!(cpu, cpu.a);
}
fn exec_php(cpu: &mut Cpu, memory: &mut impl Bus) {
    // PHP pushes SR with B=1 and unused=1, without modifying internal SR
    cpu.push_stack(cpu.sr.push_value() | 0x10, memory);
}
fn exec_plp(cpu: &mut Cpu, memory: &mut impl Bus) {
    let val = cpu.pull_stack(memory);
    cpu.sr = crate::cpu::StatusRegister::from_pulled(val);
}

fn exec_adc(cpu: &mut Cpu, val: u8) {
    let carry = cpu.sr.c() as u16;
    let result = cpu.a as u16 + val as u16 + carry;
    let overflow = ((cpu.a ^ val) & 0x80) == 0 && ((cpu.a ^ (result as u8)) & 0x80) != 0;
    cpu.sr.set_c(result > 0xFF);
    cpu.sr.set_v(overflow);
    cpu.a = result as u8;
    update_nz!(cpu, cpu.a);
}
fn exec_sbc(cpu: &mut Cpu, val: u8) {
    let c_in = cpu.sr.c() as u16;
    let result = (cpu.a as u16).wrapping_add((!val as u16).wrapping_add(1)).wrapping_sub(1 - c_in);
    let overflow = ((cpu.a ^ val) & 0x80) != 0 && ((cpu.a ^ (result as u8)) & 0x80) != 0;
    cpu.sr.set_c(result > 0xFF);
    cpu.sr.set_v(overflow);
    cpu.a = result as u8;
    update_nz!(cpu, cpu.a);
}
fn exec_and(cpu: &mut Cpu, val: u8) { cpu.a &= val; update_nz!(cpu, cpu.a); }
fn exec_ora(cpu: &mut Cpu, val: u8) { cpu.a |= val; update_nz!(cpu, cpu.a); }
fn exec_eor(cpu: &mut Cpu, val: u8) { cpu.a ^= val; update_nz!(cpu, cpu.a); }
fn exec_cmp(cpu: &mut Cpu, val: u8) { let r = cpu.a.wrapping_sub(val); cpu.sr.set_c(cpu.a >= val); cpu.sr.set_z(cpu.a == val); update_nz!(cpu, r); }
fn exec_cpx(cpu: &mut Cpu, val: u8) { let r = cpu.x.wrapping_sub(val); cpu.sr.set_c(cpu.x >= val); cpu.sr.set_z(cpu.x == val); update_nz!(cpu, r); }
fn exec_cpy(cpu: &mut Cpu, val: u8) { let r = cpu.y.wrapping_sub(val); cpu.sr.set_c(cpu.y >= val); cpu.sr.set_z(cpu.y == val); update_nz!(cpu, r); }

fn exec_inc(cpu: &mut Cpu, memory: &mut impl Bus, addr: u16) {
    let val = memory.read(addr).wrapping_add(1);
    memory.write(addr, val);
    update_nz!(cpu, val);
}
fn exec_dec(cpu: &mut Cpu, memory: &mut impl Bus, addr: u16) {
    let val = memory.read(addr).wrapping_sub(1);
    memory.write(addr, val);
    update_nz!(cpu, val);
}
fn exec_inx(cpu: &mut Cpu) { cpu.x = cpu.x.wrapping_add(1); update_nz!(cpu, cpu.x); }
fn exec_iny(cpu: &mut Cpu) { cpu.y = cpu.y.wrapping_add(1); update_nz!(cpu, cpu.y); }
fn exec_dex(cpu: &mut Cpu) { cpu.x = cpu.x.wrapping_sub(1); update_nz!(cpu, cpu.x); }
fn exec_dey(cpu: &mut Cpu) { cpu.y = cpu.y.wrapping_sub(1); update_nz!(cpu, cpu.y); }

fn exec_asl(cpu: &mut Cpu) -> u8 { cpu.sr.set_c(cpu.a & 0x80 != 0); cpu.a <<= 1; update_nz!(cpu, cpu.a); cpu.a }
fn exec_asl_mem(cpu: &mut Cpu, memory: &mut impl Bus, addr: u16) {
    let val = memory.read(addr);
    cpu.sr.set_c(val & 0x80 != 0);
    memory.write(addr, val << 1);
    update_nz!(cpu, val << 1);
}
fn exec_lsr(cpu: &mut Cpu) -> u8 { cpu.sr.set_c(cpu.a & 1 != 0); cpu.a >>= 1; update_nz!(cpu, cpu.a); cpu.a }
fn exec_lsr_mem(cpu: &mut Cpu, memory: &mut impl Bus, addr: u16) {
    let val = memory.read(addr);
    cpu.sr.set_c(val & 1 != 0);
    memory.write(addr, val >> 1);
    update_nz!(cpu, val >> 1);
}
fn exec_rol(cpu: &mut Cpu) -> u8 {
    let old_c = cpu.sr.c() as u8;
    cpu.sr.set_c(cpu.a & 0x80 != 0);
    cpu.a = (cpu.a << 1) | old_c;
    update_nz!(cpu, cpu.a);
    cpu.a
}
fn exec_rol_mem(cpu: &mut Cpu, memory: &mut impl Bus, addr: u16) {
    let val = memory.read(addr);
    let old_c = cpu.sr.c() as u8;
    cpu.sr.set_c(val & 0x80 != 0);
    memory.write(addr, (val << 1) | old_c);
    update_nz!(cpu, (val << 1) | old_c);
}
fn exec_ror(cpu: &mut Cpu) -> u8 {
    let old_c = (cpu.sr.c() as u8) << 7;
    cpu.sr.set_c(cpu.a & 1 != 0);
    cpu.a = (cpu.a >> 1) | old_c;
    update_nz!(cpu, cpu.a);
    cpu.a
}
fn exec_ror_mem(cpu: &mut Cpu, memory: &mut impl Bus, addr: u16) {
    let val = memory.read(addr);
    let old_c = (cpu.sr.c() as u8) << 7;
    cpu.sr.set_c(val & 1 != 0);
    memory.write(addr, (val >> 1) | old_c);
    update_nz!(cpu, (val >> 1) | old_c);
}

fn exec_bit(cpu: &mut Cpu, val: u8) {
    cpu.sr.set_z(cpu.a & val == 0);
    cpu.sr.set_n(val & 0x80 != 0);
    cpu.sr.set_v(val & 0x40 != 0);
}
fn exec_jmp(cpu: &mut Cpu, addr: u16) { cpu.pc = addr; }

// ===== Opcode Table =====

use AddressingMode::*;
const TABLE: [Option<(&str, AddressingMode, u8, u8)>; 256] = {
    let mut t: [Option<(&str, AddressingMode, u8, u8)>; 256] = [None; 256];
    // 0x0x
    t[0x00] = Some(("BRK",Implied,1,7)); t[0x01] = Some(("ORA",IndirectX,2,6));
    t[0x02] = Some(("KIL",Implied,1,2));
    t[0x03] = Some(("*SLO",Implied,1,2));
    t[0x04] = Some(("*NOP",ZeroPage,2,3));
    t[0x05] = Some(("ORA",ZeroPage,2,3)); t[0x06] = Some(("ASL",ZeroPage,2,5));
    t[0x07] = Some(("*SLO",Implied,1,2));
    t[0x08] = Some(("PHP",Implied,1,3)); t[0x09] = Some(("ORA",Immediate,2,2));
    t[0x0A] = Some(("ASL",Accumulator,1,2));
    t[0x0B] = Some(("*ANC",Implied,1,2));
    t[0x0C] = Some(("*NOP",Absolute,3,4));
    t[0x0D] = Some(("ORA",Absolute,3,4)); t[0x0E] = Some(("ASL",Absolute,3,6));
    t[0x0F] = Some(("*SLO",Implied,1,2));
    // 0x1x
    t[0x10] = Some(("BPL",Relative,2,2)); t[0x11] = Some(("ORA",IndirectY,2,5));
    t[0x14] = Some(("*NOP",ZeroPageX,2,4));
    t[0x15] = Some(("ORA",ZeroPageX,2,4)); t[0x16] = Some(("ASL",ZeroPageX,2,6));
    t[0x18] = Some(("CLC",Implied,1,2)); t[0x19] = Some(("ORA",AbsoluteY,3,4));
    t[0x1A] = Some(("*NOP",Implied,1,2));
    t[0x1D] = Some(("ORA",AbsoluteX,3,4)); t[0x1E] = Some(("ASL",AbsoluteX,3,7));
    t[0x1C] = Some(("*NOP",AbsoluteX,3,4));
    // 0x2x
    t[0x20] = Some(("JSR",Absolute,3,6)); t[0x21] = Some(("AND",IndirectX,2,6));
    t[0x24] = Some(("BIT",ZeroPage,2,3)); t[0x25] = Some(("AND",ZeroPage,2,3));
    t[0x26] = Some(("ROL",ZeroPage,2,5)); t[0x28] = Some(("PLP",Implied,1,4));
    t[0x29] = Some(("AND",Immediate,2,2)); t[0x2A] = Some(("ROL",Accumulator,1,2));
    t[0x2C] = Some(("BIT",Absolute,3,4)); t[0x2D] = Some(("AND",Absolute,3,4));
    t[0x2E] = Some(("ROL",Absolute,3,6));
    // 0x3x
    t[0x30] = Some(("BMI",Relative,2,2)); t[0x31] = Some(("AND",IndirectY,2,5));
    t[0x34] = Some(("*NOP",ZeroPageX,2,4)); t[0x35] = Some(("AND",ZeroPageX,2,4));
    t[0x36] = Some(("ROL",ZeroPageX,2,6));
    t[0x3A] = Some(("*NOP",Implied,1,2));
    t[0x38] = Some(("SEC",Implied,1,2)); t[0x39] = Some(("AND",AbsoluteY,3,4));
    t[0x3D] = Some(("AND",AbsoluteX,3,4)); t[0x3E] = Some(("ROL",AbsoluteX,3,7));
    t[0x3C] = Some(("*NOP",AbsoluteX,3,4));
    // 0x4x
    t[0x40] = Some(("RTI",Implied,1,6)); t[0x41] = Some(("EOR",IndirectX,2,6));
    t[0x44] = Some(("*NOP",ZeroPage,2,3));
    t[0x45] = Some(("EOR",ZeroPage,2,3)); t[0x46] = Some(("LSR",ZeroPage,2,5));
    t[0x48] = Some(("PHA",Implied,1,3)); t[0x49] = Some(("EOR",Immediate,2,2));
    t[0x4A] = Some(("LSR",Accumulator,1,2)); t[0x4C] = Some(("JMP",Absolute,3,3));
    t[0x4D] = Some(("EOR",Absolute,3,4)); t[0x4E] = Some(("LSR",Absolute,3,6));
    // 0x5x
    t[0x50] = Some(("BVC",Relative,2,2)); t[0x51] = Some(("EOR",IndirectY,2,5));
    t[0x54] = Some(("*NOP",ZeroPageX,2,4)); t[0x55] = Some(("EOR",ZeroPageX,2,4));
    t[0x56] = Some(("LSR",ZeroPageX,2,6));
    t[0x5A] = Some(("*NOP",Implied,1,2));
    t[0x58] = Some(("CLI",Implied,1,2)); t[0x59] = Some(("EOR",AbsoluteY,3,4));
    t[0x5D] = Some(("EOR",AbsoluteX,3,4)); t[0x5E] = Some(("LSR",AbsoluteX,3,7));
    t[0x5C] = Some(("*NOP",AbsoluteX,3,4));
    // 0x6x
    t[0x60] = Some(("RTS",Implied,1,6)); t[0x61] = Some(("ADC",IndirectX,2,6));
    t[0x64] = Some(("*NOP",ZeroPage,2,3));
    t[0x65] = Some(("ADC",ZeroPage,2,3)); t[0x66] = Some(("ROR",ZeroPage,2,5));
    t[0x68] = Some(("PLA",Implied,1,4)); t[0x69] = Some(("ADC",Immediate,2,2));
    t[0x6A] = Some(("ROR",Accumulator,1,2)); t[0x6C] = Some(("JMP",Indirect,3,5));
    t[0x6D] = Some(("ADC",Absolute,3,4)); t[0x6E] = Some(("ROR",Absolute,3,6));
    // 0x7x
    t[0x70] = Some(("BVS",Relative,2,2)); t[0x71] = Some(("ADC",IndirectY,2,5));
    t[0x74] = Some(("*NOP",ZeroPageX,2,4)); t[0x75] = Some(("ADC",ZeroPageX,2,4));
    t[0x76] = Some(("ROR",ZeroPageX,2,6));
    t[0x7A] = Some(("*NOP",Implied,1,2));
    t[0x78] = Some(("SEI",Implied,1,2)); t[0x79] = Some(("ADC",AbsoluteY,3,4));
    t[0x7D] = Some(("ADC",AbsoluteX,3,4)); t[0x7E] = Some(("ROR",AbsoluteX,3,7));
    t[0x7C] = Some(("*NOP",AbsoluteX,3,4));
    // 0x8x
    t[0x80] = Some(("*NOP",Immediate,2,2));
    t[0x81] = Some(("STA",IndirectX,2,6));
    t[0x82] = Some(("*NOP",Immediate,2,2));
    t[0x83] = Some(("*SAX",IndirectX,2,6));
    t[0x84] = Some(("STY",ZeroPage,2,3)); t[0x85] = Some(("STA",ZeroPage,2,3));
    t[0x86] = Some(("STX",ZeroPage,2,3));
    t[0x87] = Some(("*SAX",ZeroPage,2,3));
    t[0x88] = Some(("DEY",Implied,1,2));
    t[0x89] = Some(("*NOP",Immediate,2,2));
    t[0x8A] = Some(("TXA",Implied,1,2));
    t[0x8B] = Some(("*NOP",Implied,1,2));
    t[0x8C] = Some(("STY",Absolute,3,4)); t[0x8D] = Some(("STA",Absolute,3,4));
    t[0x8E] = Some(("STX",Absolute,3,4));
    t[0x8F] = Some(("*SAX",Absolute,3,4));
    t[0x97] = Some(("*SAX",ZeroPageY,2,4));
    t[0x9B] = Some(("*NOP",Implied,1,2));
    t[0x9C] = Some(("*NOP",Implied,1,2));
    t[0x9E] = Some(("*NOP",Implied,1,2));
    t[0x9F] = Some(("*NOP",Implied,1,2));
    // 0x9x
    t[0x90] = Some(("BCC",Relative,2,2)); t[0x91] = Some(("STA",IndirectY,2,6));
    t[0x94] = Some(("STY",ZeroPageX,2,4)); t[0x95] = Some(("STA",ZeroPageX,2,4));
    t[0x96] = Some(("STX",ZeroPageY,2,4)); t[0x98] = Some(("TYA",Implied,1,2));
    t[0x99] = Some(("STA",AbsoluteY,3,5)); t[0x9A] = Some(("TXS",Implied,1,2));
    t[0x9D] = Some(("STA",AbsoluteX,3,5));
    // 0xAx
    t[0xA0] = Some(("LDY",Immediate,2,2)); t[0xA1] = Some(("LDA",IndirectX,2,6));
    t[0xA2] = Some(("LDX",Immediate,2,2));
    t[0xA3] = Some(("*LAX",IndirectX,2,6));
    t[0xA4] = Some(("LDY",ZeroPage,2,3));
    t[0xA5] = Some(("LDA",ZeroPage,2,3));
    t[0xA6] = Some(("LDX",ZeroPage,2,3));
    t[0xA7] = Some(("*LAX",ZeroPage,2,3));
    t[0xA8] = Some(("TAY",Implied,1,2)); t[0xA9] = Some(("LDA",Immediate,2,2));
    t[0xAA] = Some(("TAX",Implied,1,2));
    t[0xAB] = Some(("*LAX",Immediate,2,2));
    t[0xAC] = Some(("LDY",Absolute,3,4)); t[0xAD] = Some(("LDA",Absolute,3,4));
    t[0xAE] = Some(("LDX",Absolute,3,4));
    t[0xAF] = Some(("*LAX",Absolute,3,4));
    // 0xBx
    t[0xB0] = Some(("BCS",Relative,2,2)); t[0xB1] = Some(("LDA",IndirectY,2,5));
    t[0xB3] = Some(("*LAX",IndirectY,2,5));
    t[0xB4] = Some(("LDY",ZeroPageX,2,4)); t[0xB5] = Some(("LDA",ZeroPageX,2,4));
    t[0xB6] = Some(("LDX",ZeroPageY,2,4));
    t[0xB7] = Some(("*LAX",ZeroPageY,2,4));
    t[0xB8] = Some(("CLV",Implied,1,2));
    t[0xB9] = Some(("LDA",AbsoluteY,3,4)); t[0xBA] = Some(("TSX",Implied,1,2));
    t[0xBC] = Some(("LDY",AbsoluteX,3,4)); t[0xBD] = Some(("LDA",AbsoluteX,3,4));
    t[0xBE] = Some(("LDX",AbsoluteY,3,4));
    t[0xBF] = Some(("*LAX",AbsoluteY,3,4));
    // 0xCx
    t[0xC0] = Some(("CPY",Immediate,2,2));
    t[0xC1] = Some(("CMP",IndirectX,2,6));
    t[0xC2] = Some(("*NOP",Immediate,2,2));
    t[0xC4] = Some(("CPY",ZeroPage,2,3)); t[0xC5] = Some(("CMP",ZeroPage,2,3));
    t[0xC6] = Some(("DEC",ZeroPage,2,5)); t[0xC8] = Some(("INY",Implied,1,2));
    t[0xC9] = Some(("CMP",Immediate,2,2)); t[0xCA] = Some(("DEX",Implied,1,2));
    t[0xCC] = Some(("CPY",Absolute,3,4)); t[0xCD] = Some(("CMP",Absolute,3,4));
    t[0xCE] = Some(("DEC",Absolute,3,6));
    // 0xDx
    t[0xD0] = Some(("BNE",Relative,2,2)); t[0xD1] = Some(("CMP",IndirectY,2,5));
    t[0xD4] = Some(("*NOP",ZeroPageX,2,4)); t[0xD5] = Some(("CMP",ZeroPageX,2,4));
    t[0xD6] = Some(("DEC",ZeroPageX,2,6));
    t[0xDA] = Some(("*NOP",Implied,1,2));
    t[0xD8] = Some(("CLD",Implied,1,2)); t[0xD9] = Some(("CMP",AbsoluteY,3,4));
    t[0xDD] = Some(("CMP",AbsoluteX,3,4)); t[0xDE] = Some(("DEC",AbsoluteX,3,7));
    t[0xDC] = Some(("*NOP",AbsoluteX,3,4));
    // 0xEx
    t[0xE0] = Some(("CPX",Immediate,2,2)); t[0xE1] = Some(("SBC",IndirectX,2,6));
    t[0xE2] = Some(("*NOP",Immediate,2,2));
    t[0xE4] = Some(("CPX",ZeroPage,2,3)); t[0xE5] = Some(("SBC",ZeroPage,2,3));
    t[0xE6] = Some(("INC",ZeroPage,2,5)); t[0xE8] = Some(("INX",Implied,1,2));
    t[0xE9] = Some(("SBC",Immediate,2,2)); t[0xEA] = Some(("NOP",Implied,1,2));
    t[0xEC] = Some(("CPX",Absolute,3,4)); t[0xED] = Some(("SBC",Absolute,3,4));
    t[0xEE] = Some(("INC",Absolute,3,6));
    t[0xEB] = Some(("*SBC",Immediate,2,2));
    // 0xFx
    t[0xF0] = Some(("BEQ",Relative,2,2)); t[0xF1] = Some(("SBC",IndirectY,2,5));
    t[0xF4] = Some(("*NOP",ZeroPageX,2,4)); t[0xF5] = Some(("SBC",ZeroPageX,2,4));
    t[0xF6] = Some(("INC",ZeroPageX,2,6));
    t[0xFA] = Some(("*NOP",Implied,1,2));
    t[0xF8] = Some(("SED",Implied,1,2)); t[0xF9] = Some(("SBC",AbsoluteY,3,4));
    t[0xFD] = Some(("SBC",AbsoluteX,3,4)); t[0xFE] = Some(("INC",AbsoluteX,3,7));
    t[0xFC] = Some(("*NOP",AbsoluteX,3,4));
    t
};
fn exec_jsr(cpu: &mut Cpu, memory: &mut impl Bus, addr: u16) {
    // CPU.pc already advanced past the instruction by execute()
    // JSR pushes (return_addr - 1); RTS pops and adds 1
    let ret = cpu.pc.wrapping_sub(1);
    cpu.push_stack((ret >> 8) as u8, memory);
    cpu.push_stack(ret as u8, memory);
    cpu.pc = addr;
}
fn exec_rts(cpu: &mut Cpu, memory: &mut impl Bus) {
    let lo = cpu.pull_stack(memory);
    let hi = cpu.pull_stack(memory);
    cpu.pc = (((hi as u16) << 8) | lo as u16).wrapping_add(1);
}

fn exec_branch(cpu: &mut Cpu, offset: u8, taken: bool) -> u8 {
    if !taken { return 2 }
    let old_pc = cpu.pc;
    cpu.pc = cpu.pc.wrapping_add(offset as i8 as u16);
    let same_page = (old_pc & 0xFF00) == (cpu.pc & 0xFF00);
    if same_page { 3 } else { 4 }
}

fn push_pc(cpu: &mut Cpu, memory: &mut impl Bus) {
    cpu.push_stack((cpu.pc >> 8) as u8, memory);
    cpu.push_stack(cpu.pc as u8, memory);
}
fn pull_pc(cpu: &mut Cpu, memory: &mut impl Bus) -> u16 {
    let lo = cpu.pull_stack(memory);
    let hi = cpu.pull_stack(memory);
    (hi as u16) << 8 | lo as u16
}
fn exec_brk(cpu: &mut Cpu, memory: &mut impl Bus) {
    cpu.pc = cpu.pc.wrapping_add(1);
    push_pc(cpu, memory);
    // Push SR with B=1 without modifying internal register
    cpu.push_stack(cpu.sr.push_value() | 0x10, memory);
    cpu.sr.set_i(true);
    cpu.pc = memory.read_u16(0xFFFE);
}
fn exec_rti(cpu: &mut Cpu, memory: &mut impl Bus) {
    let sr_val = cpu.pull_stack(memory);
    cpu.sr = crate::cpu::StatusRegister::from_pulled(sr_val);
    cpu.pc = pull_pc(cpu, memory);
}
fn exec_nop() {}

// Branch condition helpers
fn cond_bcc(cpu: &Cpu) -> bool { !cpu.sr.c() }
fn cond_bcs(cpu: &Cpu) -> bool { cpu.sr.c() }
fn cond_beq(cpu: &Cpu) -> bool { cpu.sr.z() }
fn cond_bmi(cpu: &Cpu) -> bool { cpu.sr.n() }
fn cond_bne(cpu: &Cpu) -> bool { !cpu.sr.z() }
fn cond_bpl(cpu: &Cpu) -> bool { !cpu.sr.n() }
fn cond_bvc(cpu: &Cpu) -> bool { !cpu.sr.v() }
fn cond_bvs(cpu: &Cpu) -> bool { cpu.sr.v() }

fn flg_clc(cpu: &mut Cpu) { cpu.sr.set_c(false); }
fn flg_cld(cpu: &mut Cpu) { cpu.sr.set_d(false); }
fn flg_cli(cpu: &mut Cpu) { cpu.sr.set_i(false); }
fn flg_clv(cpu: &mut Cpu) { cpu.sr.set_v(false); }
fn flg_sec(cpu: &mut Cpu) { cpu.sr.set_c(true); }
fn flg_sed(cpu: &mut Cpu) { cpu.sr.set_d(true); }
fn flg_sei(cpu: &mut Cpu) { cpu.sr.set_i(true); }

pub fn decode(opcode: u8) -> Option<OpcodeInfo> {
    TABLE[opcode as usize].map(|(name, mode, bytes, cycles)| OpcodeInfo { opcode, name, mode, bytes, cycles })
}

pub fn get_name(opcode: u8) -> &'static str {
    TABLE[opcode as usize].map(|(n, _, _, _)| n).unwrap_or("???")
}

pub fn execute(cpu: &mut Cpu, memory: &mut impl Bus) -> u8 {
    let opcode = memory.read(cpu.pc);
    let info = match decode(opcode) {
        Some(i) => i,
        None => { cpu.pc = cpu.pc.wrapping_add(1); return 2 }
    };
    let op_pc = cpu.pc + 1; // PC of operand
    // Advance PC past the instruction; branch/JMP/JSR/BRK/RTI overwrite it
    cpu.pc = cpu.pc.wrapping_add(info.bytes as u16);
    match opcode {
        0x00 => { exec_brk(cpu, memory); 7 }
        0xA9|0xA5|0xB5|0xAD|0xBD|0xB9|0xA1|0xB1 => { let (v,_,c)=fetch_operand(cpu,memory,info.mode,op_pc); exec_lda(cpu,v); info.cycles+if c{1}else{0} }
        0xA3|0xA7|0xAF|0xB3|0xB7|0xBF => { let (v,_,c)=fetch_operand(cpu,memory,info.mode,op_pc); exec_lda(cpu,v); exec_ldx(cpu,v); info.cycles+if c{1}else{0} }
        0xA2|0xA6|0xB6|0xAE|0xBE => { let (v,_,c)=fetch_operand(cpu,memory,info.mode,op_pc); exec_ldx(cpu,v); info.cycles+if c{1}else{0} }
        0xA0|0xA4|0xB4|0xAC|0xBC => { let (v,_,c)=fetch_operand(cpu,memory,info.mode,op_pc); exec_ldy(cpu,v); info.cycles+if c{1}else{0} }
        0x81|0x85|0x95|0x8D|0x9D|0x99|0x91 => { { let a=fetch_address(cpu,memory,info.mode,op_pc); exec_sta(cpu,memory,a) }; info.cycles }
        0x86|0x96|0x8E => { { let a=fetch_address(cpu,memory,info.mode,op_pc); exec_stx(cpu,memory,a) }; info.cycles }
        0x84|0x94|0x8C => { { let a=fetch_address(cpu,memory,info.mode,op_pc); exec_sty(cpu,memory,a) }; info.cycles }
        0x83|0x87|0x8F|0x97 => { let a=fetch_address(cpu,memory,info.mode,op_pc); memory.write(a, cpu.a & cpu.x); info.cycles }
        0xAA => { exec_tax(cpu); 2 } 0xA8 => { exec_tay(cpu); 2 }
        0x8A => { exec_txa(cpu); 2 } 0x98 => { exec_tya(cpu); 2 }
        0xBA => { exec_tsx(cpu); 2 } 0x9A => { exec_txs(cpu); 2 }
        0x48 => { exec_pha(cpu,memory); 3 } 0x68 => { exec_pla(cpu,memory); 4 }
        0x08 => { exec_php(cpu,memory); 3 } 0x28 => { exec_plp(cpu,memory); 4 }
        0x69|0x65|0x75|0x6D|0x7D|0x79|0x61|0x71 => { let (v,_,c)=fetch_operand(cpu,memory,info.mode,op_pc); exec_adc(cpu,v); info.cycles+if c{1}else{0} }
        0xE9|0xE5|0xF5|0xED|0xFD|0xF9|0xE1|0xF1|0xEB => { let (v,_,c)=fetch_operand(cpu,memory,info.mode,op_pc); exec_sbc(cpu,v); info.cycles+if c{1}else{0} }
        0x29|0x25|0x35|0x2D|0x3D|0x39|0x21|0x31 => { let (v,_,c)=fetch_operand(cpu,memory,info.mode,op_pc); exec_and(cpu,v); info.cycles+if c{1}else{0} }
        0x09|0x05|0x15|0x0D|0x1D|0x19|0x01|0x11 => { let (v,_,c)=fetch_operand(cpu,memory,info.mode,op_pc); exec_ora(cpu,v); info.cycles+if c{1}else{0} }
        0x49|0x45|0x55|0x4D|0x5D|0x59|0x41|0x51 => { let (v,_,c)=fetch_operand(cpu,memory,info.mode,op_pc); exec_eor(cpu,v); info.cycles+if c{1}else{0} }
        0xE6|0xF6|0xEE|0xFE => { { let a=fetch_address(cpu,memory,info.mode,op_pc); exec_inc(cpu,memory,a) }; info.cycles }
        0xC6|0xD6|0xCE|0xDE => { { let a=fetch_address(cpu,memory,info.mode,op_pc); exec_dec(cpu,memory,a) }; info.cycles }
        0xE8 => { exec_inx(cpu); 2 } 0xC8 => { exec_iny(cpu); 2 }
        0xCA => { exec_dex(cpu); 2 } 0x88 => { exec_dey(cpu); 2 }
        0x0A => { exec_asl(cpu); 2 } 0x4A => { exec_lsr(cpu); 2 }
        0x2A => { exec_rol(cpu); 2 } 0x6A => { exec_ror(cpu); 2 }
        0x06|0x16|0x0E|0x1E => { { let a=fetch_address(cpu,memory,info.mode,op_pc); exec_asl_mem(cpu,memory,a) }; info.cycles }
        0x46|0x56|0x4E|0x5E => { { let a=fetch_address(cpu,memory,info.mode,op_pc); exec_lsr_mem(cpu,memory,a) }; info.cycles }
        0x26|0x36|0x2E|0x3E => { { let a=fetch_address(cpu,memory,info.mode,op_pc); exec_rol_mem(cpu,memory,a) }; info.cycles }
        0x66|0x76|0x6E|0x7E => { { let a=fetch_address(cpu,memory,info.mode,op_pc); exec_ror_mem(cpu,memory,a) }; info.cycles }
        0xC9|0xC5|0xD5|0xCD|0xDD|0xD9|0xC1|0xD1 => { let (v,_,c)=fetch_operand(cpu,memory,info.mode,op_pc); exec_cmp(cpu,v); info.cycles+if c{1}else{0} }
        0xE0|0xE4|0xEC => { let (v,_,_)=fetch_operand(cpu,memory,info.mode,op_pc); exec_cpx(cpu,v); info.cycles }
        0xC0|0xC4|0xCC => { let (v,_,_)=fetch_operand(cpu,memory,info.mode,op_pc); exec_cpy(cpu,v); info.cycles }
        0x24|0x2C => { let (v,_,_)=fetch_operand(cpu,memory,info.mode,op_pc); exec_bit(cpu,v); info.cycles }
        0x4C => { let a=memory.read_u16(op_pc); exec_jmp(cpu,a); 3 }
        0x6C => { let a=fetch_indirect_target(cpu,memory,op_pc); exec_jmp(cpu,a); 5 }
        0x20 => { let a=memory.read_u16(op_pc); exec_jsr(cpu,memory,a); 6 }
        0x60 => { exec_rts(cpu,memory); 6 }
        0x90 => { let (o,_,_)=fetch_operand(cpu,memory,AddressingMode::Relative,op_pc); exec_branch(cpu,o,cond_bcc(cpu)) }
        0xB0 => { let (o,_,_)=fetch_operand(cpu,memory,AddressingMode::Relative,op_pc); exec_branch(cpu,o,cond_bcs(cpu)) }
        0xF0 => { let (o,_,_)=fetch_operand(cpu,memory,AddressingMode::Relative,op_pc); exec_branch(cpu,o,cond_beq(cpu)) }
        0x30 => { let (o,_,_)=fetch_operand(cpu,memory,AddressingMode::Relative,op_pc); exec_branch(cpu,o,cond_bmi(cpu)) }
        0xD0 => { let (o,_,_)=fetch_operand(cpu,memory,AddressingMode::Relative,op_pc); exec_branch(cpu,o,cond_bne(cpu)) }
        0x10 => { let (o,_,_)=fetch_operand(cpu,memory,AddressingMode::Relative,op_pc); exec_branch(cpu,o,cond_bpl(cpu)) }
        0x50 => { let (o,_,_)=fetch_operand(cpu,memory,AddressingMode::Relative,op_pc); exec_branch(cpu,o,cond_bvc(cpu)) }
        0x70 => { let (o,_,_)=fetch_operand(cpu,memory,AddressingMode::Relative,op_pc); exec_branch(cpu,o,cond_bvs(cpu)) }
        0x18 => { flg_clc(cpu); 2 } 0xD8 => { flg_cld(cpu); 2 } 0x58 => { flg_cli(cpu); 2 }
        0xB8 => { flg_clv(cpu); 2 } 0x38 => { flg_sec(cpu); 2 } 0xF8 => { flg_sed(cpu); 2 }
        0x78 => { flg_sei(cpu); 2 } 0xEA => 2,
        0x40 => { exec_rti(cpu,memory); 6 }
        _ => info.cycles
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mos6502_config::MachineConfig;
    use mos6502_memory::Memory;

    fn setup() -> (Cpu, Memory) {
        let config = MachineConfig::default();
        let cpu = Cpu::new(config.clone());
        let memory = Memory::new(&config);
        (cpu, memory)
    }

    #[test]
    fn test_decode_all_known() {
        let known: [u8; 56] = [
            0x00,0x01,0x05,0x06,0x08,0x09,0x0A,0x0D,0x0E,0x10,0x11,0x15,0x16,0x18,0x19,0x1D,
            0x1E,0x20,0x21,0x24,0x25,0x26,0x28,0x29,0x2A,0x2C,0x2D,0x2E,0x30,0x31,0x35,0x36,
            0x38,0x39,0x3D,0x3E,0x40,0x41,0x45,0x46,0x48,0x49,0x4A,0x4C,0x4D,0x4E,0x50,0x51,
            0x55,0x56,0x58,0x59,0x5D,0x5E,0x60,0x61,
        ];
        for &op in &known {
            assert!(decode(op).is_some(), "opcode {:02X} should be known", op);
        }
    }

    #[test]
    fn test_lda_immediate() {
        let (mut cpu, mut mem) = setup();
        mem.write(cpu.pc, 0xA9); mem.write(cpu.pc+1, 0x42);
        let cyc = execute(&mut cpu, &mut mem);
        assert_eq!(cpu.a, 0x42);
        assert!(!cpu.sr.n()); assert!(!cpu.sr.z());
        assert_eq!(cyc, 2);
    }

    #[test]
    fn test_tax() {
        let (mut cpu, mut mem) = setup();
        cpu.a = 0xFF;
        mem.write(cpu.pc, 0xAA);
        execute(&mut cpu, &mut mem);
        assert_eq!(cpu.x, 0xFF);
        assert!(cpu.sr.n());
    }

    #[test]
    fn test_adc_carry() {
        let (mut cpu, mut mem) = setup();
        cpu.a = 0xFF;
        mem.write(cpu.pc, 0x69); mem.write(cpu.pc+1, 0x01);
        execute(&mut cpu, &mut mem);
        assert_eq!(cpu.a, 0x00);
        assert!(cpu.sr.c());
        assert!(cpu.sr.z());
    }
}
