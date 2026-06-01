/// Non-interactive test: loads ledtest.bin into KIM-1, runs it,
/// checks that the 7-segment LEDs show changing values.
use std::path::PathBuf;

fn main() {
    let rom_dir = PathBuf::from("crates/machines/kim1/tests/roms");
    let rom2 = std::fs::read(rom_dir.join("6530-002.bin")).unwrap();
    let rom3 = std::fs::read(rom_dir.join("6530-003.bin")).unwrap();

    let prog = std::fs::read("crates/tools/kim1-term/examples/ledtest.bin").unwrap();
    eprintln!("Program: {} bytes", prog.len());

    let mut kim = kim1::Kim1::new(rom2, rom3);
    // Load program at $0200
    for (i, &b) in prog.iter().enumerate() {
        kim.bus.write(0x0200 + i as u16, b);
    }
    kim.cpu.set_register_pc(0x0200);
    kim.cpu.set_register_sp(0xFF);

    // Run for 50000 instructions
    for _ in 0..50000 {
        kim.tick();
    }

    // Check that the display data bytes changed
    // The program stores counter in $02-$03, monitor displays them
    let data_lo = kim.bus.read(0x0002);
    let data_hi = kim.bus.read(0x0003);
    eprintln!("After 50000 ticks: data at $02=${:02X} $03=${:02X}", data_lo, data_hi);
    eprintln!("PC=${:04X} A=${:02X} X=${:02X}", kim.get_register_pc(), kim.get_register_a(), kim.get_register_x());

    // Check the display was rendered
    kim.render_display();
    let segs = kim.bus.led_segments();
    eprintln!("LED segments: {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
        segs[0], segs[1], segs[2], segs[3], segs[4], segs[5]);

    // Verify some segments are lit (program runs, display updates)
    let any_lit = segs.iter().any(|&s| s != 0);
    assert!(any_lit, "LED segments should show non-zero values after running");
    assert!(data_lo != 0 || data_hi != 0, "Counter should have incremented past 0");

    eprintln!("\n✓ KIM-1 LED test passed!");
}
