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
