use crate::cpu::Cpu;
use crate::instruction::{execute, Quirks};
use cpu_display::Display;
use cpu_keyboard::Keyboard;
use cpu_memory::Memory as FlatMemory;

fn setup() -> (Cpu, FlatMemory, Display, Keyboard, Quirks) {
    (
        Cpu::new(),
        FlatMemory::new(4096),
        Display::new(64, 32),
        Keyboard::new(),
        Quirks { shift_vy: false, memory_inc_i: false, vf_reset: true },
    )
}

#[test]
fn test_00e0_cls() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    display.set_pixel(0, 0, 1);
    assert!(display.get_pixel(0, 0) != 0);
    execute(0x00E0, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(display.get_pixel(0, 0), 0);
}

#[test]
fn test_00ee_ret() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.stack_push(0x234);
    cpu.pc = 0x456;
    execute(0x00EE, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.pc, 0x234);
}

#[test]
fn test_1nnn_jp() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    execute(0x1345, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.pc, 0x345);
}

#[test]
fn test_2nnn_call() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.pc = 0x300;
    execute(0x2500, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.pc, 0x500);
    assert_eq!(cpu.stack_pop(), 0x300);
}

#[test]
fn test_3xnn_se() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.v[0] = 42; cpu.pc = 0x200;
    execute(0x302A, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn test_3xnn_se_no_skip() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.v[0] = 41; cpu.pc = 0x200;
    execute(0x302A, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.pc, 0x200);
}

#[test]
fn test_4xnn_sne() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.v[0] = 42; cpu.pc = 0x200;
    execute(0x4000, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn test_5xy0_se() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.v[0] = 5; cpu.v[1] = 5; cpu.pc = 0x200;
    execute(0x5010, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn test_6xnn_ld() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    execute(0x6A42, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.v[0xA], 0x42);
}

#[test]
fn test_7xnn_add() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.v[3] = 10;
    execute(0x7305, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.v[3], 15);
}

#[test]
fn test_8xy0_ld() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.v[1] = 99;
    execute(0x8010, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.v[0], 99);
}

#[test]
fn test_8xy1_or() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.v[0] = 0xF0; cpu.v[1] = 0x0F;
    execute(0x8011, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.v[0], 0xFF); assert_eq!(cpu.v[0xF], 0);
}

#[test]
fn test_8xy2_and() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.v[0] = 0xFF; cpu.v[1] = 0x0F;
    execute(0x8012, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.v[0], 0x0F);
}

#[test]
fn test_8xy3_xor() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.v[0] = 0xFF; cpu.v[1] = 0x0F;
    execute(0x8013, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.v[0], 0xF0);
}

#[test]
fn test_8xy4_add_carry() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.v[0] = 0xFF; cpu.v[1] = 0x01;
    execute(0x8014, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.v[0], 0x00); assert_eq!(cpu.v[0xF], 1);
}

#[test]
fn test_8xy4_add_no_carry() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.v[0] = 0x01; cpu.v[1] = 0x02;
    execute(0x8014, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.v[0], 0x03); assert_eq!(cpu.v[0xF], 0);
}

#[test]
fn test_8xy5_sub_borrow() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.v[0] = 0x01; cpu.v[1] = 0xFF;
    execute(0x8015, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.v[0], 0x02); assert_eq!(cpu.v[0xF], 0);
}

#[test]
fn test_8xy5_sub_no_borrow() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.v[0] = 0x05; cpu.v[1] = 0x03;
    execute(0x8015, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.v[0], 0x02); assert_eq!(cpu.v[0xF], 1);
}

#[test]
fn test_8xy6_shr() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.v[0] = 0x05;
    execute(0x8016, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.v[0], 0x02); assert_eq!(cpu.v[0xF], 1);
}

#[test]
fn test_8xy7_subn() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.v[0] = 0x03; cpu.v[1] = 0x08;
    execute(0x8017, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.v[0], 0x05); assert_eq!(cpu.v[0xF], 1);
}

#[test]
fn test_8xye_shl() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.v[0] = 0x80;
    execute(0x801E, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.v[0], 0x00); assert_eq!(cpu.v[0xF], 1);
}

#[test]
fn test_9xy0_sne() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.v[0] = 1; cpu.v[1] = 2; cpu.pc = 0x200;
    execute(0x9010, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn test_annn_ld_i() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    execute(0xA345, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.i, 0x345);
}

#[test]
fn test_bnnn_jp_v0() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.v[0] = 0x10;
    execute(0xB200, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.pc, 0x210);
}

#[test]
fn test_cxnn_rnd() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    execute(0xC0FF, &mut cpu, &mut mem, &mut display, &kb, &config);
}

#[test]
fn test_dxyn_draw() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.i = 0x200;
    mem.write(0x200, 0xFF);
    cpu.v[0] = 10; cpu.v[1] = 5;
    execute(0xD011, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert!(display.get_pixel(10, 5) != 0);
    assert!(display.get_pixel(17, 5) != 0);
    assert!(display.get_pixel(18, 5) == 0);
}

#[test]
fn test_ex9e_skip_key() {
    let (mut cpu, mut mem, mut display, mut kb, config) = setup();
    cpu.v[0] = 0xA; kb.press(0xA); cpu.pc = 0x200;
    execute(0xE09E, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn test_exa1_skip_no_key() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.v[0] = 0xA; cpu.pc = 0x200;
    execute(0xE0A1, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn test_fx07_ld_dt() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.delay = 42;
    execute(0xF007, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.v[0], 42);
}

#[test]
fn test_fx0a_wait_key() {
    let (mut cpu, mut mem, mut display, mut kb, config) = setup();
    cpu.pc = 0x202;
    execute(0xF00A, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.pc, 0x200);
    kb.press(0x7); cpu.pc += 2;
    execute(0xF00A, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.v[0], 7); assert_eq!(cpu.pc, 0x202);
}

#[test]
fn test_fx15_ld_dt() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.v[0] = 30;
    execute(0xF015, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.delay, 30);
}

#[test]
fn test_fx18_ld_st() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.v[1] = 15;
    execute(0xF118, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.sound, 15);
}

#[test]
fn test_fx1e_add_i() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.i = 0x100; cpu.v[0] = 0x50;
    execute(0xF01E, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.i, 0x150);
}

#[test]
fn test_fx29_font() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.v[0] = 0x0A;
    execute(0xF029, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.i, 0x050 + 0x0A * 5);
}

#[test]
fn test_fx33_bcd() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.v[0] = 123; cpu.i = 0x300;
    execute(0xF033, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(mem.read(0x300), 1);
    assert_eq!(mem.read(0x301), 2);
    assert_eq!(mem.read(0x302), 3);
}

#[test]
fn test_fx55_store() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    cpu.v[0] = 0x10; cpu.v[1] = 0x20; cpu.v[2] = 0x30; cpu.i = 0x400;
    execute(0xF255, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(mem.read(0x400), 0x10);
    assert_eq!(mem.read(0x401), 0x20);
    assert_eq!(mem.read(0x402), 0x30);
    assert_eq!(cpu.i, 0x400);
}

#[test]
fn test_fx65_load() {
    let (mut cpu, mut mem, mut display, kb, config) = setup();
    mem.write(0x400, 0xAA); mem.write(0x401, 0xBB); mem.write(0x402, 0xCC);
    cpu.i = 0x400;
    execute(0xF265, &mut cpu, &mut mem, &mut display, &kb, &config);
    assert_eq!(cpu.v[0], 0xAA);
    assert_eq!(cpu.v[1], 0xBB);
    assert_eq!(cpu.v[2], 0xCC);
    assert_eq!(cpu.i, 0x400);
}

#[test]
fn test_fx55_store_inc_i() {
    let (mut cpu, mut mem, mut display, kb, mut quirks) = setup();
    quirks.memory_inc_i = true;
    cpu.v[0] = 0x10; cpu.i = 0x400;
    execute(0xF055, &mut cpu, &mut mem, &mut display, &kb, &quirks);
    assert_eq!(cpu.i, 0x401);
}

#[test]
fn test_shift_vy_vs_vx() {
    let (mut cpu, mut mem, mut display, kb, mut quirks) = setup();
    cpu.v[0] = 0x03; cpu.v[1] = 0x80;
    execute(0x8016, &mut cpu, &mut mem, &mut display, &kb, &quirks);
    assert_eq!(cpu.v[0], 0x01); assert_eq!(cpu.v[0xF], 1);

    quirks.shift_vy = true;
    cpu.v[0] = 99; cpu.v[1] = 0x05;
    execute(0x8016, &mut cpu, &mut mem, &mut display, &kb, &quirks);
    assert_eq!(cpu.v[0], 0x02); assert_eq!(cpu.v[0xF], 1);
}
