use super::*;

#[test]
fn test_read_write() {
    let mut pia = Pia6821::new();
    pia.write(0, 0xAA);
    assert_eq!(pia.read(0, 0, 0), 0x00); // DDR=0 → reads input (0)
    pia.ddra = 0xFF;
    assert_eq!(pia.read(0, 0, 0), 0xAA); // DDR=1 → reads output
    pia.set_input_a(0x55);
    assert_eq!(pia.read(0, 0x55, 0), 0xAA); // still reads output (DDR=1)
    pia.ddra = 0x00;
    assert_eq!(pia.read(0, 0x55, 0), 0x55); // DDR=0 → reads input
}

#[test]
fn test_control_registers() {
    let mut pia = Pia6821::new();
    pia.write(1, 0xA7);
    assert_eq!(pia.read(1, 0, 0), 0xA7);
    pia.write(3, 0x34);
    assert_eq!(pia.read(3, 0, 0), 0x34);
}

#[test]
fn test_output() {
    let mut pia = Pia6821::new();
    pia.ddra = 0xF0;
    pia.ora = 0xFF;
    assert_eq!(pia.output_a(), 0xF0); // only high nibble is output
    pia.set_input_a(0x0F);
    assert_eq!(pia.read(0, 0x0F, 0), 0xFF); // F0 (out) | 0F (in)
}
