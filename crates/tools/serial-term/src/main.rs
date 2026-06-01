use std::io::{stdin, stdout, Read, Write};
use std::time::{Duration, Instant};

use cpu_machine::SerialMachine;
use crossterm::cursor::{Hide, Show};
use crossterm::event::{poll, read, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use crossterm::{execute, queue};

const STATUS_INTERVAL: Duration = Duration::from_millis(100);
const PIPE_BURST: u64 = 100_000;

fn create_machine(kind: &str, rom_file: Option<&str>) -> Box<dyn SerialMachine> {
    match kind {
        "eater" => {
            let rom: Vec<u8> = if let Some(path) = rom_file {
                let data = std::fs::read(path).expect("Cannot read ROM file");
                let mut full = vec![0xFF; 0x8000];
                let len = data.len().min(0x8000);
                full[..len].copy_from_slice(&data[..len]);
                full
            } else {
                eater_6502::rom::generate_monitor()
            };
            Box::new(eater_6502::Eater6502::new(rom))
        }
        _ => panic!("Unknown machine: {kind}. Supported: eater"),
    }
}

fn status_line(m: &dyn SerialMachine, instr: u64, key: Option<u8>) -> String {
    let last = key.map(|b| if b.is_ascii_graphic() || b == b' ' { b as char } else { '?' });
    format!(
        "PC=${:04X}  A=${:02X}  X=${:02X}  Y=${:02X}  SP=${:02X}  P={:08b}  instr={}  last={:?}",
        m.pc(), m.a(), m.x(), m.y(), m.sp(), m.p(), instr, last,
    )
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut machine_kind = "eater";
    let mut rom_file: Option<String> = None;
    let mut load_file: Option<(String, u16)> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--machine" => { i += 1; machine_kind = &args[i]; }
            "--rom" => { i += 1; rom_file = Some(args[i].clone()); }
            "--load" => {
                i += 1;
                let path = args[i].clone();
                let addr = if i + 1 < args.len() && !args[i+1].starts_with('-') {
                    i += 1;
                    u16::from_str_radix(args[i].trim_start_matches("0x"), 16).expect("Invalid address")
                } else { 0x0200 };
                load_file = Some((path, addr));
            }
            h if h == "-h" || h == "--help" => {
                println!("serial-term -- Universal serial terminal for CPU emulators");
                println!();
                println!("USAGE:");
                println!("  serial-term [OPTIONS]");
                println!();
                println!("OPTIONS:");
                println!("  --machine <kind>    Machine type (default: eater)");
                println!("  --rom <file>        Custom ROM file");
                println!("  --load <file> [addr] Load binary at address (default 0x0200)");
                println!();
                println!("KEYS:");
                println!("  Any printable char   Send to machine");
                println!("  Enter                Send CR (0x0D)");
                println!("  Backspace            Send BS (0x08)");
                println!("  Ctrl-C / ESC         Quit");
                return;
            }
            _ => {}
        }
        i += 1;
    }

    let is_tty = atty::is(atty::Stream::Stdin);
    if is_tty {
        enable_raw_mode().unwrap();
        execute!(stdout(), Hide, Clear(ClearType::All)).unwrap();
    }

    // Non-blocking stdin for pipe mode
    if !is_tty {
        use std::os::unix::io::AsRawFd;
        let fd = stdin().as_raw_fd();
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK); }
    }

    let mut machine = create_machine(machine_kind, rom_file.as_deref());

    let mut instr_count: u64 = 0;

    if let Some((_path, _addr)) = load_file {
        eprintln!("Warning: --load not yet supported (use --rom)");
    }

    // Warm up: run init before accepting input
    for _ in 0..200 { machine.tick(); instr_count += 1; }

    let mut last_status = Instant::now();
    let mut running = true;
    let mut done = false;
    let mut last_key: Option<u8> = None;
    let mut stdin_buf: Vec<u8> = Vec::new();
    let mut guard = [0u8; 1024];

    // Main loop
    while running {
        // ── Input ──
        if is_tty {
            while poll(Duration::ZERO).unwrap() {
                match read().unwrap() {
                    Event::Key(ke) if ke.kind == KeyEventKind::Press => {
                        match ke.code {
                            KeyCode::Esc => running = false,
                            KeyCode::Char('c') if ke.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => running = false,
                            KeyCode::Enter => {
                                last_key = Some(0x0D);
                                machine.serial_send(0x0D);
                            }
                            KeyCode::Backspace => {
                                last_key = Some(0x08);
                                machine.serial_send(0x08);
                            }
                            KeyCode::Char(ch) => {
                                last_key = Some(ch as u8);
                                machine.serial_send(ch as u8);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        } else if !done {
            loop {
                match stdin().read(&mut guard) {
                    Ok(0) => { done = true; break; }
                    Ok(n) => { stdin_buf.extend_from_slice(&guard[..n]); }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(e) => { eprintln!("stdin error: {e}"); running = false; break; }
                }
            }
        }

        // ── Execute ──
        let burst = if done { PIPE_BURST.min(stdin_buf.len().max(1) as u64 * 500) } else { 1 };
        for _ in 0..burst {
            // Feed a byte only when ACIA is ready
            if !stdin_buf.is_empty() && machine.serial_send_ready() {
                let b = stdin_buf.remove(0);
                last_key = Some(b);
                machine.serial_send(b);
            }

            machine.tick();
            instr_count += 1;

            // Drain serial output
            while let Some(byte) = machine.serial_recv() {
                let _ = stdout().write(&[byte]);
            }
        }

        // ── Flush ──
        stdout().flush().unwrap();

        // ── Status ──
        let now = Instant::now();
        if is_tty && now - last_status >= STATUS_INTERVAL {
            let line = status_line(&*machine, instr_count, last_key);
            queue!(stdout(), Clear(ClearType::CurrentLine)).unwrap();
            print!("\r{}", line);
            stdout().flush().unwrap();
            last_status = now;
        }

        // ── Termination (pipe mode) ──
        if done && stdin_buf.is_empty() {
            // Final drain
            for _ in 0..1000 {
                machine.tick();
                while let Some(byte) = machine.serial_recv() {
                    let _ = stdout().write(&[byte]);
                }
            }
            running = false;
        }

        // ── Yield ──
        if !is_tty {
            std::thread::sleep(Duration::from_micros(100));
        } else {
            std::thread::sleep(Duration::from_micros(10));
        }
    }

    // Restore terminal
    if is_tty {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), Show, Clear(ClearType::All));
    }
    println!();
    println!("serial-term: {} instructions, {} cycles.", instr_count, machine.cycles());
}
