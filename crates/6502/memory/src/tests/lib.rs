use super::*;

#[test]
fn test_memory_creation() {
    let config = MachineConfig::default();
    let mem = Memory::new(&config);
    assert_eq!(mem.len(), 65536);
}

#[test]
fn test_basic_read_write() {
    let config = MachineConfig::default();
    let mut mem = Memory::new(&config);

    mem.write(0x1234, 0xAB);
    assert_eq!(mem.read(0x1234), 0xAB);
}

#[test]
fn test_read_write_u16() {
    let config = MachineConfig::default();
    let mut mem = Memory::new(&config);

    mem.write_u16(0x1000, 0x1234);
    assert_eq!(mem.read(0x1000), 0x34);
    assert_eq!(mem.read(0x1001), 0x12);
    assert_eq!(mem.read_u16(0x1000), 0x1234);
}

#[test]
fn test_load_rom() {
    let config = MachineConfig::default();
    let mut mem = Memory::new(&config);

    let rom = vec![0x00, 0xE0, 0x6A, 0x42];
    mem.load_rom(&rom, 0x8000);

    assert_eq!(mem.read(0x8000), 0x00);
    assert_eq!(mem.read(0x8001), 0xE0);
    assert_eq!(mem.read(0x8002), 0x6A);
    assert_eq!(mem.read(0x8003), 0x42);
}

#[test]
fn test_zero_page_detection() {
    assert!(Memory::is_zero_page(0x0000));
    assert!(Memory::is_zero_page(0x00FF));
    assert!(!Memory::is_zero_page(0x0100));
}

#[test]
fn test_stack_page_detection() {
    assert!(Memory::is_stack_page(0x0100));
    assert!(Memory::is_stack_page(0x01FF));
    assert!(!Memory::is_stack_page(0x00FF));
    assert!(!Memory::is_stack_page(0x0200));
}

#[test]
fn test_vectors() {
    let config = MachineConfig::default();
    let mut mem = Memory::new(&config);

    mem.set_reset_vector(0x8000);
    assert_eq!(mem.get_reset_vector(), 0x8000);

    mem.set_nmi_vector(0x9000);
    assert_eq!(mem.get_nmi_vector(), 0x9000);

    mem.set_irq_vector(0xA000);
    assert_eq!(mem.get_irq_vector(), 0xA000);
}

#[test]
fn test_bank_switching() {
    let mut config = MachineConfig::nmos6502();
    config.memory.bank_switching = true;
    config.memory.num_banks = 2;

    let mut mem = Memory::new(&config);
    assert_eq!(mem.num_banks(), 2);

    mem.set_bank(0).unwrap();
    mem.write(0x1000, 0xAA);

    mem.set_bank(1).unwrap();
    mem.write(0x1000, 0xBB);

    mem.set_bank(0).unwrap();
    assert_eq!(mem.read(0x1000), 0xAA);

    mem.set_bank(1).unwrap();
    assert_eq!(mem.read(0x1000), 0xBB);
}

#[test]
fn test_load_rom_bank() {
    let mut config = MachineConfig::nmos6502();
    config.memory.bank_switching = true;
    config.memory.num_banks = 2;

    let mut mem = Memory::new(&config);

    let rom = vec![0x01, 0x02, 0x03];
    mem.load_rom_bank(1, &rom, 0x8000).unwrap();

    mem.set_bank(1).unwrap();
    assert_eq!(mem.read(0x8000), 0x01);
    assert_eq!(mem.read(0x8001), 0x02);
    assert_eq!(mem.read(0x8002), 0x03);
}

#[test]
fn test_clear() {
    let config = MachineConfig::default();
    let mut mem = Memory::new(&config);

    mem.write(0x1000, 0xFF);
    mem.clear();
    assert_eq!(mem.read(0x1000), 0x00);
}
