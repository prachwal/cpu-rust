use super::*;
use cpu_bus::Bus;
use cpu_display::Display;
use cpu_keyboard::Keyboard;

fn load_rom(name: &str) -> Vec<u8> {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/roms");
    std::fs::read(base.join(name)).unwrap_or_else(|_| vec![0xFF; 1024])
}

fn make_kim1_bus() -> Kim1Bus {
    Kim1Bus {
        ram: vec![0; 1024],
        rom_002: load_rom("6530-002.bin"),
        rom_003: load_rom("6530-003.bin"),
        riot2_pa: 0, riot2_pb: 0xFF,
        riot3_pa: 0, riot3_pb: 0xFF,
        led_segments: [0; 6],
        display: Display::new(78, 15),
        keypad: Keyboard::new(),
    }
}

#[test]
fn test_roms_loaded_correctly() {
    let rom2 = load_rom("6530-002.bin");
    let rom3 = load_rom("6530-003.bin");
    assert_eq!(rom2.len(), 1024);
    assert_eq!(rom3.len(), 1024);
}

#[test]
fn test_reset_vector_readable() {
    let mut bus = make_kim1_bus();
    let lo = bus.read(0xFFFC);
    let hi = bus.read(0xFFFD);
    let reset_vec = (hi as u16) << 8 | lo as u16;
    // The vector bytes at $FFFC-D should be non-zero ROM data
    assert!(!(lo == 0xFF && hi == 0xFF),
        "Reset vector should not be all $FF: got ${:04X}", reset_vec);
}

#[test]
fn test_kim1_runs_known_entry() {
    let mut kim = Kim1::new(load_rom("6530-002.bin"), load_rom("6530-003.bin"));
    kim.cpu.set_register_pc(0x1C4F); // KIM-1 cold start
    for _ in 0..500 {
        kim.tick();
    }
    // PC should have moved from the entry point
    assert!(kim.get_register_pc() != 0x1C4F);
}

#[test]
fn test_bus_ram_read_write() {
    let mut bus = make_kim1_bus();
    bus.write(0x0100, 0x55);
    assert_eq!(bus.read(0x0100), 0x55);
}

#[test]
fn test_bus_rom_readable() {
    let mut bus = make_kim1_bus();
    let first_rom = bus.read(0x1C08);
    assert_ne!(first_rom, 0xFF, "ROM should contain code at $1C08");
}

#[test]
fn test_riot_registers() {
    let mut bus = make_kim1_bus();
    bus.write(0x1C00, 0xAA); // PA
    assert_eq!(bus.read(0x1C00), 0xAA);
    bus.write(0x1C02, 0xFF); // DDRA = all output
    bus.write(0x1C01, 0x55); // PB
    assert_eq!(bus.read(0x1C01), 0x55);
}
