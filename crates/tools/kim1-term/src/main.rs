use std::io::{stdout, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use cpu_bus::Bus;
use crossterm::cursor::{Hide, Show};
use crossterm::event::{poll, read, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use crossterm::{execute, queue};

const TICK_HZ: u64 = 100_000;
const TICK_INTERVAL: Duration = Duration::from_nanos(1_000_000_000 / TICK_HZ);
const FRAME_INTERVAL: Duration = Duration::from_millis(33);
const ROM_DIR: &str = "crates/machines/kim1/tests/roms";

fn render_leds(segments: &[u8; 6]) -> Vec<String> {
    const W: usize = 13;
    const H: usize = 13;
    let mut lines = vec![String::new(); H];
    for (i, &seg) in segments.iter().enumerate() {
        let mut dbuf = [0u8; W * H];
        cpu_segment::render(seg, &mut dbuf, W, W, H);
        for y in 0..H {
            for x in 0..W { lines[y].push(if dbuf[y*W+x] != 0 { '█' } else { ' ' }); }
            if i < 5 { lines[y].push(' '); }
        }
    }
    lines
}

fn load_roms() -> (Vec<u8>, Vec<u8>) {
    let dir = PathBuf::from(ROM_DIR);
    let r2 = std::fs::read(dir.join("6530-002.bin")).expect("6530-002.bin not found");
    let r3 = std::fs::read(dir.join("6530-003.bin")).expect("6530-003.bin not found");
    (r2, r3)
}

/// Map KIM-1 hex keypad position (row, col) to monitor keycode
/// KIM-1 keypad layout (row × col):
///   Row 0: 1  2  3  F
///   Row 1: 4  5  6  E
///   Row 2: 7  8  9  D
///   Row 3: A  0  B  C
fn kim_keycode(row: u8, col: u8) -> Option<u8> {
    Some(match (row, col) {
        (0, 0) => 0x01, (0, 1) => 0x02, (0, 2) => 0x03, (0, 3) => 0x0F,
        (1, 0) => 0x04, (1, 1) => 0x05, (1, 2) => 0x06, (1, 3) => 0x0E,
        (2, 0) => 0x07, (2, 1) => 0x08, (2, 2) => 0x09, (2, 3) => 0x0D,
        (3, 0) => 0x0A, (3, 1) => 0x00, (3, 2) => 0x0B, (3, 3) => 0x0C,
        _ => return None,
    })
}

/// PC key → (row, col) on KIM-1 keypad
fn pc_to_kim_pos(key: KeyCode) -> Option<(u8, u8)> {
    match key {
        KeyCode::Char('1') => Some((0,0)), KeyCode::Char('2') => Some((0,1)),
        KeyCode::Char('3') => Some((0,2)), KeyCode::Char('4') => Some((1,0)),
        KeyCode::Char('5') => Some((1,1)), KeyCode::Char('6') => Some((1,2)),
        KeyCode::Char('7') => Some((2,0)), KeyCode::Char('8') => Some((2,1)),
        KeyCode::Char('9') => Some((2,2)), KeyCode::Char('0') => Some((3,1)),
        KeyCode::Char('a')|KeyCode::Char('A') => Some((3,0)),
        KeyCode::Char('b')|KeyCode::Char('B') => Some((3,2)),
        KeyCode::Char('c')|KeyCode::Char('C') => Some((3,3)),
        KeyCode::Char('d')|KeyCode::Char('D') => Some((2,3)),
        KeyCode::Char('e')|KeyCode::Char('E') => Some((1,3)),
        KeyCode::Char('f')|KeyCode::Char('F') => Some((0,3)),
        KeyCode::Char('+') => Some((4,1)),  // AD
        KeyCode::Enter => Some((4,2)),      // DA
        KeyCode::Tab => Some((4,3)),        // PC
        KeyCode::Char('g')|KeyCode::Char('G') => Some((4,4)),  // GO
        KeyCode::Char('s')|KeyCode::Char('S') => Some((4,5)),  // ST
        KeyCode::Char('r')|KeyCode::Char('R') => Some((4,6)),  // RS
        _ => None,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (rom2, rom3) = load_roms();
    let mut kim = kim1::Kim1::new(rom2, rom3);

    // Load program if specified
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--load" => {
                i += 1;
                let path = &args[i];
                let data = std::fs::read(path).expect("Cannot read ROM file");
                let addr = if i + 1 < args.len() && !args[i+1].starts_with('-') {
                    i += 1;
                    u16::from_str_radix(args[i].trim_start_matches("0x"), 16).expect("Invalid address")
                } else {
                    0x0200 // default
                };
                for (j, &b) in data.iter().enumerate() {
                    kim.bus.write(addr.wrapping_add(j as u16), b);
                }
                kim.cpu.set_register_pc(addr);
                eprintln!("Loaded {} bytes at ${:04X}", data.len(), addr);
            }
            "--addr" => {
                i += 1;
                let a = u16::from_str_radix(args[i].trim_start_matches("0x"), 16).expect("Invalid address");
                kim.cpu.set_register_pc(a);
            }
            _ => {}
        }
        i += 1;
    }

    // Initialize KIM-1 RAM locations for monitor
    kim.bus.write(0x17F5, 0x00); // clear last key

    enable_raw_mode().unwrap();
    execute!(stdout(), Hide, Clear(ClearType::All)).unwrap();

    let mut last_tick = Instant::now();
    let mut last_frame = Instant::now();
    let mut running = true;
    let mut instr_count: u64 = 0;
    let mut key_label = String::new();

    // KIM keypad matrix state: which row is being scanned
    while running {
        let now = Instant::now();

        while poll(Duration::ZERO).unwrap() {
            match read().unwrap() {
                Event::Key(ke) if ke.kind == KeyEventKind::Press => {
                    match ke.code {
                        KeyCode::Esc => running = false,
                        KeyCode::Char('c') if ke.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => running = false,
                        k => {
                            if let Some((row, col)) = pc_to_kim_pos(k) {
                                key_label = format!("R{}C{}", row, col);
                                // Store keycode in KIM-1 keyboard buffer
                                if let Some(kc) = kim_keycode(row, col) {
                                    kim.bus.write(0x17F5, kc); // last key for monitor
                                    kim.bus.write(0x17F7, kc); // alternate buffer
                                }
                                // Set RIOT 6530-003 PA to simulate keypress
                                // (keyboard column data)
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if now - last_tick >= TICK_INTERVAL {
            kim.tick();
            instr_count += 1;
            // RIOT 6530-003 keypad scan emulation: set PB based on scan
            // When monitor reads port B, return key column
            last_tick = now;
        }

        if now - last_frame >= FRAME_INTERVAL {
            kim.render_display();
            let segs = kim.bus.led_segments();
            let led = render_leds(segs);

            let info = format!(
                "  PC=${:04X} A=${:02X} X=${:02X} Y=${:02X} SP=${:02X}  instr={}",
                kim.get_register_pc(), kim.get_register_a(),
                kim.get_register_x(), kim.get_register_y(),
                kim.get_register_sp(), instr_count,
            );

            queue!(stdout(), Clear(ClearType::All)).unwrap();
            let mut buf = String::new();
            buf.push_str("\r\n  KIM-1\r\n\r\n");
            for l in &led { buf.push_str("  "); buf.push_str(l); buf.push_str("\r\n"); }
            buf.push_str(&format!("\r\n  Key: {}\r\n", key_label));
            buf.push_str(&info);
            buf.push_str("\r\n  0-9 A-F=hex  +=AD  Enter=DA  G=GO  S=ST  R=RS  ESC=quit\r\n");
            print!("{}", buf);
            stdout().flush().unwrap();
            last_frame = now;
        }

        std::thread::sleep(Duration::from_millis(1));
    }

    let _ = disable_raw_mode();
    let _ = execute!(stdout(), Show, Clear(ClearType::All));
    println!("KIM-1: {} instructions.", instr_count);
}
