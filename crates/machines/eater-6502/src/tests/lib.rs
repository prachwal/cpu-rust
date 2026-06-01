use super::*;

fn make_machine() -> Eater6502 {
    let rom = crate::rom::generate_monitor();
    Eater6502::new(rom)
}

fn drain_prompt(machine: &mut Eater6502) {
    // Monitor sends '>' after init; consume it before testing echo
    for _ in 0..2000 { machine.tick(); }
    let tx = machine.bus.read_transmitted();
    if tx == Some(b'>') {
        // consume the prompt
    }
    // also drain any remaining prompt bytes
    while machine.bus.drain_tx().len() > 0 {}
}

#[test]
fn test_eater_6502_boots() {
    let mut machine = make_machine();
    assert!(machine.get_pc() >= 0x8000);
    for _ in 0..5000 { machine.tick(); }
    assert!(machine.get_pc() >= 0x8000);
}

#[test]
fn test_eater_6502_memory() {
    let mut machine = make_machine();
    machine.bus.write(0x0200, 0x55);
    assert_eq!(machine.bus.read(0x0200), 0x55);
    assert_ne!(machine.bus.read(0x8000), 0xFF, "ROM should contain code");
}

#[test]
fn test_eater_6502_acia_echo() {
    let mut machine = make_machine();
    drain_prompt(&mut machine);

    machine.bus.receive_byte(0x41);
    for _ in 0..5000 { machine.tick(); }

    let tx = machine.bus.read_transmitted();
    assert_eq!(tx, Some(0x41), "Should echo 'A'");
}

#[test]
fn test_eater_6502_acia_echo_cr_lf() {
    let mut machine = make_machine();
    drain_prompt(&mut machine);

    machine.bus.receive_byte(0x0D);
    for _ in 0..10000 { machine.tick(); }

    let out = machine.bus.drain_tx();
    assert!(out.len() >= 2, "Should have at least 2 bytes, got {out:?}");
    assert_eq!(out[0], 0x0D);
    assert_eq!(out[1], 0x0A);
}

#[test]
fn test_eater_6502_acia_multiple_echo() {
    let mut machine = make_machine();
    drain_prompt(&mut machine);

    for &ch in b"HELLO" {
        machine.bus.receive_byte(ch);
        for _ in 0..5000 { machine.tick(); }
        let tx = machine.bus.read_transmitted();
        assert_eq!(tx, Some(ch));
    }
}
