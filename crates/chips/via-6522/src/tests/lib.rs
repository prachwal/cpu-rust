use super::*;

// ── Register read/write ──

#[test]
fn test_read_write_regs() {
    let mut via = Via6522::new();
    via.write(3, 0xFF); // DDRA = all output
    via.write(1, 0x42);
    assert_eq!(via.read(1), 0x42);
    via.write(2, 0xF0);
    assert_eq!(via.read(2), 0xF0);
}

#[test]
fn test_reset_state() {
    let mut via = Via6522::new();
    assert_eq!(via.read(0), 0);
    assert_eq!(via.read(1), 0);
    assert_eq!(via.read(2), 0);
    assert_eq!(via.read(3), 0);
    assert_eq!(via.read(11), 0);
    assert_eq!(via.read(12), 0);
    assert_eq!(via.read(13), 0);
    assert_eq!(via.read(14), 0x80);
    assert!(!via.irq);
}

// ── IFR/IER ──

#[test]
fn test_ier_set_clear() {
    let mut via = Via6522::new();
    via.write(14, 0xC0); // set T1
    assert_eq!(via.read(14) & 0x40, 0x40);
    via.write(14, 0x40); // clear T1
    assert_eq!(via.read(14) & 0x40, 0);
}

#[test]
fn test_ifr_write_clears_bits() {
    let mut via = Via6522::new();
    via.ifr_set(IRQ_T1 | IRQ_CA1);
    assert!(via.ifr & IRQ_T1 != 0);
    via.write(13, IRQ_T1); // IFR write clears T1
    assert_eq!(via.ifr & IRQ_T1, 0);
    assert!(via.ifr & IRQ_CA1 != 0); // CA1 preserved
}

#[test]
fn test_irq_asserted_when_enabled_flag_pending() {
    let mut via = Via6522::new();
    via.write(14, 0xC0); // enable T1
    via.ifr_set(IRQ_T1);
    assert!(via.irq);
    assert!(via.read(13) & 0x80 != 0); // IFR bit 7 set
}

#[test]
fn test_irq_deasserted_when_disabled() {
    let mut via = Via6522::new();
    via.write(14, 0xC0);
    via.ifr_set(IRQ_T1);
    assert!(via.irq);
    via.write(14, 0x40); // disable T1
    assert!(!via.irq);
    // flag still set but IRQ not asserted
    assert!(via.ifr & IRQ_T1 != 0);
}

// ── Timer 1 one-shot ──

#[test]
fn test_timer1_oneshot() {
    let mut via = Via6522::new();
    via.write(14, 0xC0);
    via.write(4, 0x05);
    via.write(5, 0x00);
    assert_eq!(via.t1_counter, 5);
    via.tick(4);
    assert!(!via.irq);
    via.tick(1);
    assert!(via.irq);
}

#[test]
fn test_timer1_oneshot_stops_after_underflow() {
    let mut via = Via6522::new();
    via.write(14, 0xC0);
    via.write(4, 0x03);
    via.write(5, 0x00);
    via.tick(3);
    assert!(via.irq);
    via.tick(10); // should not wrap around
    assert_eq!(via.t1_counter, 0);
}

#[test]
fn test_timer1_read_high_clears_flag() {
    let mut via = Via6522::new();
    via.write(14, 0xC0);
    via.write(4, 0x01);
    via.write(5, 0x00);
    via.tick(1);
    assert!(via.irq);
    let _ = via.read(5); // T1C-H read
    assert!(!via.irq);
}

// ── Timer 1 free-run ──

#[test]
fn test_timer1_freerun() {
    let mut via = Via6522::new();
    via.write(14, 0xC0);
    via.write(11, 0x80); // ACR bit 7 = free-run
    via.write(4, 0x03);
    via.write(5, 0x00);
    via.tick(3);
    assert!(via.irq);
    assert_eq!(via.t1_counter, 3); // reloaded
    via.ifr_clear(IRQ_T1);
    assert!(!via.irq);
    via.tick(3);
    assert!(via.irq);
}

// ── Timer 2 timed mode ──

#[test]
fn test_timer2_timed() {
    let mut via = Via6522::new();
    via.write(14, 0xA0); // enable T2
    via.write(8, 0x05);
    via.write(9, 0x00);
    assert_eq!(via.t2_counter, 5);
    via.tick(4);
    assert!(!via.irq);
    via.tick(1);
    assert!(via.irq);
}

#[test]
fn test_timer2_read_high_clears_flag() {
    let mut via = Via6522::new();
    via.write(14, 0xA0);
    via.write(8, 0x01);
    via.write(9, 0x00);
    via.tick(1);
    assert!(via.irq);
    let _ = via.read(9);
    assert!(!via.irq);
}

// ── Port A/B with DDR masking ──

#[test]
fn test_port_output() {
    let mut via = Via6522::new();
    via.write(3, 0xF0); // DDRA: high nibble output
    via.write(1, 0xFF); // ORA
    assert_eq!(via.port_a_output(), 0xF0);
    via.write(0, 0xFF); // ORB
    via.write(2, 0x0F); // DDRB: low nibble output
    assert_eq!(via.port_b_output(), 0x0F);
}

#[test]
fn test_port_read_with_ddr() {
    let mut via = Via6522::new();
    via.write(3, 0xF0);
    via.write(1, 0xFF);
    via.set_input_a(0x0F);
    assert_eq!(via.read(1), 0xFF);
}

// ── CA1 edge ──

#[test]
fn test_ca1_falling_sets_flag() {
    let mut via = Via6522::new();
    via.write(14, 0x82); // enable CA1
    via.set_ca1(true);
    via.set_ca1(false); // falling (default)
    assert!(via.irq);
}

#[test]
fn test_ca1_rising_sets_flag() {
    let mut via = Via6522::new();
    via.write(14, 0x82);
    via.write(12, 0x01); // PCR: CA1 rising
    via.set_ca1(false);
    via.set_ca1(true);
    assert!(via.irq);
}

// ── CB1 edge ──

#[test]
fn test_cb1_falling_sets_flag() {
    let mut via = Via6522::new();
    via.write(14, 0x90); // enable CB1
    via.set_cb1(true);
    via.set_cb1(false);
    assert!(via.irq);
}

// ── CA2 output manual ──

#[test]
fn test_ca2_manual_low() {
    let mut via = Via6522::new();
    via.write(12, 0x0A); // PCR: CA2 manual low
    assert!(!via.ca2_output());
}

#[test]
fn test_ca2_manual_high() {
    let mut via = Via6522::new();
    via.write(12, 0x0C); // PCR: CA2 manual high
    assert!(via.ca2_output());
}

// ── CA2 handshake ──

#[test]
fn test_ca2_handshake_strobes_on_port_a_access() {
    let mut via = Via6522::new();
    via.write(12, 0x0E); // PCR: CA2 handshake output
    assert!(via.ca2_output());
    via.write(1, 0xFF); // write PORTA → strobe
    assert!(!via.ca2_output());
}

// ── CB2 handshake ──

#[test]
fn test_cb2_handshake_strobes_on_port_b_access() {
    let mut via = Via6522::new();
    via.write(12, 0xE0); // PCR: CB2 handshake output
    assert!(via.cb2_output());
    via.write(0, 0xFF); // write PORTB → strobe
    assert!(!via.cb2_output());
}

// ── PB7 output ──

#[test]
fn test_pb7_normal_mode() {
    let mut via = Via6522::new();
    via.write(0, 0x80); // ORB bit 7
    via.write(2, 0x80); // DDRB bit 7 = output
    assert!(via.pb7_output());
}

#[test]
fn test_pb7_timer_mode_toggles() {
    let mut via = Via6522::new();
    via.write(11, 0x80); // ACR: T1 PB7 output
    via.write(4, 0x02);
    via.write(5, 0x00);
    assert!(!via.pb7_output());
    via.tick(2); // underflow
    assert!(via.pb7_output()); // toggled
}
