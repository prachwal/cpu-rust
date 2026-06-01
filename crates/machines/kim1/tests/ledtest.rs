use cpu_bus::Bus;

fn load_rom(name: &str) -> Vec<u8> {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/roms");
    std::fs::read(base.join(name)).unwrap()
}

/// Direct RIOT test: write segment data to PA, select digit via PB
/// and check that led_segments captures it
#[test]
fn test_direct_led_write() {
    let rom2 = load_rom("6530-002.bin");
    let rom3 = load_rom("6530-003.bin");
    let mut kim = kim1::Kim1::new(rom2, rom3);

    // Write segment pattern for "8" (all segments on = $7F) to PA
    kim.bus.write(0x1C00, 0x7F); // 6530-002 PA = segments

    // Write PB with digit 0 selected (bit 0 low)
    kim.bus.write(0x1C01, 0b111110); // bit 0 = 0 → digit 0 active

    // Check that led_segments captured it
    let segs = *kim.bus.led_segments();
    assert_eq!(segs[0], 0x7F, "Digit 0 should show all segments");
    assert_eq!(segs[1], 0x00, "Other digits should be 0");

    // Select digit 1 with a different pattern
    kim.bus.write(0x1C00, 0x06); // segment pattern for "1" (B+C)
    kim.bus.write(0x1C01, 0b111101); // bit 1 = 0

    let segs = *kim.bus.led_segments();
    assert_eq!(segs[0], 0x7F, "Digit 0 preserved");
    assert_eq!(segs[1], 0x06, "Digit 1 shows '1' pattern");
}

/// Load a simple program that writes directly to RIOT ports
#[test]
fn test_direct_port_program() {
    let rom2 = load_rom("6530-002.bin");
    let rom3 = load_rom("6530-003.bin");
    let mut kim = kim1::Kim1::new(rom2, rom3);

    // Program that:
    // 1. Writes $7F (8) to PA
    // 2. Writes bit select to PB
    // 3. Loops forever (JMP $)
    let prog: &[u8] = &[
        0xA9, 0x7F,           // lda #$7F (all segments)
        0x8D, 0x00, 0x1C,     // sta $1C00 (PA)
        0xA9, 0xFE,           // lda #$FE (bit 0 = 0)
        0x8D, 0x01, 0x1C,     // sta $1C01 (PB)
        0x4C, 0x00, 0x02,     // jmp $0200
    ];
    for (i, &b) in prog.iter().enumerate() {
        kim.bus.write(0x0200 + i as u16, b);
    }

    kim.cpu.set_register_pc(0x0200);
    kim.cpu.set_register_sp(0xFF);
    for _ in 0..100 { kim.tick(); }

    kim.render_display();
    let segs = *kim.bus.led_segments();
    eprintln!("Direct port test LEDs: {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
        segs[0], segs[1], segs[2], segs[3], segs[4], segs[5]);

    assert_eq!(segs[0], 0x7F, "Digit 0 should be lit");
}
