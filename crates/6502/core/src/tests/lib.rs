use super::*;

#[test]
fn test_reset() {
    let mut emu = Emulator::new();
    emu.cpu.a = 0xFF;
    emu.cpu.pc = 0x0000;
    emu.set_reset_vector(0x8000);

    emu.reset();
    assert_eq!(emu.cpu.a, 0);
    assert_eq!(emu.cpu.pc, 0x8000);
}

#[test]
fn test_reset_preserves_vector_memory() {
    let mut emu = Emulator::new();
    emu.set_reset_vector(0xABCD);
    emu.reset();
    assert_eq!(emu.get_memory(0xFFFC), 0xCD);
    assert_eq!(emu.get_memory(0xFFFD), 0xAB);
}

#[test]
fn test_soft_reset() {
    let mut emu = Emulator::new();
    emu.cpu.a = 0xFF;
    emu.cpu.pc = 0x0000;
    emu.set_reset_vector(0x8000);

    emu.reset();
    assert_eq!(emu.cpu.a, 0);

    assert_eq!(emu.cpu.pc, 0x8000);
}

#[test]
fn test_register_access() {
    let mut emu = Emulator::new();
    emu.set_register_a(0x42);
    assert_eq!(emu.get_register_a(), 0x42);

    emu.set_register_x(0x13);
    assert_eq!(emu.get_register_x(), 0x13);

    emu.set_register_y(0x55);
    assert_eq!(emu.get_register_y(), 0x55);

    emu.set_register_pc(0xABCD);
    assert_eq!(emu.cpu.pc, 0xABCD);

    emu.set_register_sp(0xFE);
    assert_eq!(emu.cpu.sp, 0xFE);
}

#[test]
fn test_status_flags() {
    let mut emu = Emulator::new();

    assert!(!emu.get_status_n());
    assert!(!emu.get_status_v());
    assert!(!emu.get_status_z());
    assert!(!emu.get_status_c());
    assert!(emu.get_status_i()); // I=1 after reset
    assert!(!emu.get_status_d());

    emu.cpu.sr.set_n(true);
    assert!(emu.get_status_n());

    emu.cpu.sr.set_z(true);
    assert!(emu.get_status_z());

    emu.cpu.sr.set_c(true);
    assert!(emu.get_status_c());
}

#[test]
fn test_memory_access() {
    let mut emu = Emulator::new();

    emu.set_memory(0x1234, 0xAB);
    assert_eq!(emu.get_memory(0x1234), 0xAB);

    emu.set_memory_u16(0x1000, 0x1234);
    assert_eq!(emu.get_memory_u16(0x1000), 0x1234);
}

#[test]
fn test_trigger_nmi() {
    let mut emu = Emulator::new();
    emu.memory.set_nmi_vector(0x9000);
    emu.memory.write(0x9000, 0xEA); // NOP at NMI handler

    emu.cpu.pc = 0x8000;
    emu.trigger_nmi();

    assert_eq!(emu.cpu.pc, 0x9000);
    assert!(emu.cpu.sr.i()); // I flag should be set
}

#[test]
fn test_trigger_irq() {
    let mut emu = Emulator::new();
    emu.memory.set_irq_vector(0xA000);
    emu.memory.write(0xA000, 0xEA); // NOP at IRQ handler
    emu.cpu.sr.set_i(false); // Enable interrupts

    emu.cpu.pc = 0x8000;
    let accepted = emu.trigger_irq();

    assert!(accepted);
    assert_eq!(emu.cpu.pc, 0xA000);
    assert!(emu.cpu.sr.i()); // I flag should be set
}

#[test]
fn test_irq_blocked() {
    let mut emu = Emulator::new();
    emu.cpu.sr.set_i(true); // Disable interrupts

    let accepted = emu.trigger_irq();
    assert!(!accepted);
}

#[test]
fn test_save_load_state() {
    let mut emu = Emulator::new();
    emu.cpu.a = 0x42;
    emu.cpu.x = 0x13;
    emu.cpu.pc = 0xABCD;
    emu.memory.write(0x1000, 0xFF);

    let state = emu.save_state();

    let mut emu2 = Emulator::new();
    emu2.load_state(&state).unwrap();

    assert_eq!(emu2.cpu.a, emu.cpu.a);
    assert_eq!(emu2.cpu.x, emu.cpu.x);
    assert_eq!(emu2.cpu.pc, emu.cpu.pc);
    assert_eq!(emu2.memory.read(0x1000), 0xFF);
}

#[test]
fn test_variants() {
    let emu_nmos = Emulator::new_nmos();
    assert_eq!(emu_nmos.get_variant(), "nmos6502");

    let emu_cmos = Emulator::new_cmos();
    assert_eq!(emu_cmos.get_variant(), "w65c02");
}

#[test]
fn test_set_variant() {
    let mut emu = Emulator::new();
    emu.set_variant("w65c02").unwrap();
    assert_eq!(emu.get_variant(), "w65c02");

    emu.set_variant("nmos6502").unwrap();
    assert_eq!(emu.get_variant(), "nmos6502");
}

#[test]
fn test_run() {
    let mut emu = Emulator::new();
    for addr in 0x8000..0x8100 {
        emu.memory.write(addr, 0xEA);
    }

    let cycles = emu.run(100);
    assert!(cycles > 0);
    assert!(emu.cpu.pc > 0x8000);
}

#[test]
fn test_trigger_brk() {
    let mut emu = Emulator::new();
    emu.memory.set_irq_vector(0xA000);
    emu.memory.write(0xA000, 0xEA);
    emu.cpu.pc = 0x8000;
    emu.trigger_brk();
    assert_eq!(emu.cpu.pc, 0xA000);
    assert!(emu.cpu.sr.i());
}

#[test]
fn test_disassemble_known_opcode() {
    let mut emu = Emulator::new();
    emu.set_memory(0x8000, 0xA9); // LDA #imm
    emu.set_memory(0x8001, 0x42);
    let s = emu.disassemble(0x8000);
    assert_eq!(s, "LDA #$42");
}

#[test]
fn test_disassemble_illegal_opcode_shows_dot_byte() {
    let mut emu = Emulator::new();
    emu.set_memory(0x8000, 0x03); // *SLO
    let s = emu.disassemble(0x8000);
    assert_eq!(s, ".byte $03");
}

#[test]
fn test_disassemble_kil_shows_name() {
    let mut emu = Emulator::new();
    emu.set_memory(0x8000, 0x02); // KIL
    let s = emu.disassemble(0x8000);
    assert_eq!(s, "KIL");
}

#[test]
fn test_load_rom_default() {
    let mut emu = Emulator::new();
    let rom = vec![0xEA, 0xEA];
    emu.load_rom_default(&rom);
    assert_eq!(emu.get_memory(0x8000), 0xEA);
}

#[test]
fn test_set_variant_all() {
    let mut emu = Emulator::new();
    for variant in &["nmos", "nmos6502", "cmos", "w65c02", "nes", "ricoh2a03", "r65c02", "c64"] {
        assert!(emu.set_variant(variant).is_ok(), "variant {}", variant);
    }
    assert!(emu.set_variant("invalid").is_err());
}
