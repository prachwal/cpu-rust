use super::*;

#[test]
fn test_font_loaded() {
    let m = Memory::new();
    assert_eq!(m.read(0x050), 0xF0);
    assert_eq!(m.read(0x09F), 0x80);
}

#[test]
fn test_custom_font_offset() {
    let m = Memory::with_font_offset(0x100);
    assert_eq!(m.read(0x100), 0xF0);
}

#[test]
fn test_read_write() {
    let mut m = Memory::new();
    m.write(0x200, 0xAB);
    assert_eq!(m.read(0x200), 0xAB);
}

#[test]
fn test_load_rom() {
    let mut m = Memory::new();
    let rom = vec![0x00, 0xE0, 0x12, 0x34];
    m.load_rom(&rom, 0x200);
    assert_eq!(m.read(0x200), 0x00);
    assert_eq!(m.read(0x203), 0x34);
}

#[test]
fn test_wraparound() {
    let m = Memory::with_font_offset(0x050);
    m.read(0x1000);
}
