use std::path::PathBuf;

fn main() {
    let rom = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| {
        "crates/machines/chip8/tests/roms/6-keypad.ch8".into()
    }));

    let data = std::fs::read(&rom).expect("read ROM");
    let mut emu = chip8_machine::Emulator::new();
    emu.load_rom(&data);

    let mut instr_count: u64 = 0;
    let key = 0x1u8;

    println!("=== keytest: loading {:?} ===", rom);
    println!("Display: {}x{}", emu.get_display_width(), emu.get_display_height());

    // run for 1000 instructions, then press key 0x1
    for _ in 0..1000 {
        emu.tick();
        instr_count += 1;
    }
    println!("After 1000 instr: PC=${:04X} I=${:04X}", emu.get_register_pc(), emu.get_register_i());
    for i in 0..16 { print!("V{:X}=${:02X} ", i, emu.get_register_v(i)); }
    println!();

    // press key
    println!("\n--- pressing key 0x{:X} ---", key);
    emu.key_down(key);

    // run for another 2000 instructions
    for _ in 0..2000 {
        emu.tick();
        instr_count += 1;
    }
    println!("After +2000 instr: PC=${:04X} I=${:04X}", emu.get_register_pc(), emu.get_register_i());
    for i in 0..16 { print!("V{:X}=${:02X} ", i, emu.get_register_v(i)); }
    println!();

    // release key
    println!("\n--- releasing key 0x{:X} ---", key);
    emu.key_up(key);

    // run for another 2000
    for _ in 0..2000 {
        emu.tick();
        instr_count += 1;
    }
    println!("After +2000 instr: PC=${:04X} I=${:04X}", emu.get_register_pc(), emu.get_register_i());
    for i in 0..16 { print!("V{:X}=${:02X} ", i, emu.get_register_v(i)); }
    println!();

    // press+hold key again
    println!("\n--- holding key 0x{:X} for 5000 instr ---", key);
    emu.key_down(key);
    for _ in 0..5000 {
        emu.tick();
        instr_count += 1;
    }
    println!("After +5000 instr: PC=${:04X} I=${:04X}", emu.get_register_pc(), emu.get_register_i());
    for i in 0..16 { print!("V{:X}=${:02X} ", i, emu.get_register_v(i)); }
    println!();

    println!("\nTotal: {} instructions", instr_count);
}
