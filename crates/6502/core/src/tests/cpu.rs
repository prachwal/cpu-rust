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
