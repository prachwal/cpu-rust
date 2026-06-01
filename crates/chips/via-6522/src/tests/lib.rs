use super::*;

#[test]
fn test_read_write_regs() {
    let mut via = Via6522::new();
    via.write(1, 0x42); // PORTA
    assert_eq!(via.read(1), 0x42);
    via.write(2, 0xF0); // DDRB
    assert_eq!(via.read(2), 0xF0);
}

#[test]
fn test_timer1_oneshot() {
    let mut via = Via6522::new();
    via.write(14, 0xC0); // IER: enable T1 (bit 6)
    via.write(4, 0x05);  // T1L-L = 5
    via.write(5, 0x00);  // T1L-H = 0 → T1L = 5, start
    assert_eq!(via.t1_counter, 5);
    via.tick(4);
    assert!(!via.irq);
    via.tick(1); // total 5 → underflow
    assert!(via.irq);
}

#[test]
fn test_timer1_freerun() {
    let mut via = Via6522::new();
    via.write(14, 0xC0); // IER: enable T1
    via.write(11, 0x80); // ACR bit 7 = free-running
    via.write(4, 0x03);  // T1L-L = 3
    via.write(5, 0x00);  // T1L-H = 0 → start
    // First underflow after 3 ticks
    via.tick(3);
    assert!(via.irq);
    via.ifr_clear(IRQ_T1);
    assert!(!via.irq);
    via.tick(3);
    assert!(via.irq); // auto-reloaded
}

#[test]
fn test_ier_ien() {
    let mut via = Via6522::new();
    // Enable T1 interrupt: IER = $C0 (bit 7=set, bit 6=T1)
    via.write(14, 0xC0);
    assert_eq!(via.regs[14] & 0x40, 0x40);
    // Trigger T1
    via.ifr_set(IRQ_T1);
    assert!(via.irq);
    // Disable T1: IER = $40 (bit 7=clear, bit 6=T1)
    via.write(14, 0x40);
    assert_eq!(via.regs[14] & 0x40, 0x00);
    // IFR still set but IRQ shouldn't fire
    assert!(!via.irq);
}

#[test]
fn test_cb1_trigger() {
    let mut via = Via6522::new();
    via.trigger_cb1();
    assert_eq!(via.regs[13] & IRQ_CB1, IRQ_CB1);
    // Read IFR to clear
    let ifr = via.read(13);
    assert!(ifr & IRQ_CB1 != 0);
    assert_eq!(via.regs[13], 0);
}
