use super::*;
use chip8_cpu::cpu;
use chip8_cpu::instruction;
use cpu_display::Display;
use cpu_keyboard::Keyboard;

fn make_cpu() -> cpu::Cpu { cpu::Cpu::new() }
fn make_mem() -> memory::Memory { memory::Memory::new() }
fn make_disp() -> Display { Display::new(64, 32) }
fn make_kbd() -> Keyboard { Keyboard::new() }

#[test]
fn test_tick_executes_instruction() {
    let mut cpu = make_cpu();
    let mut mem = make_mem();
    let mut disp = make_disp();
    let kbd = make_kbd();
    let q = instruction::Quirks { shift_vy: false, memory_inc_i: false, vf_reset: true };

    mem.write(0x200, 0x6A); mem.write(0x201, 0x42);
    cpu.pc = 0x200;

    let opcode = (mem.read(cpu.pc) as u16) << 8 | mem.read(cpu.pc + 1) as u16;
    cpu.pc += 2;
    instruction::execute(opcode, &mut cpu, &mut mem, &mut disp, &kbd, &q);

    assert_eq!(cpu.v[0xA], 0x42);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn test_tick_timers() {
    let mut cpu = make_cpu();
    cpu.delay = 3;
    cpu.sound = 2;
    cpu.tick_timers();
    assert_eq!(cpu.delay, 2);
    assert_eq!(cpu.sound, 1);
}

#[test]
fn test_reset() {
    let mut cpu = make_cpu();
    cpu.pc = 0x500;
    cpu.v[0] = 42;
    cpu.reset();
    assert_eq!(cpu.pc, 0x200);
    assert_eq!(cpu.v[0], 0);
}

#[test]
fn test_load_rom_sets_memory() {
    let mut mem = make_mem();
    let rom = vec![0x00, 0xE0, 0x6A, 0x42];
    mem.load_rom(&rom, 0x200);
    assert_eq!(mem.read(0x200), 0x00);
    assert_eq!(mem.read(0x203), 0x42);
}

#[test]
fn test_keyboard_roundtrip() {
    let mut kbd = make_kbd();
    kbd.press(0xF);
    assert!(kbd.is_pressed(0xF));
    kbd.release(0xF);
    assert!(!kbd.is_pressed(0xF));
}

#[test]
fn test_display_buffer_size() {
    let disp = make_disp();
    assert_eq!(disp.buffer_len(), 2048);
}

#[test]
fn test_get_sound() {
    let cpu = make_cpu();
    assert!(!(cpu.sound > 0));
}
