use super::*;

// ── Register select ──

#[test]
fn test_new_reset_state() {
    let mut pia = Pia6821::new();
    assert_eq!(pia.read(1, 0, 0), 0);
    assert_eq!(pia.read(3, 0, 0), 0);
    assert_eq!(pia.output_a(), 0);
    assert_eq!(pia.output_b(), 0);
    assert!(!pia.irq_a());
    assert!(!pia.irq_b());
}

#[test]
fn test_cra_bit2_selects_ora_vs_ddra() {
    let mut pia = Pia6821::new();
    pia.ddra = 0xAA;
    assert_eq!(pia.read(0, 0, 0), 0xAA);
    pia.cra |= 0x04;
    pia.ddra = 0xFF; // all output
    pia.ora = 0x55;
    assert_eq!(pia.read(0, 0x00, 0), 0x55);
}

#[test]
fn test_crb_bit2_selects_orb_vs_ddrb() {
    let mut pia = Pia6821::new();
    pia.ddrb = 0xF0;
    assert_eq!(pia.read(2, 0, 0), 0xF0);
    pia.crb |= 0x04;
    pia.ddrb = 0xFF;
    pia.orb = 0x0F;
    assert_eq!(pia.read(2, 0, 0x55), 0x0F);
}

#[test]
fn test_read_control_register() {
    let mut pia = Pia6821::new();
    pia.write(1, 0xA5);
    assert_eq!(pia.read(1, 0, 0), 0xA5 & 0x3F);
    pia.cra |= 0xC0;
    assert_eq!(pia.read(1, 0, 0) & 0xC0, 0xC0);
}

// ── Port data with DDR ──

#[test]
fn test_ddr_00_reads_input() {
    let mut pia = Pia6821::new();
    pia.cra |= 0x04;
    pia.ora = 0xFF;
    assert_eq!(pia.read(0, 0x00, 0), 0x00);
}

#[test]
fn test_ddr_ff_drives_output() {
    let mut pia = Pia6821::new();
    pia.cra |= 0x04;
    pia.ora = 0xAB;
    pia.ddra = 0xFF;
    assert_eq!(pia.read(0, 0x00, 0), 0xAB);
    assert_eq!(pia.output_a(), 0xAB);
}

#[test]
fn test_ddr_mixed_read() {
    let mut pia = Pia6821::new();
    pia.cra |= 0x04;
    pia.ddra = 0xF0;
    pia.ora = 0xFF;
    assert_eq!(pia.read(0, 0x0F, 0), 0xFF);
}

#[test]
fn test_output_latch_preserved_across_ddr_change() {
    let mut pia = Pia6821::new();
    pia.cra |= 0x04;
    pia.ora = 0xAA;
    pia.ddra = 0xFF;
    assert_eq!(pia.output_a(), 0xAA);
    pia.ddra = 0x00;
    assert_eq!(pia.output_a(), 0x00);
    pia.ddra = 0xFF;
    assert_eq!(pia.output_a(), 0xAA);
}

// ── Write to port ──

#[test]
fn test_write_ora() {
    let mut pia = Pia6821::new();
    pia.cra |= 0x04;
    pia.ddra = 0xFF;
    pia.write(0, 0x42);
    assert_eq!(pia.ora, 0x42);
    assert_eq!(pia.output_a(), 0x42);
}

#[test]
fn test_write_ddra() {
    let mut pia = Pia6821::new();
    pia.write(0, 0x0F);
    assert_eq!(pia.ddra, 0x0F);
}

// ── CA1 interrupt ──

#[test]
fn test_ca1_rising_edge_sets_flag() {
    let mut pia = Pia6821::new();
    assert_eq!(pia.read(1, 0, 0) & 0x40, 0);
    pia.set_ca1(false);
    pia.set_ca1(true);
    assert_eq!(pia.read(1, 0, 0) & 0x40, 0); // falling active, rising ignored

    pia.cra |= 0x01; // select rising
    pia.set_ca1(false);
    pia.set_ca1(true);
    assert_eq!(pia.read(1, 0, 0) & 0x40, 0x40); // flag set
}

#[test]
fn test_ca1_falling_edge_sets_flag() {
    let mut pia = Pia6821::new();
    pia.set_ca1(true);
    pia.set_ca1(false);
    assert_eq!(pia.read(1, 0, 0) & 0x40, 0x40);
}

#[test]
fn test_ca1_flag_cleared_by_ora_read() {
    let mut pia = Pia6821::new();
    pia.cra |= 0x04;
    pia.set_ca1(true);
    pia.set_ca1(false);
    assert!(pia.irq_a());
    pia.read(0, 0, 0);
    assert!(!pia.irq_a());
    assert_eq!(pia.read(1, 0, 0) & 0x40, 0);
}

#[test]
fn test_ca1_flag_not_cleared_by_ddra_read() {
    let mut pia = Pia6821::new();
    pia.set_ca1(true);
    pia.set_ca1(false);
    assert_eq!(pia.read(1, 0, 0) & 0x40, 0x40);
    pia.read(0, 0, 0); // reads DDRA
    assert_eq!(pia.read(1, 0, 0) & 0x40, 0x40);
}

// ── IRQ output ──

#[test]
fn test_irq_a_asserted_when_flag_set() {
    let mut pia = Pia6821::new();
    assert!(!pia.irq_a());
    pia.set_ca1(true);
    pia.set_ca1(false);
    assert!(pia.irq_a());
}

#[test]
fn test_irq_a_deasserted_by_ora_read() {
    let mut pia = Pia6821::new();
    pia.cra |= 0x04;
    pia.set_ca1(true);
    pia.set_ca1(false);
    assert!(pia.irq_a());
    pia.read(0, 0, 0);
    assert!(!pia.irq_a());
}

// ── CB1 interrupt ──

#[test]
fn test_cb1_edge_sets_flag() {
    let mut pia = Pia6821::new();
    pia.set_cb1(true);
    pia.set_cb1(false);
    assert_eq!(pia.read(3, 0, 0) & 0x40, 0x40);
}

#[test]
fn test_cb1_flag_cleared_by_orb_read() {
    let mut pia = Pia6821::new();
    pia.crb |= 0x04;
    pia.set_cb1(true);
    pia.set_cb1(false);
    assert!(pia.irq_b());
    pia.read(2, 0, 0);
    assert!(!pia.irq_b());
}

// ── CA2 input mode ──

#[test]
fn test_ca2_input_rising_sets_flag() {
    let mut pia = Pia6821::new();
    pia.set_ca2(true);
    pia.set_ca2(false); // falling, default bit 1 = 0
    assert_eq!(pia.read(1, 0, 0) & 0x80, 0x80);

    pia.cra |= 0x04;
    pia.read(0, 0, 0); // clear
    assert_eq!(pia.read(1, 0, 0) & 0x80, 0);

    pia.cra |= 0x02; // CA2 active rising
    pia.set_ca2(false);
    pia.set_ca2(true);
    assert_eq!(pia.read(1, 0, 0) & 0x80, 0x80);
}

// ── CA2 output manual mode ──

#[test]
fn test_ca2_output_manual_low() {
    let mut pia = Pia6821::new();
    pia.write(1, 0b000_10_000);
    assert!(!pia.ca2_output());
    pia.write(1, 0b000_10_010);
    assert!(pia.ca2_output());
}

#[test]
fn test_ca2_output_manual_high() {
    let mut pia = Pia6821::new();
    pia.write(1, 0b000_11_000);
    assert!(!pia.ca2_output());
    pia.write(1, 0b000_11_010);
    assert!(pia.ca2_output());
}

// ── CA2 pulse/handshake modes ──

#[test]
fn test_ca2_pulse_on_ora_write() {
    let mut pia = Pia6821::new();
    pia.write(1, 0x04 | 0b100_000);
    assert!(pia.ca2_output());
    pia.write(0, 0xFF);
    assert!(!pia.ca2_output());
}

#[test]
fn test_ca2_handshake_on_ora_write() {
    let mut pia = Pia6821::new();
    pia.write(1, 0x04 | 0b110_000);
    assert!(pia.ca2_output());
    pia.write(0, 0xFF);
    assert!(!pia.ca2_output());
}

#[test]
fn test_cb2_pulse_on_orb_write() {
    let mut pia = Pia6821::new();
    pia.write(3, 0x04 | 0b100_000);
    assert!(pia.cb2_output());
    pia.write(2, 0xFF);
    assert!(!pia.cb2_output());
}

// ── Reset ──

#[test]
fn test_new_reset_clears_all() {
    let pia = Pia6821::new();
    assert_eq!(pia.ora, 0);
    assert_eq!(pia.orb, 0);
    assert_eq!(pia.ddra, 0);
    assert_eq!(pia.ddrb, 0);
    assert_eq!(pia.cra, 0);
    assert_eq!(pia.crb, 0);
    assert!(!pia.irq_a());
    assert!(!pia.irq_b());
    assert_eq!(pia.output_a(), 0);
    assert_eq!(pia.output_b(), 0);
}

// ── Port pin input ──

#[test]
fn test_set_input_a_visible_on_read() {
    let mut pia = Pia6821::new();
    pia.cra |= 0x04;
    pia.ddra = 0x00;
    pia.ora = 0xFF;
    pia.set_input_a(0x55);
    assert_eq!(pia.read(0, 0x55, 0), 0x55);
}

#[test]
fn test_set_input_b_visible_on_read() {
    let mut pia = Pia6821::new();
    pia.crb |= 0x04;
    pia.ddrb = 0x00;
    pia.set_input_b(0xAA);
    assert_eq!(pia.read(2, 0, 0xAA), 0xAA);
}
