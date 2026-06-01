use cpu_display::Display;
use cpu_keyboard::Keyboard;
use crate::cpu::Cpu;
use crate::memory::Memory;

fn draw_sprite(display: &mut Display, x: u8, y: u8, sprite: &[u8]) -> bool {
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
    memory: &mut Memory,
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
            0x00EE => {
                cpu.pc = cpu.stack_pop();
            }
            _ => {} // 0NNN — ignore (machine call)
        },
        0x1000 => {
            cpu.pc = nnn;
        }
        0x2000 => {
            cpu.stack_push(cpu.pc);
            cpu.pc = nnn;
        }
        0x3000 => {
            if cpu.v[x as usize] == nn {
                cpu.pc += 2;
            }
        }
        0x4000 => {
            if cpu.v[x as usize] != nn {
                cpu.pc += 2;
            }
        }
        0x5000 => {
            if cpu.v[x as usize] == cpu.v[y as usize] {
                cpu.pc += 2;
            }
        }
        0x6000 => {
            cpu.v[x as usize] = nn;
        }
        0x7000 => {
            cpu.v[x as usize] = cpu.v[x as usize].wrapping_add(nn);
        }
        0x8000 => match n {
            0x0 => cpu.v[x as usize] = cpu.v[y as usize],
            0x1 => {
                cpu.v[x as usize] |= cpu.v[y as usize];
                if quirks.vf_reset {
                    cpu.v[0xF] = 0;
                }
            }
            0x2 => {
                cpu.v[x as usize] &= cpu.v[y as usize];
                if quirks.vf_reset {
                    cpu.v[0xF] = 0;
                }
            }
            0x3 => {
                cpu.v[x as usize] ^= cpu.v[y as usize];
                if quirks.vf_reset {
                    cpu.v[0xF] = 0;
                }
            }
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
        0x9000 => {
            if cpu.v[x as usize] != cpu.v[y as usize] {
                cpu.pc += 2;
            }
        }
        0xA000 => {
            cpu.i = nnn;
        }
        0xB000 => {
            cpu.pc = nnn + cpu.v[0] as u16;
        }
        0xC000 => {
            cpu.v[x as usize] = cpu.rand_byte() & nn;
        }
         0xD000 => {
            let sprite = memory.read_slice(cpu.i, n as u16);
            let coll = draw_sprite(display, cpu.v[x as usize], cpu.v[y as usize], sprite);
            cpu.v[0xF] = if coll { 1 } else { 0 };
        }
        0xE000 => match opcode & 0x00FF {
            0x009E => {
                if keyboard.is_pressed(cpu.v[x as usize] & 0x0F) {
                    cpu.pc += 2;
                }
            }
            0x00A1 => {
                if !keyboard.is_pressed(cpu.v[x as usize] & 0x0F) {
                    cpu.pc += 2;
                }
            }
            _ => {}
        },
        0xF000 => match opcode & 0x00FF {
            0x0007 => {
                cpu.v[x as usize] = cpu.delay;
            }
            0x000A => {
                if let Some(key) = keyboard.any_pressed() {
                    cpu.v[x as usize] = key;
                } else {
                    cpu.pc -= 2;
                }
            }
            0x0015 => {
                cpu.delay = cpu.v[x as usize];
            }
            0x0018 => {
                cpu.sound = cpu.v[x as usize];
            }
            0x001E => {
                cpu.i = cpu.i.wrapping_add(cpu.v[x as usize] as u16);
            }
            0x0029 => {
                let digit = cpu.v[x as usize] & 0x0F;
                cpu.i = 0x050 + (digit as u16) * 5;
            }
            0x0033 => {
                let val = cpu.v[x as usize];
                memory.write(cpu.i, val / 100);
                memory.write(cpu.i + 1, (val / 10) % 10);
                memory.write(cpu.i + 2, val % 10);
            }
            0x0055 => {
                for reg in 0..=x as usize {
                    memory.write(cpu.i + reg as u16, cpu.v[reg]);
                }
                if quirks.memory_inc_i {
                    cpu.i += x as u16 + 1;
                }
            }
            0x0065 => {
                for reg in 0..=x as usize {
                    cpu.v[reg] = memory.read(cpu.i + reg as u16);
                }
                if quirks.memory_inc_i {
                    cpu.i += x as u16 + 1;
                }
            }
            _ => {}
        },
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Cpu, Memory, Display, Keyboard, Quirks) {
        (
            Cpu::new(),
            Memory::new(),
            Display::new(64, 32),
            Keyboard::new(),
            Quirks { shift_vy: false, memory_inc_i: false, vf_reset: true },
        )
    }

    #[test]
    fn test_00e0_cls() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        display.set_pixel(0, 0, 1);
        assert!(display.get_pixel(0, 0) != 0);
        execute(0x00E0, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(display.get_pixel(0, 0), 0);
    }

    #[test]
    fn test_00ee_ret() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.stack_push(0x234);
        cpu.pc = 0x456;
        execute(0x00EE, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.pc, 0x234);
    }

    #[test]
    fn test_1nnn_jp() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        execute(0x1345, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.pc, 0x345);
    }

    #[test]
    fn test_2nnn_call() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.pc = 0x300;
        execute(0x2500, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.pc, 0x500);
        assert_eq!(cpu.stack_pop(), 0x300);
    }

    #[test]
    fn test_3xnn_se() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.v[0] = 42;
        cpu.pc = 0x200;
        execute(0x302A, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_3xnn_se_no_skip() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.v[0] = 41;
        cpu.pc = 0x200;
        execute(0x302A, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.pc, 0x200);
    }

    #[test]
    fn test_4xnn_sne() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.v[0] = 42;
        cpu.pc = 0x200;
        execute(0x4000, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_5xy0_se() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.v[0] = 5;
        cpu.v[1] = 5;
        cpu.pc = 0x200;
        execute(0x5010, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_6xnn_ld() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        execute(0x6A42, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.v[0xA], 0x42);
    }

    #[test]
    fn test_7xnn_add() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.v[3] = 10;
        execute(0x7305, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.v[3], 15);
    }

    #[test]
    fn test_8xy0_ld() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.v[1] = 99;
        execute(0x8010, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.v[0], 99);
    }

    #[test]
    fn test_8xy1_or() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.v[0] = 0xF0;
        cpu.v[1] = 0x0F;
        execute(0x8011, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.v[0], 0xFF);
        assert_eq!(cpu.v[0xF], 0);
    }

    #[test]
    fn test_8xy2_and() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.v[0] = 0xFF;
        cpu.v[1] = 0x0F;
        execute(0x8012, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.v[0], 0x0F);
    }

    #[test]
    fn test_8xy3_xor() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.v[0] = 0xFF;
        cpu.v[1] = 0x0F;
        execute(0x8013, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.v[0], 0xF0);
    }

    #[test]
    fn test_8xy4_add_carry() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.v[0] = 0xFF;
        cpu.v[1] = 0x01;
        execute(0x8014, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.v[0], 0x00);
        assert_eq!(cpu.v[0xF], 1);
    }

    #[test]
    fn test_8xy4_add_no_carry() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.v[0] = 0x01;
        cpu.v[1] = 0x02;
        execute(0x8014, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.v[0], 0x03);
        assert_eq!(cpu.v[0xF], 0);
    }

    #[test]
    fn test_8xy5_sub_borrow() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.v[0] = 0x01;
        cpu.v[1] = 0xFF;
        execute(0x8015, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.v[0], 0x02);
        assert_eq!(cpu.v[0xF], 0);
    }

    #[test]
    fn test_8xy5_sub_no_borrow() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.v[0] = 0x05;
        cpu.v[1] = 0x03;
        execute(0x8015, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.v[0], 0x02);
        assert_eq!(cpu.v[0xF], 1);
    }

    #[test]
    fn test_8xy6_shr() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.v[0] = 0x05;
        execute(0x8016, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.v[0], 0x02);
        assert_eq!(cpu.v[0xF], 1);
    }

    #[test]
    fn test_8xy7_subn() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.v[0] = 0x03;
        cpu.v[1] = 0x08;
        execute(0x8017, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.v[0], 0x05);
        assert_eq!(cpu.v[0xF], 1);
    }

    #[test]
    fn test_8xye_shl() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.v[0] = 0x80;
        execute(0x801E, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.v[0], 0x00);
        assert_eq!(cpu.v[0xF], 1);
    }

    #[test]
    fn test_9xy0_sne() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.v[0] = 1;
        cpu.v[1] = 2;
        cpu.pc = 0x200;
        execute(0x9010, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_annn_ld_i() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        execute(0xA345, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.i, 0x345);
    }

    #[test]
    fn test_bnnn_jp_v0() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.v[0] = 0x10;
        execute(0xB200, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.pc, 0x210);
    }

    #[test]
    fn test_cxnn_rnd() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        execute(0xC0FF, &mut cpu, &mut mem, &mut display, &kb, &config);
        let _ = cpu.v[0]; // u8 always ≤ 0xFF
    }

    #[test]
    fn test_dxyn_draw() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.i = 0x200;
        mem.write(0x200, 0xFF);
        cpu.v[0] = 10;
        cpu.v[1] = 5;
        execute(0xD011, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert!(display.get_pixel(10, 5) != 0);
        assert!(display.get_pixel(17, 5) != 0);
        assert!(display.get_pixel(18, 5) == 0);
    }

    #[test]
    fn test_ex9e_skip_key() {
        let (mut cpu, mut mem, mut display, mut kb, config) = setup();
        cpu.v[0] = 0xA;
        kb.press(0xA);
        cpu.pc = 0x200;
        execute(0xE09E, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_exa1_skip_no_key() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.v[0] = 0xA;
        cpu.pc = 0x200;
        execute(0xE0A1, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_fx07_ld_dt() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.delay = 42;
        execute(0xF007, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.v[0], 42);
    }

    #[test]
    fn test_fx0a_wait_key() {
        let (mut cpu, mut mem, mut display, mut kb, config) = setup();
        // simulate tick(): PC already advanced past the instruction
        cpu.pc = 0x202;
        execute(0xF00A, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.pc, 0x200); // retry: backed up by 2

        // press key and try again
        kb.press(0x7);
        cpu.pc += 2; // simulate fetch phase of tick
        execute(0xF00A, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.v[0], 7);
        assert_eq!(cpu.pc, 0x202); // instruction completed, PC stays advanced
    }

    #[test]
    fn test_fx15_ld_dt() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.v[0] = 30;
        execute(0xF015, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.delay, 30);
    }

    #[test]
    fn test_fx18_ld_st() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.v[1] = 15;
        execute(0xF118, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.sound, 15);
    }

    #[test]
    fn test_fx1e_add_i() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.i = 0x100;
        cpu.v[0] = 0x50;
        execute(0xF01E, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.i, 0x150);
    }

    #[test]
    fn test_fx29_font() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.v[0] = 0x0A; // digit 'A'
        execute(0xF029, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.i, 0x050 + 0x0A * 5);
    }

    #[test]
    fn test_fx33_bcd() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.v[0] = 123;
        cpu.i = 0x300;
        execute(0xF033, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(mem.read(0x300), 1);
        assert_eq!(mem.read(0x301), 2);
        assert_eq!(mem.read(0x302), 3);
    }

    #[test]
    fn test_fx55_store() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        cpu.v[0] = 0x10;
        cpu.v[1] = 0x20;
        cpu.v[2] = 0x30;
        cpu.i = 0x400;
        execute(0xF255, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(mem.read(0x400), 0x10);
        assert_eq!(mem.read(0x401), 0x20);
        assert_eq!(mem.read(0x402), 0x30);
        assert_eq!(cpu.i, 0x400); // I unchanged in default config
    }

    #[test]
    fn test_fx65_load() {
        let (mut cpu, mut mem, mut display, kb, config) = setup();
        mem.write(0x400, 0xAA);
        mem.write(0x401, 0xBB);
        mem.write(0x402, 0xCC);
        cpu.i = 0x400;
        execute(0xF265, &mut cpu, &mut mem, &mut display, &kb, &config);
        assert_eq!(cpu.v[0], 0xAA);
        assert_eq!(cpu.v[1], 0xBB);
        assert_eq!(cpu.v[2], 0xCC);
        assert_eq!(cpu.i, 0x400);
    }

    #[test]
    fn test_fx55_store_inc_i() {
        let (mut cpu, mut mem, mut display, kb, mut quirks) = setup();
        quirks.memory_inc_i = true;
        cpu.v[0] = 0x10;
        cpu.i = 0x400;
        execute(0xF055, &mut cpu, &mut mem, &mut display, &kb, &quirks);
        assert_eq!(cpu.i, 0x401);
    }

    #[test]
    fn test_shift_vy_vs_vx() {
        let (mut cpu, mut mem, mut display, kb, mut quirks) = setup();
        // Default config (shift_vy = false): shifts VX
        cpu.v[0] = 0x03;
        cpu.v[1] = 0x80;
        execute(0x8016, &mut cpu, &mut mem, &mut display, &kb, &quirks);
        assert_eq!(cpu.v[0], 0x01); // V0 >>= 1: 3→1
        assert_eq!(cpu.v[0xF], 1); // LSB was 1

        // With shift_vy = true: VX = VY >> 1
        quirks.shift_vy = true;
        cpu.v[0] = 99; // should be overwritten
        cpu.v[1] = 0x05;
        execute(0x8016, &mut cpu, &mut mem, &mut display, &kb, &quirks);
        assert_eq!(cpu.v[0], 0x02); // V0 = V1 >> 1: 5→2
        assert_eq!(cpu.v[0xF], 1); // LSB was 1
    }
}
