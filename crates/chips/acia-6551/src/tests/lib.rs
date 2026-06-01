use super::*;

#[test]
fn test_new_reset_state() {
    let acia = Acia6551::new();
    assert_eq!(acia.status(), SR_TX_EMPTY | SR_DSR | SR_DCD);
    assert!(!acia.irq);
    assert_eq!(acia.command_reg(), 0);
    assert_eq!(acia.control_reg(), 0);
}

#[test]
fn test_write_data_sets_tx_output() {
    let mut acia = Acia6551::new();
    acia.write(0, 0x42);
    assert_eq!(acia.tx_output, Some(0x42));
}

#[test]
fn test_tx_empty_cleared_on_write_then_set_on_complete() {
    let mut acia = Acia6551::new();
    assert!(acia.status() & SR_TX_EMPTY != 0);
    acia.write(0, 0x55);
    assert_eq!(acia.status() & SR_TX_EMPTY, 0); // cleared
    assert!(acia.tx_complete());
    assert!(acia.status() & SR_TX_EMPTY != 0); // restored
}

#[test]
fn test_receive_sets_rx_full() {
    let mut acia = Acia6551::new();
    acia.receive(0xAB);
    assert!(acia.status() & SR_RX_FULL != 0);
    let data = acia.read(0);
    assert_eq!(data, 0xAB);
    assert_eq!(acia.status() & SR_RX_FULL, 0); // cleared on read
}

#[test]
fn test_overrun_detected() {
    let mut acia = Acia6551::new();
    acia.receive(0x01);
    acia.receive(0x02); // not read yet
    assert!(acia.status() & SR_OVERRUN != 0);
}

#[test]
fn test_irq_tx_enabled() {
    let mut acia = Acia6551::new();
    acia.write(2, CMD_IRQ_TX); // enable Tx IRQ
    assert!(acia.irq); // Tx empty already set → IRQ asserted
    assert!(acia.status() & SR_IRQ != 0);

    // write data → Tx empty cleared → IRQ deasserted
    acia.write(0, 0xFF);
    assert!(!acia.irq);
    assert_eq!(acia.status() & SR_IRQ, 0);

    // complete → IRQ reasserted
    acia.tx_complete();
    assert!(acia.irq);
}

#[test]
fn test_irq_rx_enabled() {
    let mut acia = Acia6551::new();
    acia.write(2, CMD_IRQ_RX); // enable Rx IRQ
    assert!(!acia.irq);

    acia.receive(0x77);
    assert!(acia.irq);

    acia.read(0); // clear
    assert!(!acia.irq);
}

#[test]
fn test_echo_mode_internal() {
    let mut acia = Acia6551::new();
    acia.write(2, CMD_MODE_ECHO as u8); // mode bits 1-0 = 01
    acia.write(0, 0x77);

    // In echo mode, data loops back to Rx; no Tx output
    assert_eq!(acia.tx_output, None);
    assert!(acia.status() & SR_RX_FULL != 0);
    assert_eq!(acia.read(0), 0x77);
}

#[test]
fn test_loopback_mode() {
    let mut acia = Acia6551::new();
    acia.write(2, CMD_MODE_LOOPBACK as u8); // mode = 10

    acia.write(0, 0x42);
    // In loopback, both Tx output and Rx receive
    assert_eq!(acia.tx_output, Some(0x42));
    assert!(acia.status() & SR_RX_FULL != 0);
    assert_eq!(acia.read(0), 0x42);
}

#[test]
fn test_reset_clears_everything() {
    let mut acia = Acia6551::new();
    acia.receive(0xFF);
    acia.write(0, 0x55);
    acia.write(2, 0xFF);
    acia.write(3, 0xFF);

    acia.write(1, 0); // software reset

    assert_eq!(acia.status(), SR_TX_EMPTY | SR_DSR | SR_DCD);
    assert_eq!(acia.command_reg(), 0);
    assert_eq!(acia.control_reg(), 0);
    assert!(!acia.irq);
    assert!(!acia.tx_pending);
    assert!(!acia.rx_pending);
}

#[test]
fn test_dsr_dcd_pins() {
    let mut acia = Acia6551::new();
    assert!(acia.status() & SR_DSR != 0);
    assert!(acia.status() & SR_DCD != 0);

    acia.set_dsr(false);
    assert_eq!(acia.status() & SR_DSR, 0);

    acia.set_dcd(false);
    assert_eq!(acia.status() & SR_DCD, 0);
}

#[test]
fn test_register_addressing() {
    let mut acia = Acia6551::new();
    acia.write(0, 0xAA);
    assert_eq!(acia.read(0), 0x00); // data register reads rx data

    acia.receive(0xBB);
    assert_eq!(acia.read(0), 0xBB); // rx data

    acia.write(3, 0x1E); // control
    assert_eq!(acia.read(3), 0x1E);

    acia.write(2, 0x0B); // command
    assert_eq!(acia.read(2), 0x0B);
}

#[test]
fn test_tx_empty_on_startup_triggers_irq_if_enabled() {
    let mut acia = Acia6551::new();
    acia.write(2, 0x08); // enable Tx IRQ
    // Tx empty is already set after reset → IRQ should fire
    assert!(acia.irq);
}

#[test]
fn test_write_after_tx_complete_sets_new_output() {
    let mut acia = Acia6551::new();
    acia.write(0, 0x11);
    assert_eq!(acia.tx_output, Some(0x11));

    acia.tx_complete();
    acia.write(0, 0x22);
    assert_eq!(acia.tx_output, Some(0x22));
}
