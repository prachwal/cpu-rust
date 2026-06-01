use cpu_bus::Bus;
use cpu_display::Display;
use cpu_keyboard::Keyboard;
use crate::cpu::Cpu;

pub fn draw_sprite(display: &mut Display, x: u8, y: u8, sprite: &[u8]) -> bool {
    let mut collision = false;
    let w = display.width();
    let h = display.height();
    for (row, &byte) in sprite.iter().enumerate() {
        if row as u16 >= h { break; }
        let py = (y.wrapping_add(row as u8)) % (h as u8);
        for bit in 0..8 {
            if byte & (0x80 >> bit) == 0 { continue; }
            let px = (x.wrapping_add(bit)) % (w as u8);
            let old = display.get_pixel(px, py);
            display.set_pixel(px, py, old ^ 1);
            if old != 0 { collision = true; }
        }
    }
    collision
}

pub struct Quirks {
    pub shift_vy: bool,
    pub memory_inc_i: bool,
    pub vf_reset: bool,
}

pub fn execute(
    opcode: u16,
    cpu: &mut Cpu,
    memory: &mut impl Bus,
    display: &mut Display,
    keyboard: &Keyboard,
    quirks: &Quirks,
) {
    let x = ((opcode & 0x0F00) >> 8) as u8;
    let y = ((opcode & 0x00F0) >> 4) as u8;
    let n = (opcode & 0x000F) as u8;
    let nn = (opcode & 0x00FF) as u8;
    let nnn = opcode & 0x0FFF;

    match opcode & 0xF000 {
        0x0000 => match opcode & 0x00FF {
            0x00E0 => display.clear(),
            0x00EE => { cpu.pc = cpu.stack_pop(); }
            _ => {}
        },
        0x1000 => { cpu.pc = nnn; }
        0x2000 => { cpu.stack_push(cpu.pc); cpu.pc = nnn; }
        0x3000 => { if cpu.v[x as usize] == nn { cpu.pc += 2; } }
        0x4000 => { if cpu.v[x as usize] != nn { cpu.pc += 2; } }
        0x5000 => { if cpu.v[x as usize] == cpu.v[y as usize] { cpu.pc += 2; } }
        0x6000 => { cpu.v[x as usize] = nn; }
        0x7000 => { cpu.v[x as usize] = cpu.v[x as usize].wrapping_add(nn); }
        0x8000 => match n {
            0x0 => cpu.v[x as usize] = cpu.v[y as usize],
            0x1 => { cpu.v[x as usize] |= cpu.v[y as usize]; if quirks.vf_reset { cpu.v[0xF] = 0; } }
            0x2 => { cpu.v[x as usize] &= cpu.v[y as usize]; if quirks.vf_reset { cpu.v[0xF] = 0; } }
            0x3 => { cpu.v[x as usize] ^= cpu.v[y as usize]; if quirks.vf_reset { cpu.v[0xF] = 0; } }
            0x4 => {
                let (result, carry) = cpu.v[x as usize].overflowing_add(cpu.v[y as usize]);
                cpu.v[x as usize] = result;
                cpu.v[0xF] = if carry { 1 } else { 0 };
            }
            0x5 => {
                let (result, borrow) = cpu.v[x as usize].overflowing_sub(cpu.v[y as usize]);
                cpu.v[x as usize] = result;
                cpu.v[0xF] = if borrow { 0 } else { 1 };
            }
            0x6 => {
                if quirks.shift_vy {
                    cpu.v[0xF] = cpu.v[y as usize] & 1;
                    cpu.v[x as usize] = cpu.v[y as usize] >> 1;
                } else {
                    cpu.v[0xF] = cpu.v[x as usize] & 1;
                    cpu.v[x as usize] >>= 1;
                }
            }
            0x7 => {
                let (result, borrow) = cpu.v[y as usize].overflowing_sub(cpu.v[x as usize]);
                cpu.v[x as usize] = result;
                cpu.v[0xF] = if borrow { 0 } else { 1 };
            }
            0xE => {
                if quirks.shift_vy {
                    cpu.v[0xF] = (cpu.v[y as usize] >> 7) & 1;
                    cpu.v[x as usize] = cpu.v[y as usize] << 1;
                } else {
                    cpu.v[0xF] = (cpu.v[x as usize] >> 7) & 1;
                    cpu.v[x as usize] <<= 1;
                }
            }
            _ => {}
        },
        0x9000 => { if cpu.v[x as usize] != cpu.v[y as usize] { cpu.pc += 2; } }
        0xA000 => { cpu.i = nnn; }
        0xB000 => { cpu.pc = nnn + cpu.v[0] as u16; }
        0xC000 => { cpu.v[x as usize] = cpu.rand_byte() & nn; }
        0xD000 => {
            let h = n;
            let mut sprite = Vec::with_capacity(h as usize);
            for row in 0..h {
                sprite.push(memory.read(cpu.i + row as u16));
            }
            let coll = draw_sprite(display, cpu.v[x as usize], cpu.v[y as usize], &sprite);
            cpu.v[0xF] = if coll { 1 } else { 0 };
        }
        0xE000 => match opcode & 0x00FF {
            0x009E => { if keyboard.is_pressed(cpu.v[x as usize] & 0x0F) { cpu.pc += 2; } }
            0x00A1 => { if !keyboard.is_pressed(cpu.v[x as usize] & 0x0F) { cpu.pc += 2; } }
            _ => {}
        },
        0xF000 => match opcode & 0x00FF {
            0x0007 => { cpu.v[x as usize] = cpu.delay; }
            0x000A => {
                if let Some(key) = keyboard.any_pressed() {
                    cpu.v[x as usize] = key;
                } else {
                    cpu.pc -= 2;
                }
            }
            0x0015 => { cpu.delay = cpu.v[x as usize]; }
            0x0018 => { cpu.sound = cpu.v[x as usize]; }
            0x001E => { cpu.i = cpu.i.wrapping_add(cpu.v[x as usize] as u16); }
            0x0029 => { let digit = cpu.v[x as usize] & 0x0F; cpu.i = 0x050 + (digit as u16) * 5; }
            0x0033 => {
                let val = cpu.v[x as usize];
                memory.write(cpu.i, val / 100);
                memory.write(cpu.i + 1, (val / 10) % 10);
                memory.write(cpu.i + 2, val % 10);
            }
            0x0055 => {
                for reg in 0..=x as usize { memory.write(cpu.i + reg as u16, cpu.v[reg]); }
                if quirks.memory_inc_i { cpu.i += x as u16 + 1; }
            }
            0x0065 => {
                for reg in 0..=x as usize { cpu.v[reg] = memory.read(cpu.i + reg as u16); }
                if quirks.memory_inc_i { cpu.i += x as u16 + 1; }
            }
            _ => {}
        },
        _ => {}
    }
}
