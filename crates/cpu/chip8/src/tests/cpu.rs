use crate::cpu::Cpu;

#[test]
fn test_new() {
    let cpu = Cpu::new();
    assert_eq!(cpu.pc, 0x200);
    assert_eq!(cpu.sp, 0);
}

#[test]
fn test_reset() {
    let mut cpu = Cpu::new();
    cpu.v[0] = 0xFF;
    cpu.reset();
    assert_eq!(cpu.pc, 0x200);
    assert_eq!(cpu.v[0], 0);
}

#[test]
fn test_stack() {
    let mut cpu = Cpu::new();
    cpu.stack_push(0x1234);
    assert_eq!(cpu.sp, 1);
    assert_eq!(cpu.stack_pop(), 0x1234);
    assert_eq!(cpu.sp, 0);

    cpu.stack_push(0xAAAA);
    cpu.stack_push(0xBBBB);
    assert_eq!(cpu.stack_pop(), 0xBBBB);
    assert_eq!(cpu.stack_pop(), 0xAAAA);
}

#[test]
fn test_timers() {
    let mut cpu = Cpu::new();
    cpu.delay = 3;
    cpu.sound = 2;
    cpu.tick_timers();
    assert_eq!(cpu.delay, 2);
    assert_eq!(cpu.sound, 1);
}

#[test]
fn test_timers_dont_underflow() {
    let mut cpu = Cpu::new();
    cpu.tick_timers();
    assert_eq!(cpu.delay, 0);
    assert_eq!(cpu.sound, 0);
}
