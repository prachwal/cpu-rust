use std::io::{stdout, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const ROM_DIR: &str = "crates/machines/chip8/tests/roms";

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{
    poll, read, Event, KeyCode, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags,
    PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use crossterm::{execute, queue};

const TICK_HZ: u64 = 800;
const TIMER_HZ: u64 = 60;
const TICK_INTERVAL: Duration = Duration::from_nanos(1_000_000_000 / TICK_HZ);
const TIMER_INTERVAL: Duration = Duration::from_nanos(1_000_000_000 / TIMER_HZ);

fn pc_key_to_chip8(key: KeyCode) -> Option<u8> {
    match key {
        KeyCode::Char('1') => Some(0x1),
        KeyCode::Char('2') => Some(0x2),
        KeyCode::Char('3') => Some(0x3),
        KeyCode::Char('4') => Some(0xC),
        KeyCode::Char('q') | KeyCode::Char('Q') => Some(0x4),
        KeyCode::Char('w') | KeyCode::Char('W') => Some(0x5),
        KeyCode::Char('e') | KeyCode::Char('E') => Some(0x6),
        KeyCode::Char('r') | KeyCode::Char('R') => Some(0xD),
        KeyCode::Char('a') | KeyCode::Char('A') => Some(0x7),
        KeyCode::Char('s') | KeyCode::Char('S') => Some(0x8),
        KeyCode::Char('d') | KeyCode::Char('D') => Some(0x9),
        KeyCode::Char('f') | KeyCode::Char('F') => Some(0xE),
        KeyCode::Char('z') | KeyCode::Char('Z') => Some(0xA),
        KeyCode::Char('x') | KeyCode::Char('X') => Some(0x0),
        KeyCode::Char('c') | KeyCode::Char('C') => Some(0xB),
        KeyCode::Char('v') | KeyCode::Char('V') => Some(0xF),
        _ => None,
    }
}

fn render_display(buffer: &[u8], width: u16, height: u16) -> Vec<String> {
    let w = width as usize;
    let h = height as usize;
    let mut lines = Vec::new();

    lines.push(format!("┌{}┐", "─".repeat(w)));

    for row in (0..h).step_by(2) {
        let mut line = String::with_capacity(w + 2);
        line.push('│');
        for col in 0..w {
            let top = buffer[row * w + col] != 0;
            let bot = if row + 1 < h {
                buffer[(row + 1) * w + col] != 0
            } else {
                false
            };
            line.push(match (top, bot) {
                (true, true) => '█',
                (true, false) => '▀',
                (false, true) => '▄',
                (false, false) => ' ',
            });
        }
        line.push('│');
        lines.push(line);
    }

    lines.push(format!("└{}┘", "─".repeat(w)));
    lines
}

fn draw_screen(lines: &[String], info: &str) {
    let mut buf = String::new();
    for l in lines {
        buf.push_str(l);
        buf.push_str("\r\n");
    }
    buf.push_str(info);
    buf.push_str("\r\n");
    print!("{}", buf);
    stdout().flush().unwrap();
}

fn help() -> ! {
    println!("CHIP-8 Terminal Emulator");
    println!();
    println!("Usage: chip8-term [OPTIONS] <rom.ch8>");
    println!();
    println!("Options:");
    println!("  --debug          Log key events to stderr");
    println!("  --auto-key <hex> Auto-press+release key (e.g. 0x1) every 500ms");
    println!("  --help           This help");
    println!();
    println!("Controls:");
    println!("  HEX KEYPAD       PC KEY        HEX KEYPAD    PC KEY");
    println!("    1  2  3  C       1  2  3  4     A  0  B  F     Z  X  C  V");
    println!("    4  5  6  D       Q  W  E  R");
    println!("    7  8  9  E       A  S  D  F");
    println!("  ESC/Ctrl+C  Quit    Ctrl+R  Reset    H/F1  Help");
    std::process::exit(0);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut debug = false;
    let mut auto_key: Option<u8> = None;
    let mut rom_arg: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--debug" => debug = true,
            "--auto-key" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--auto-key needs a hex value");
                    std::process::exit(1);
                }
                auto_key = Some(
                    u8::from_str_radix(args[i].trim_start_matches("0x"), 16).unwrap_or_else(|_| {
                        eprintln!("Invalid key: {}", args[i]);
                        std::process::exit(1);
                    }),
                );
            }
            "--help" | "-h" => help(),
            s if s.starts_with('-') => {
                eprintln!("Unknown: {}", s);
                help();
            }
            _ => rom_arg = Some(args[i].clone()),
        }
        i += 1;
    }

    let rom_path = match rom_arg {
        Some(ref name) => {
            let p = PathBuf::from(name);
            if p.is_file() {
                p
            } else {
                let fallback = Path::new(ROM_DIR).join(name);
                if fallback.is_file() {
                    fallback
                } else {
                    eprintln!(
                        "ROM not found: {} (searched {:?} and {:?})",
                        name, p, fallback
                    );
                    std::process::exit(1);
                }
            }
        }
        None => {
            eprintln!("No ROM specified");
            help();
        }
    };

    let rom_data = std::fs::read(&rom_path).unwrap_or_else(|e| {
        eprintln!("Cannot read ROM: {}", e);
        std::process::exit(1);
    });
    let rom_name = rom_path.file_name().unwrap().to_string_lossy().to_string();

    let mut emu = chip8_machine::Emulator::new();
    emu.load_rom(&rom_data);

    if debug {
        eprintln!(
            "[DEBUG] ROM loaded: {} ({} bytes)",
            rom_name,
            rom_data.len()
        );
        eprintln!(
            "[DEBUG] Display: {}x{}",
            emu.get_display_width(),
            emu.get_display_height()
        );
        if let Some(k) = auto_key {
            eprintln!("[DEBUG] Auto-key: 0x{:X}", k);
        }
    }

    enable_raw_mode().unwrap();
    let enhanced_keyboard = matches!(
        crossterm::terminal::supports_keyboard_enhancement(),
        Ok(true)
    );
    if enhanced_keyboard {
        execute!(
            stdout(),
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
        )
        .unwrap();
    }
    execute!(stdout(), Hide, Clear(ClearType::All)).unwrap();

    let mut last_tick = Instant::now();
    let mut last_timer = Instant::now();
    let mut last_frame = Instant::now();
    let mut last_auto_key = Instant::now();
    let app_start = Instant::now();
    let mut running = true;
    let mut show_help = false;
    let mut instr_count: u64 = 0;
    let mut info;

    // Some terminals do not emit release events. Without keyboard enhancement,
    // treat a press as a one-tick tap so keys cannot remain stuck.
    let mut release_pending = [false; 16];

    // auto-key state for simulated press+release
    let mut auto_key_pressed = false;

    while running {
        let now = Instant::now();

        // --- input ---
        while poll(Duration::ZERO).unwrap() {
            match read().unwrap() {
                Event::Key(ke) => {
                    let chip8_key = pc_key_to_chip8(ke.code);
                    if debug {
                        eprintln!(
                            "[DEBUG] key={:?} kind={:?} chip8={:?}",
                            ke.code, ke.kind, chip8_key
                        );
                    }
                    match ke.kind {
                        KeyEventKind::Press | KeyEventKind::Repeat => {
                            if let Some(chip8) = chip8_key {
                                emu.key_down(chip8);
                                release_pending[chip8 as usize] = !enhanced_keyboard;
                                if debug {
                                    eprintln!("[DEBUG]   -> key_down(0x{:X})", chip8);
                                }
                            }
                            match ke.code {
                                KeyCode::Esc => running = false,
                                KeyCode::Char('c')
                                    if ke.modifiers.contains(KeyModifiers::CONTROL) =>
                                {
                                    running = false
                                }
                                KeyCode::Char('r') | KeyCode::Char('R')
                                    if ke.modifiers.contains(KeyModifiers::CONTROL) =>
                                {
                                    emu = chip8_machine::Emulator::new();
                                    emu.load_rom(&rom_data);
                                    instr_count = 0;
                                    if debug {
                                        eprintln!("[DEBUG] Reset");
                                    }
                                }
                                KeyCode::F(1) | KeyCode::Char('h') | KeyCode::Char('H') => {
                                    show_help = !show_help
                                }
                                _ => {}
                            }
                        }
                        KeyEventKind::Release => {
                            if let Some(chip8) = chip8_key {
                                release_pending[chip8 as usize] = true;
                                if debug {
                                    eprintln!("[DEBUG]   -> release_pending(0x{:X})", chip8);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // --- auto-key simulation ---
        if let Some(k) = auto_key {
            if now - last_auto_key >= Duration::from_millis(500) {
                if auto_key_pressed {
                    release_pending[k as usize] = true; // will flush after next tick
                } else {
                    emu.key_down(k); // press
                    release_pending[k as usize] = true;
                }
                auto_key_pressed = !auto_key_pressed;
                last_auto_key = now;
                if debug {
                    eprintln!(
                        "[DEBUG] auto-key toggle 0x{:X} (pressed={})",
                        k, auto_key_pressed
                    );
                }
            }
        }

        // --- tick CPU ---
        if now - last_tick >= TICK_INTERVAL {
            emu.tick();
            instr_count += 1;

            // flush pending releases AFTER the tick — guarantees each press
            // is visible for at least one tick, even if press+release
            // happens between two ticks.
            for i in 0..16 {
                if release_pending[i] {
                    emu.key_up(i as u8);
                    release_pending[i] = false;
                    if debug {
                        eprintln!("[DEBUG]   -> delayed key_up(0x{:X})", i);
                    }
                }
            }

            last_tick = now;
        }

        // --- timers ---
        if now - last_timer >= TIMER_INTERVAL {
            emu.tick_timers();
            last_timer = now;
        }

        // --- render ---
        if now - last_frame >= Duration::from_millis(33) {
            let buf = emu.get_display();
            let w = emu.get_display_width();
            let h = emu.get_display_height();
            let screen = render_display(&buf, w, h);

            let frame_duration = now - last_frame;
            let display_fps = if frame_duration.as_secs_f64() > 0.0 {
                (1.0 / frame_duration.as_secs_f64()) as u64
            } else {
                0
            };
            let sound_label = if emu.get_sound() && (app_start.elapsed().as_millis() / 120) % 2 == 0
            {
                "BEEP"
            } else {
                "    "
            };

            info = if show_help {
                "\r\n  ┌─ KEYMAP ───────────────────────────┐\
                 \r\n  │  HEX KEYPAD         PC KEY          │\
                 \r\n  │    1  2  3  C        1  2  3  4     │\
                 \r\n  │    4  5  6  D        Q  W  E  R     │\
                 \r\n  │    7  8  9  E        A  S  D  F     │\
                 \r\n  │    A  0  B  F        Z  X  C  V     │\
                 \r\n  │                                      │\
                 \r\n  │  Controls:                           │\
                 \r\n  │    ESC/Ctrl+C  Quit                  │\
                 \r\n  │    R           Reset                 │\
                 \r\n  │    H/F1        Toggle this help      │\
                 \r\n  └──────────────────────────────────────┘"
                    .to_string()
            } else {
                format!(
                    "  {}  instr={}  fps={}  PC=${:04X}  I=${:04X}  DT={}  ST={}\
                     \r\n  SOUND: {}",
                    rom_name,
                    instr_count,
                    display_fps,
                    emu.get_register_pc(),
                    emu.get_register_i(),
                    emu.get_delay(),
                    emu.get_sound_remaining(),
                    sound_label,
                )
            };

            queue!(stdout(), MoveTo(0, 0)).unwrap();
            draw_screen(&screen, &info);
            last_frame = now;
        }

        std::thread::sleep(Duration::from_millis(1));
    }

    let _ = disable_raw_mode();
    if enhanced_keyboard {
        let _ = execute!(stdout(), PopKeyboardEnhancementFlags);
    }
    let _ = execute!(stdout(), Show, Clear(ClearType::All));
    println!("Quit after {} instructions.", instr_count);

    if debug {
        eprintln!(
            "[DEBUG] Final PC=${:04X} I=${:04X} V0=${:02X}",
            emu.get_register_pc(),
            emu.get_register_i(),
            emu.get_register_v(0)
        );
    }
}
