use super::*;

#[test]
fn test_new() {
    let mem = Memory::new(65536);
    assert_eq!(mem.len(), 65536);
}

#[test]
fn test_read_write() {
    let mut mem = Memory::new(65536);
    mem.write(0x1234, 0xAB);
    assert_eq!(mem.read(0x1234), 0xAB);
}

#[test]
fn test_read_write_u16() {
    let mut mem = Memory::new(65536);
    mem.write_u16(0x1000, 0x1234);
    assert_eq!(mem.read_u16(0x1000), 0x1234);
}

#[test]
fn test_load() {
    let mut mem = Memory::new(65536);
    let data = vec![0x01, 0x02, 0x03, 0x04];
    mem.load(&data, 0x8000);
    assert_eq!(mem.read(0x8000), 0x01);
    assert_eq!(mem.read(0x8001), 0x02);
    assert_eq!(mem.read(0x8003), 0x04);
}

#[test]
fn test_wraparound() {
    let mut mem = Memory::new(256);
    mem.write(0x100, 0x42);
    assert_eq!(mem.read(0x00), 0x42);
}

#[test]
fn test_clear() {
    let mut mem = Memory::new(256);
    mem.write(0x10, 0xFF);
    mem.clear();
    assert_eq!(mem.read(0x10), 0x00);
}

#[test]
fn test_bus_trait() {
    let mut mem = Memory::new(65536);
    mem.write(0x2000, 0xAA);
    assert_eq!(mem.read(0x2000), 0xAA);
}
