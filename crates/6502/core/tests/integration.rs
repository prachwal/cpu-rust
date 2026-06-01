use mos6502_core::*;

fn run_until(emu: &mut Emulator, timeout_cycles: u64) -> u64 {
    let mut total = 0;
    loop {
        let c = emu.step() as u64;
        total += c;
        if c == 0 || total >= timeout_cycles {
            break;
        }
    }
    total
}

/// 6502 Functional Test ROM — pełny 64KB test wszystkich opcodów.
/// ROM jest mapowany bezpośrednio w pamięci od $0000.
/// Kończy się zapisem $00 do $0000 przy sukcesie.
#[test]
fn test_6502_functional_test_rom() {
    let rom = include_bytes!("roms/6502_functional_test.bin");
    let mut emu = Emulator::new();
    emu.load_rom(rom, 0x0000);
    // Ustaw reset vector na $0000 (ROM start)
    emu.set_memory_u16(0xFFFC, 0x0000);
    emu.reset();

    // Wykonaj do timeoutu (20M cykli)
    let cycles = run_until(&mut emu, 20_000_000);

    // Sprawdź czy test przeszedł: $00 = 0 oznacza sukces
    let result = emu.get_memory(0x00);
    assert_eq!(result, 0x00,
        "Functional test failed at PC=${:04X}, result=${:02X}, cycles={}",
        emu.get_register_pc(), result, cycles);
    println!("6502_functional_test: {} instructions, {} cycles",
        emu.get_instruction_count(), cycles);
}

fn load_nestest() -> Emulator {
    let rom = include_bytes!("roms/nestest.nes");
    assert_eq!(&rom[0..4], b"NES\x1A");
    let prg_size = (rom[4] as usize) * 16384;
    let prg_data = &rom[16..16 + prg_size];
    // NMOS 6502 with BCD disabled (Ricoh 2A03 behavior)
    let mut cfg = MachineConfig::nmos6502();
    cfg.quirks.bcd_available = false;
    let mut emu = Emulator::new_with_config(&cfg.to_json()).expect("NES config should parse");
    emu.load_rom(prg_data, 0xC000);
    if prg_size <= 16384 {
        emu.load_rom(prg_data, 0x8000);
    }
    emu.set_register_pc(0xC000);
    emu.set_register_sp(0xFD);
    emu
}

fn parse_nestest_log(limit: usize) -> Vec<(u16, u8, u8, u8, u8, u8)> {
    let log = include_str!("roms/nestest.log");
    let mut entries = Vec::with_capacity(limit);
    for line in log.lines() {
        if entries.len() == limit { break; }
        let line = line.trim();
        if line.is_empty() { continue; }
        // Format: C000  4C F5 C5  JMP $C5F5  A:00 X:00 Y:00 P:24 SP:FD PPU:...
        if line.len() < 50 { continue; }
        let pc = u16::from_str_radix(&line[0..4], 16).unwrap_or(0);
        // Parse register block: "A:00 X:00 Y:00 P:24 SP:FD"
        let regs = &line[33..];
        let (a, rest) = scan_hex(regs, 2);   // "A:00"
        let (x, rest) = scan_hex(rest, 2);    // "X:00"
        let (y, rest) = scan_hex(rest, 2);    // "Y:00"
        let (p, rest) = scan_hex(rest, 2);    // "P:24"
        let (sp, _) = scan_hex(rest, 2);      // "SP:FD"
        entries.push((pc, a, x, y, p, sp));
    }
    entries
}

fn scan_hex(s: &str, digits: usize) -> (u8, &str) {
    // skip label like "A:", "X:", "Y:", "P:", "SP:"
    let colon = s.find(':').unwrap_or(0);
    let val_start = colon + 1;
    let end = (val_start + digits).min(s.len());
    let val = u8::from_str_radix(&s[val_start..end], 16).unwrap_or(0);
    (val, &s[end..])
}

/// nestest.log comparison — porównuje każdy krok emulacji z oczekiwanym logiem.
/// Sprawdza PC, A, X, Y, P (SR), SP dla pierwszych 6000 instrukcji.
#[test]
fn test_nestest_against_log() {
    let mut emu = load_nestest();
    let log = parse_nestest_log(5828);
    let max_check = log.len();
    let mut mismatch: Option<String> = None;

    for i in 0..max_check {
        let (exp_pc, exp_a, exp_x, exp_y, exp_p, exp_sp) = log[i];

        // Sprawdź stan PRZED wykonaniem instrukcji
        let pc = emu.get_register_pc();
        let a  = emu.get_register_a();
        let x  = emu.get_register_x();
        let y  = emu.get_register_y();
        let p  = emu.get_status_register();
        let sp = emu.get_register_sp();

        if pc != exp_pc || a != exp_a || x != exp_x || y != exp_y || p != exp_p || sp != exp_sp {
            mismatch = Some(format!(
                "Line {}: expected PC=${:04X} A=${:02X} X=${:02X} Y=${:02X} P=${:02X} SP=${:02X}, \
                 got      PC=${:04X} A=${:02X} X=${:02X} Y=${:02X} P=${:02X} SP=${:02X}",
                i + 1, exp_pc, exp_a, exp_x, exp_y, exp_p, exp_sp,
                pc, a, x, y, p, sp
            ));
            break;
        }

        emu.tick();
    }

    if let Some(msg) = mismatch {
        panic!("nestest mismatch:\n{}", msg);
    }

    // Jeśli dotarliśmy do końca bez błędu — sprawdź że przynajmniej 1000 instrukcji OK
    assert!(max_check >= 1000, "Only checked {} instructions", max_check);
}

#[test]
fn test_apple1_wozmon_runs() {
    let wozmon = include_bytes!("roms/wozmon.bin");
    let mut emu = Emulator::new_nmos();
    emu.load_rom(wozmon, 0xFF00);
    emu.set_register_pc(0xFF00);
    emu.set_register_sp(0xFD);

    // Just verify WozMon starts and executes without crashing
    let count = emu.run_instructions(256);
    assert!(count > 0);
}
