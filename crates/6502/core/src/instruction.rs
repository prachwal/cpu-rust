use cpu_bus::Bus;
use mos6502_config::{CpuFamily, RmwBehavior};
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
    ZeroPageIndirect,
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
        ZeroPageIndirect => {
            let ptr = memory.read(pc) as u16;
            let addr = memory.read_u16(ptr);
            (memory.read(addr), addr, false)
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
        ZeroPageIndirect => {
            let ptr = memory.read(pc) as u16;
            memory.read_u16(ptr)
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
    let a = cpu.a as u16;
    let v = val as u16;
    let bin = a + v + carry;
    let overflow = ((cpu.a ^ val) & 0x80) == 0 && ((cpu.a ^ (bin as u8)) & 0x80) != 0;

    let (result, c) = if cpu.sr.d() && cpu.config.supports_bcd() {
        let mut lo = (a & 0x0F) + (v & 0x0F) + carry;
        let mut hi = (a >> 4) + (v >> 4);
        if lo >= 10 { lo -= 10; hi += 1; }
        let c = hi >= 10;
        if c { hi -= 10; }
        ((hi << 4) | lo, c)
    } else {
        (bin, bin > 0xFF)
    };

    cpu.sr.set_c(c);
    cpu.sr.set_v(overflow);
    cpu.a = result as u8;
    update_nz!(cpu, cpu.a);
}
fn exec_sbc(cpu: &mut Cpu, val: u8) {
    let c_in = cpu.sr.c() as u16;
    let a = cpu.a as u16;
    let bin = a.wrapping_add((!val) as u16).wrapping_add(c_in);
    let overflow = ((cpu.a ^ val) & 0x80) != 0 && ((cpu.a ^ (bin as u8)) & 0x80) != 0;
    let carry = bin > 0xFF;

    let (result, c) = if cpu.sr.d() && cpu.config.supports_bcd() {
        let not_carry = (1 - c_in) as u8;
        let a8 = cpu.a;
        let v8 = val;
        let mut r = a8.wrapping_sub(v8).wrapping_sub(not_carry);
        if (a8 & 0x0F) < (v8 & 0x0F).wrapping_add(not_carry) {
            r = r.wrapping_sub(6);
        }
        if a8 < v8.wrapping_add(not_carry) || r > 0x99 {
            r = r.wrapping_sub(0x60);
        }
        let c_ok = a8 >= v8.wrapping_add(not_carry);
        (r as u16, c_ok)
    } else {
        (bin, carry)
    };

    cpu.sr.set_c(c);
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

fn rmw_dummy(cpu: &Cpu, memory: &mut impl Bus, addr: u16, original: u8) {
    if cpu.config.rmw_behavior() == RmwBehavior::Cmos {
        let _ = memory.read(addr);
    } else {
        memory.write(addr, original);
    }
}
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
    rmw_dummy(cpu, memory, addr, val);
    memory.write(addr, val << 1);
    update_nz!(cpu, val << 1);
}
fn exec_lsr_mem(cpu: &mut Cpu, memory: &mut impl Bus, addr: u16) {
    let val = memory.read(addr);
    cpu.sr.set_c(val & 1 != 0);
    rmw_dummy(cpu, memory, addr, val);
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
    rmw_dummy(cpu, memory, addr, val);
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
    rmw_dummy(cpu, memory, addr, val);
    memory.write(addr, (val >> 1) | old_c);
    update_nz!(cpu, (val >> 1) | old_c);
}
fn exec_lsr(cpu: &mut Cpu) -> u8 { cpu.sr.set_c(cpu.a & 1 != 0); cpu.a >>= 1; update_nz!(cpu, cpu.a); cpu.a }

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
    t[0x03] = Some(("*SLO",IndirectX,2,8));
    t[0x04] = Some(("*NOP",ZeroPage,2,3));
    t[0x05] = Some(("ORA",ZeroPage,2,3)); t[0x06] = Some(("ASL",ZeroPage,2,5));
    t[0x07] = Some(("*SLO",ZeroPage,2,5));
    t[0x08] = Some(("PHP",Implied,1,3)); t[0x09] = Some(("ORA",Immediate,2,2));
    t[0x0A] = Some(("ASL",Accumulator,1,2));
    t[0x0B] = Some(("*ANC",Immediate,2,2));
    t[0x2B] = Some(("*ANC",Immediate,2,2));
    t[0x0C] = Some(("*NOP",Absolute,3,4));
    t[0x0D] = Some(("ORA",Absolute,3,4)); t[0x0E] = Some(("ASL",Absolute,3,6));
    t[0x0F] = Some(("*SLO",Absolute,3,6));
    // 0x1x
    t[0x10] = Some(("BPL",Relative,2,2)); t[0x11] = Some(("ORA",IndirectY,2,5));
    t[0x12] = Some(("KIL",Implied,1,2));
    t[0x13] = Some(("*SLO",IndirectY,2,8));
    t[0x14] = Some(("*NOP",ZeroPageX,2,4));
    t[0x15] = Some(("ORA",ZeroPageX,2,4)); t[0x16] = Some(("ASL",ZeroPageX,2,6));
    t[0x17] = Some(("*SLO",ZeroPageX,2,6));
    t[0x18] = Some(("CLC",Implied,1,2)); t[0x19] = Some(("ORA",AbsoluteY,3,4));
    t[0x1A] = Some(("*NOP",Implied,1,2));
    t[0x1B] = Some(("*SLO",AbsoluteY,3,7));
    t[0x1C] = Some(("*NOP",AbsoluteX,3,4));
    t[0x1D] = Some(("ORA",AbsoluteX,3,4)); t[0x1E] = Some(("ASL",AbsoluteX,3,7));
    t[0x1F] = Some(("*SLO",AbsoluteX,3,7));
    // 0x2x
    t[0x20] = Some(("JSR",Absolute,3,6)); t[0x21] = Some(("AND",IndirectX,2,6));
    t[0x22] = Some(("KIL",Implied,1,2));
    t[0x23] = Some(("*RLA",IndirectX,2,8));
    t[0x24] = Some(("BIT",ZeroPage,2,3)); t[0x25] = Some(("AND",ZeroPage,2,3));
    t[0x26] = Some(("ROL",ZeroPage,2,5)); t[0x27] = Some(("*RLA",ZeroPage,2,5));
    t[0x28] = Some(("PLP",Implied,1,4));
    t[0x29] = Some(("AND",Immediate,2,2)); t[0x2A] = Some(("ROL",Accumulator,1,2));
    t[0x2C] = Some(("BIT",Absolute,3,4)); t[0x2D] = Some(("AND",Absolute,3,4));
    t[0x2E] = Some(("ROL",Absolute,3,6)); t[0x2F] = Some(("*RLA",Absolute,3,6));
    // 0x3x
    t[0x30] = Some(("BMI",Relative,2,2)); t[0x31] = Some(("AND",IndirectY,2,5));
    t[0x32] = Some(("KIL",Implied,1,2));
    t[0x33] = Some(("*RLA",IndirectY,2,8));
    t[0x34] = Some(("*NOP",ZeroPageX,2,4)); t[0x35] = Some(("AND",ZeroPageX,2,4));
    t[0x36] = Some(("ROL",ZeroPageX,2,6)); t[0x37] = Some(("*RLA",ZeroPageX,2,6));
    t[0x38] = Some(("SEC",Implied,1,2)); t[0x39] = Some(("AND",AbsoluteY,3,4));
    t[0x3A] = Some(("*NOP",Implied,1,2));
    t[0x3B] = Some(("*RLA",AbsoluteY,3,7));
    t[0x3C] = Some(("*NOP",AbsoluteX,3,4));
    t[0x3D] = Some(("AND",AbsoluteX,3,4)); t[0x3E] = Some(("ROL",AbsoluteX,3,7));
    t[0x3F] = Some(("*RLA",AbsoluteX,3,7));
    // 0x4x
    t[0x40] = Some(("RTI",Implied,1,6)); t[0x41] = Some(("EOR",IndirectX,2,6));
    t[0x42] = Some(("KIL",Implied,1,2));
    t[0x43] = Some(("*SRE",IndirectX,2,8));
    t[0x44] = Some(("*NOP",ZeroPage,2,3));
    t[0x45] = Some(("EOR",ZeroPage,2,3)); t[0x46] = Some(("LSR",ZeroPage,2,5));
    t[0x47] = Some(("*SRE",ZeroPage,2,5));
    t[0x48] = Some(("PHA",Implied,1,3)); t[0x49] = Some(("EOR",Immediate,2,2));
    t[0x4A] = Some(("LSR",Accumulator,1,2)); t[0x4B] = Some(("*ALR",Immediate,2,2));
    t[0x4C] = Some(("JMP",Absolute,3,3)); t[0x4D] = Some(("EOR",Absolute,3,4));
    t[0x4E] = Some(("LSR",Absolute,3,6)); t[0x4F] = Some(("*SRE",Absolute,3,6));
    // 0x5x
    t[0x50] = Some(("BVC",Relative,2,2)); t[0x51] = Some(("EOR",IndirectY,2,5));
    t[0x52] = Some(("KIL",Implied,1,2));
    t[0x53] = Some(("*SRE",IndirectY,2,8));
    t[0x54] = Some(("*NOP",ZeroPageX,2,4)); t[0x55] = Some(("EOR",ZeroPageX,2,4));
    t[0x56] = Some(("LSR",ZeroPageX,2,6)); t[0x57] = Some(("*SRE",ZeroPageX,2,6));
    t[0x58] = Some(("CLI",Implied,1,2)); t[0x59] = Some(("EOR",AbsoluteY,3,4));
    t[0x5A] = Some(("PHY",Implied,1,3));
    t[0x5B] = Some(("*SRE",AbsoluteY,3,7));
    t[0x5C] = Some(("*NOP",AbsoluteX,3,4));
    t[0x5D] = Some(("EOR",AbsoluteX,3,4)); t[0x5E] = Some(("LSR",AbsoluteX,3,7));
    t[0x5F] = Some(("*SRE",AbsoluteX,3,7));
    // 0x6x
    t[0x60] = Some(("RTS",Implied,1,6)); t[0x61] = Some(("ADC",IndirectX,2,6));
    t[0x62] = Some(("KIL",Implied,1,2));
    t[0x63] = Some(("*RRA",IndirectX,2,8));
    t[0x64] = Some(("*NOP",ZeroPage,2,3));
    t[0x65] = Some(("ADC",ZeroPage,2,3)); t[0x66] = Some(("ROR",ZeroPage,2,5));
    t[0x67] = Some(("*RRA",ZeroPage,2,5));
    t[0x68] = Some(("PLA",Implied,1,4)); t[0x69] = Some(("ADC",Immediate,2,2));
    t[0x6A] = Some(("ROR",Accumulator,1,2)); t[0x6B] = Some(("*ARR",Immediate,2,2));
    t[0x6C] = Some(("JMP",Indirect,3,5));
    t[0x6D] = Some(("ADC",Absolute,3,4)); t[0x6E] = Some(("ROR",Absolute,3,6));
    t[0x6F] = Some(("*RRA",Absolute,3,6));
    // 0x7x
    t[0x70] = Some(("BVS",Relative,2,2)); t[0x71] = Some(("ADC",IndirectY,2,5));
    t[0x72] = Some(("KIL",Implied,1,2));
    t[0x73] = Some(("*RRA",IndirectY,2,8));
    t[0x74] = Some(("*NOP",ZeroPageX,2,4)); t[0x75] = Some(("ADC",ZeroPageX,2,4));
    t[0x76] = Some(("ROR",ZeroPageX,2,6)); t[0x77] = Some(("*RRA",ZeroPageX,2,6));
    t[0x78] = Some(("SEI",Implied,1,2)); t[0x79] = Some(("ADC",AbsoluteY,3,4));
    t[0x7A] = Some(("*NOP",Implied,1,2));
    t[0x7B] = Some(("*RRA",AbsoluteY,3,7));
    t[0x7C] = Some(("*NOP",AbsoluteX,3,4));
    t[0x7D] = Some(("ADC",AbsoluteX,3,4)); t[0x7E] = Some(("ROR",AbsoluteX,3,7));
    t[0x7F] = Some(("*RRA",AbsoluteX,3,7));
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
    t[0x92] = Some(("KIL",Implied,1,2));
    t[0x97] = Some(("*SAX",ZeroPageY,2,4));
    t[0x93] = Some(("*AXA",IndirectY,2,6));
    t[0x9B] = Some(("*TAS",AbsoluteY,3,5));
    t[0x9C] = Some(("*SAY",AbsoluteX,3,5));
    t[0x9E] = Some(("*XAS",AbsoluteY,3,5));
    t[0x9F] = Some(("*AXA",AbsoluteY,3,5));
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
    t[0xB2] = Some(("KIL",Implied,1,2));
    t[0xB3] = Some(("*LAX",IndirectY,2,5));
    t[0xB4] = Some(("LDY",ZeroPageX,2,4)); t[0xB5] = Some(("LDA",ZeroPageX,2,4));
    t[0xB6] = Some(("LDX",ZeroPageY,2,4));
    t[0xB7] = Some(("*LAX",ZeroPageY,2,4));
    t[0xB8] = Some(("CLV",Implied,1,2));
    t[0xB9] = Some(("LDA",AbsoluteY,3,4));     t[0xBA] = Some(("TSX",Implied,1,2));
    t[0xBB] = Some(("*LAS",AbsoluteY,3,4));
    t[0xBC] = Some(("LDY",AbsoluteX,3,4)); t[0xBD] = Some(("LDA",AbsoluteX,3,4));
    t[0xBE] = Some(("LDX",AbsoluteY,3,4));
    t[0xBF] = Some(("*LAX",AbsoluteY,3,4));
    // 0xCx
    t[0xC0] = Some(("CPY",Immediate,2,2));
    t[0xC1] = Some(("CMP",IndirectX,2,6));
    t[0xC2] = Some(("*NOP",Immediate,2,2));
    t[0xC3] = Some(("*DCP",IndirectX,2,8));
    t[0xC4] = Some(("CPY",ZeroPage,2,3)); t[0xC5] = Some(("CMP",ZeroPage,2,3));
    t[0xC7] = Some(("*DCP",ZeroPage,2,5));
    t[0xC6] = Some(("DEC",ZeroPage,2,5));     t[0xC8] = Some(("INY",Implied,1,2));
    t[0xC9] = Some(("CMP",Immediate,2,2)); t[0xCA] = Some(("DEX",Implied,1,2));
    t[0xCB] = Some(("*SAX",Immediate,2,2));
    t[0xCC] = Some(("CPY",Absolute,3,4)); t[0xCD] = Some(("CMP",Absolute,3,4));
    t[0xCE] = Some(("DEC",Absolute,3,6)); t[0xCF] = Some(("*DCP",Absolute,3,6));
    // 0xDx
    t[0xD0] = Some(("BNE",Relative,2,2)); t[0xD1] = Some(("CMP",IndirectY,2,5));
    t[0xD2] = Some(("KIL",Implied,1,2));
    t[0xD3] = Some(("*DCP",IndirectY,2,8));
    t[0xD4] = Some(("*NOP",ZeroPageX,2,4)); t[0xD5] = Some(("CMP",ZeroPageX,2,4));
    t[0xD6] = Some(("DEC",ZeroPageX,2,6)); t[0xD7] = Some(("*DCP",ZeroPageX,2,6));
    t[0xD8] = Some(("CLD",Implied,1,2)); t[0xD9] = Some(("CMP",AbsoluteY,3,4));
    t[0xDA] = Some(("*NOP",Implied,1,2));
    t[0xDB] = Some(("*DCP",AbsoluteY,3,7));
    t[0xDC] = Some(("*NOP",AbsoluteX,3,4));
    t[0xDD] = Some(("CMP",AbsoluteX,3,4)); t[0xDE] = Some(("DEC",AbsoluteX,3,7));
    t[0xDF] = Some(("*DCP",AbsoluteX,3,7));
    // 0xEx
    t[0xE0] = Some(("CPX",Immediate,2,2)); t[0xE1] = Some(("SBC",IndirectX,2,6));
    t[0xE2] = Some(("*NOP",Immediate,2,2));
    t[0xE3] = Some(("*ISC",IndirectX,2,8));
    t[0xE4] = Some(("CPX",ZeroPage,2,3)); t[0xE5] = Some(("SBC",ZeroPage,2,3));
    t[0xE6] = Some(("INC",ZeroPage,2,5)); t[0xE7] = Some(("*ISC",ZeroPage,2,5));
    t[0xE8] = Some(("INX",Implied,1,2));
    t[0xE9] = Some(("SBC",Immediate,2,2)); t[0xEA] = Some(("NOP",Implied,1,2));
    t[0xEB] = Some(("*SBC",Immediate,2,2));
    t[0xEC] = Some(("CPX",Absolute,3,4)); t[0xED] = Some(("SBC",Absolute,3,4));
    t[0xEE] = Some(("INC",Absolute,3,6)); t[0xEF] = Some(("*ISC",Absolute,3,6));
    // 0xFx
    t[0xF0] = Some(("BEQ",Relative,2,2)); t[0xF1] = Some(("SBC",IndirectY,2,5));
    t[0xF2] = Some(("KIL",Implied,1,2));
    t[0xF3] = Some(("*ISC",IndirectY,2,8));
    t[0xF4] = Some(("*NOP",ZeroPageX,2,4)); t[0xF5] = Some(("SBC",ZeroPageX,2,4));
    t[0xF6] = Some(("INC",ZeroPageX,2,6)); t[0xF7] = Some(("*ISC",ZeroPageX,2,6));
    t[0xF8] = Some(("SED",Implied,1,2)); t[0xF9] = Some(("SBC",AbsoluteY,3,4));
    t[0xFA] = Some(("*NOP",Implied,1,2));
    t[0xFB] = Some(("*ISC",AbsoluteY,3,7));
    t[0xFC] = Some(("*NOP",AbsoluteX,3,4));
    t[0xFD] = Some(("SBC",AbsoluteX,3,4)); t[0xFE] = Some(("INC",AbsoluteX,3,7));
    t[0xFF] = Some(("*ISC",AbsoluteX,3,7));
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
fn exec_slo(cpu: &mut Cpu, memory: &mut impl Bus, addr: u16) {
    let val = memory.read(addr);
    cpu.sr.set_c(val & 0x80 != 0);
    rmw_dummy(cpu, memory, addr, val);
    let shifted = val << 1;
    memory.write(addr, shifted);
    cpu.a |= shifted;
    update_nz!(cpu, cpu.a);
}
fn exec_rla(cpu: &mut Cpu, memory: &mut impl Bus, addr: u16) {
    let val = memory.read(addr);
    let old_c = cpu.sr.c() as u8;
    cpu.sr.set_c(val & 0x80 != 0);
    rmw_dummy(cpu, memory, addr, val);
    let rotated = (val << 1) | old_c;
    memory.write(addr, rotated);
    cpu.a &= rotated;
    update_nz!(cpu, cpu.a);
}
fn exec_sre(cpu: &mut Cpu, memory: &mut impl Bus, addr: u16) {
    let val = memory.read(addr);
    cpu.sr.set_c(val & 1 != 0);
    rmw_dummy(cpu, memory, addr, val);
    let shifted = val >> 1;
    memory.write(addr, shifted);
    cpu.a ^= shifted;
    update_nz!(cpu, cpu.a);
}
fn exec_rra(cpu: &mut Cpu, memory: &mut impl Bus, addr: u16) {
    let val = memory.read(addr);
    let old_c = (cpu.sr.c() as u8) << 7;
    cpu.sr.set_c(val & 1 != 0);
    rmw_dummy(cpu, memory, addr, val);
    let rotated = (val >> 1) | old_c;
    memory.write(addr, rotated);
    exec_adc(cpu, rotated);
}
fn exec_dcp(cpu: &mut Cpu, memory: &mut impl Bus, addr: u16) {
    let val = memory.read(addr).wrapping_sub(1);
    memory.write(addr, val);
    let r = cpu.a.wrapping_sub(val);
    cpu.sr.set_c(cpu.a >= val);
    cpu.sr.set_z(cpu.a == val);
    update_nz!(cpu, r);
}
fn exec_isc(cpu: &mut Cpu, memory: &mut impl Bus, addr: u16) {
    let val = memory.read(addr).wrapping_add(1);
    memory.write(addr, val);
    exec_sbc(cpu, val);
}
fn exec_alr(cpu: &mut Cpu, val: u8) {
    cpu.a &= val;
    cpu.sr.set_c(cpu.a & 1 != 0);
    cpu.a >>= 1;
    update_nz!(cpu, cpu.a);
}
fn exec_arr(cpu: &mut Cpu, val: u8) {
    cpu.a &= val;
    let old_c = cpu.sr.c() as u8;
    let result = (cpu.a >> 1) | (old_c << 7);
    cpu.sr.set_c(cpu.a & 1 != 0);
    cpu.sr.set_v(((cpu.a ^ result) & 0x40) != 0);
    cpu.a = result;
    update_nz!(cpu, cpu.a);
}
fn exec_anc(cpu: &mut Cpu, val: u8) {
    cpu.a &= val;
    cpu.sr.set_c(cpu.a & 0x80 != 0);
    update_nz!(cpu, cpu.a);
}
fn exec_sax_imm(cpu: &mut Cpu, val: u8) {
    let tmp = cpu.a & cpu.x;
    let result = tmp.wrapping_sub(val);
    cpu.sr.set_c(tmp >= val);
    cpu.sr.set_z(tmp == val);
    update_nz!(cpu, result);
    cpu.x = result;
}

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

    // CMOS/65C02 extensions — override NMOS illegal/*NOP opcodes
    // Must check BEFORE undocumented_ops gate (which blocks *-prefixed names)
    if matches!(cpu.config.family, CpuFamily::W65C02 | CpuFamily::R65C02) {
        match opcode {
            0x80 => { let (o,_,_)=fetch_operand(cpu,memory,Relative,op_pc); return exec_branch(cpu,o,true); }
            0x89 => { let (v,_,_)=fetch_operand(cpu,memory,Immediate,op_pc); exec_bit(cpu,v); return 2; }
            0xDA => { cpu.push_stack(cpu.x, memory); return 3; }
            0x5A => { cpu.push_stack(cpu.y, memory); return 3; }
            0xFA => { cpu.x = cpu.pull_stack(memory); update_nz!(cpu, cpu.x); return 4; }
            0x7A => { cpu.y = cpu.pull_stack(memory); update_nz!(cpu, cpu.y); return 4; }
            0x7C => { let base=memory.read_u16(op_pc); let ptr=base.wrapping_add(cpu.x as u16); let a=memory.read_u16(ptr); cpu.pc=a; return 6; }
            // 65C02 zero-page indirect (zp)
            0x12 => { let (v,_,_)=fetch_operand(cpu,memory,ZeroPageIndirect,op_pc); exec_ora(cpu,v); return 5; }
            0x32 => { let (v,_,_)=fetch_operand(cpu,memory,ZeroPageIndirect,op_pc); exec_and(cpu,v); return 5; }
            0x52 => { let (v,_,_)=fetch_operand(cpu,memory,ZeroPageIndirect,op_pc); exec_eor(cpu,v); return 5; }
            0x72 => { let (v,_,_)=fetch_operand(cpu,memory,ZeroPageIndirect,op_pc); exec_adc(cpu,v); return 5; }
            0x92 => { let a=fetch_address(cpu,memory,ZeroPageIndirect,op_pc); memory.write(a, cpu.a); return 5; }
            0xB2 => { let (v,_,_)=fetch_operand(cpu,memory,ZeroPageIndirect,op_pc); exec_lda(cpu,v); return 5; }
            0xD2 => { let (v,_,_)=fetch_operand(cpu,memory,ZeroPageIndirect,op_pc); exec_cmp(cpu,v); return 5; }
             0xF2 => { let (v,_,_)=fetch_operand(cpu,memory,ZeroPageIndirect,op_pc); exec_sbc(cpu,v); return 5; }
             // R65C02 TSB/TRB — override *NOP on 65C02 variants
             0x04 => { let a=fetch_address(cpu,memory,info.mode,op_pc); let v=memory.read(a); cpu.sr.set_z((cpu.a & v) == 0); let r=v|cpu.a; memory.write(a,r); cpu.sr.set_n(r & 0x80 != 0); return 5; }
            0x0C => { let a=fetch_address(cpu,memory,info.mode,op_pc); let v=memory.read(a); cpu.sr.set_z((cpu.a & v) == 0); let r=v|cpu.a; memory.write(a,r); cpu.sr.set_n(r & 0x80 != 0); return 6; }
              0x14 => { let a=fetch_address(cpu,memory,ZeroPage,op_pc); let v=memory.read(a); cpu.sr.set_z((cpu.a & v) == 0); let r=v&!cpu.a; memory.write(a,r); cpu.sr.set_n(r & 0x80 != 0); return 5; }
              0x1C => { let a=fetch_address(cpu,memory,Absolute,op_pc); let v=memory.read(a); cpu.sr.set_z((cpu.a & v) == 0); let r=v&!cpu.a; memory.write(a,r); cpu.sr.set_n(r & 0x80 != 0); return 6; }
            _ => {}
        }
    }
    if matches!(cpu.config.family, CpuFamily::W65C02 | CpuFamily::R65C02) {
        if opcode == 0xCB { cpu.waiting = true; return 2; }
        if opcode == 0xDB { cpu.stopped = true; return 2; }
    }

    // KIL/JAM — halt CPU immediately
    if matches!(opcode, 0x02 | 0x12 | 0x22 | 0x32 | 0x42 | 0x52 | 0x62 | 0x72 | 0x92 | 0xB2 | 0xD2 | 0xF2) {
        cpu.halted = true;
        return 2;
    }

    // CMOS/65C02 extensions — override NMOS illegal/*NOP opcodes
    // Must check BEFORE undocumented_ops gate (which blocks *-prefixed names)
    if matches!(cpu.config.family, CpuFamily::W65C02 | CpuFamily::R65C02) {
        if opcode == 0xCB { cpu.waiting = true; return 2; }
        if opcode == 0xDB { cpu.stopped = true; return 2; }
        match opcode {
            0x80 => { let (o,_,_)=fetch_operand(cpu,memory,Relative,op_pc); return exec_branch(cpu,o,true); }
            0x89 => { let (v,_,_)=fetch_operand(cpu,memory,Immediate,op_pc); exec_bit(cpu,v); return 2; }
            0xDA => { cpu.push_stack(cpu.x, memory); return 3; }
            0x5A => { cpu.push_stack(cpu.y, memory); return 3; }
            0xFA => { cpu.x = cpu.pull_stack(memory); update_nz!(cpu, cpu.x); return 4; }
            0x7A => { cpu.y = cpu.pull_stack(memory); update_nz!(cpu, cpu.y); return 4; }
            0x7C => { let base=memory.read_u16(op_pc); let ptr=base.wrapping_add(cpu.x as u16); let a=memory.read_u16(ptr); cpu.pc=a; return 6; }
            0x12 => { let (v,_,_)=fetch_operand(cpu,memory,ZeroPageIndirect,op_pc); exec_ora(cpu,v); return 5; }
            0x32 => { let (v,_,_)=fetch_operand(cpu,memory,ZeroPageIndirect,op_pc); exec_and(cpu,v); return 5; }
            0x52 => { let (v,_,_)=fetch_operand(cpu,memory,ZeroPageIndirect,op_pc); exec_eor(cpu,v); return 5; }
            0x72 => { let (v,_,_)=fetch_operand(cpu,memory,ZeroPageIndirect,op_pc); exec_adc(cpu,v); return 5; }
            0x92 => { let a=fetch_address(cpu,memory,ZeroPageIndirect,op_pc); memory.write(a, cpu.a); return 5; }
            0xB2 => { let (v,_,_)=fetch_operand(cpu,memory,ZeroPageIndirect,op_pc); exec_lda(cpu,v); return 5; }
            0xD2 => { let (v,_,_)=fetch_operand(cpu,memory,ZeroPageIndirect,op_pc); exec_cmp(cpu,v); return 5; }
            0xF2 => { let (v,_,_)=fetch_operand(cpu,memory,ZeroPageIndirect,op_pc); exec_sbc(cpu,v); return 5; }
            _ => {}
        }
    }
    // If undocumented ops disabled, skip them as NOPs
    if !cpu.config.has_undocumented_ops() && info.name.as_bytes()[0] == b'*' {
        let _ = fetch_operand(cpu, memory, info.mode, op_pc);
        return 2;
    }

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
         // TSB for 65C02 (0x04=zp, 0x0C=abs — *NOP on NMOS)
         0x04 => { if !cpu.config.quirks.stp_available { let _=fetch_operand(cpu,memory,ZeroPage,op_pc); return 3; } let a=fetch_address(cpu,memory,ZeroPage,op_pc); let v=memory.read(a); cpu.sr.set_z((cpu.a & v) == 0); let r=v|cpu.a; memory.write(a,r); cpu.sr.set_n(r & 0x80 != 0); 5 }
         0x0C => { if !cpu.config.quirks.stp_available { let _=fetch_operand(cpu,memory,Absolute,op_pc); return 4; } let a=fetch_address(cpu,memory,Absolute,op_pc); let v=memory.read(a); cpu.sr.set_z((cpu.a & v) == 0); let r=v|cpu.a; memory.write(a,r); cpu.sr.set_n(r & 0x80 != 0); 6 }
         // *SLO — ASL memory then ORA result with A
         0x03|0x07|0x0F|0x13|0x17|0x1B|0x1F => { let a=fetch_address(cpu,memory,info.mode,op_pc); exec_slo(cpu,memory,a); info.cycles }
         // *RLA — ROL memory then AND result with A
         0x23|0x27|0x2F|0x33|0x37|0x3B|0x3F => { let a=fetch_address(cpu,memory,info.mode,op_pc); exec_rla(cpu,memory,a); info.cycles }
         // *SRE — LSR memory then EOR result with A
         0x43|0x47|0x4F|0x53|0x57|0x5B|0x5F => { let a=fetch_address(cpu,memory,info.mode,op_pc); exec_sre(cpu,memory,a); info.cycles }
         // *RRA — ROR memory then ADC result with A
         0x63|0x67|0x6F|0x73|0x77|0x7B|0x7F => { let a=fetch_address(cpu,memory,info.mode,op_pc); exec_rra(cpu,memory,a); info.cycles }
         // *DCP — DEC memory then CMP result with A
         0xC3|0xC7|0xCF|0xD3|0xD7|0xDF => { let a=fetch_address(cpu,memory,info.mode,op_pc); exec_dcp(cpu,memory,a); info.cycles }
         // *ISC — INC memory then SBC result from A
         0xE3|0xE7|0xEF|0xF3|0xF7|0xFB|0xFF => { let a=fetch_address(cpu,memory,info.mode,op_pc); exec_isc(cpu,memory,a); info.cycles }
         // *ANC — AND #imm, then bit7→C
         0x0B|0x2B => { let (v,_,_)=fetch_operand(cpu,memory,Immediate,op_pc); exec_anc(cpu,v); 2 }
         // *ALR — AND #imm, then LSR A
         0x4B => { let (v,_,_)=fetch_operand(cpu,memory,Immediate,op_pc); exec_alr(cpu,v); 2 }
         // *ARR — AND #imm, then ROR A
         0x6B => { let (v,_,_)=fetch_operand(cpu,memory,Immediate,op_pc); exec_arr(cpu,v); 2 }
         // *XAA — TXA + AND #imm
         0x8B => { let (v,_,_)=fetch_operand(cpu,memory,Immediate,op_pc); cpu.a = cpu.x & v; update_nz!(cpu, cpu.a); 2 }
         // *OAL — ORA #$EE + AND #imm
         0xAB => { let (v,_,_)=fetch_operand(cpu,memory,Immediate,op_pc); cpu.a = (cpu.a | 0xEE) & v; cpu.x = cpu.a; update_nz!(cpu, cpu.a); 2 }
         // *SAX Immediate — A&X - #imm → X
         0xCB => { let (v,_,_)=fetch_operand(cpu,memory,Immediate,op_pc); exec_sax_imm(cpu,v); 2 }
         // *LAS AbsoluteY — mem & SP → A, X, SP
         0xBB => { let a=fetch_address(cpu,memory,info.mode,op_pc); let v=memory.read(a); cpu.a=cpu.sp as u8 & v; cpu.x=cpu.a; cpu.sp=cpu.a; update_nz!(cpu,cpu.a); info.cycles }
         // *TAS AbsoluteY — A&X → SP, then SP&(addr_hi+1) → memory
         0x9B => { let a=fetch_address(cpu,memory,info.mode,op_pc); let sp = cpu.a & cpu.x; cpu.sp = sp; memory.write(a, sp & ((a >> 8) as u8 + 1)); info.cycles }
         // *SAY AbsoluteX — Y & (addr_hi+1) → memory
         0x9C => { let a=fetch_address(cpu,memory,info.mode,op_pc); memory.write(a, cpu.y & ((a >> 8) as u8 + 1)); info.cycles }
         // *XAS AbsoluteY — X & (addr_hi+1) → memory
         0x9E => { let a=fetch_address(cpu,memory,info.mode,op_pc); memory.write(a, cpu.x & ((a >> 8) as u8 + 1)); info.cycles }
         // *AXA AbsoluteY / IndirectY — A & X & (addr_hi+1) → memory
         0x9F|0x93 => { let a=fetch_address(cpu,memory,info.mode,op_pc); memory.write(a, cpu.a & cpu.x & ((a >> 8) as u8 + 1)); info.cycles }
         // *NOP Immediate — read byte, 2 cycles
         0x80 | 0x82 | 0x89 | 0xC2 | 0xE2 => { let _ = fetch_operand(cpu,memory,Immediate,op_pc); 2 }
         // *NOP ZeroPage — 3 cycles
         0x04 | 0x44 | 0x64 => { let _ = fetch_operand(cpu,memory,ZeroPage,op_pc); 3 }
         // *NOP ZeroPage,X — 4 cycles
         0x14 | 0x34 | 0x54 | 0x74 | 0xD4 | 0xF4 => { let _ = fetch_operand(cpu,memory,ZeroPageX,op_pc); 4 }
         // *NOP Absolute — 4 cycles
         0x0C => { let _ = fetch_operand(cpu,memory,Absolute,op_pc); 4 }
         // *NOP Absolute,X — 4 cycles (+1 page cross)
         0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => { let (_,_,x)=fetch_operand(cpu,memory,AbsoluteX,op_pc); 4 + if x { 1 } else { 0 } }
        _ => info.cycles
    }
}

#[cfg(test)]
#[path = "tests/instruction.rs"]
mod tests;
