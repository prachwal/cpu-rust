pub struct Cpu {
    pub v: [u8; 16],
    pub i: u16,
    pub pc: u16,
    pub sp: u8,
    pub stack: [u16; 16],
    pub delay: u8,
    pub sound: u8,
    pub halted: bool,
    pub halt_reg: u8,
    // simple LCG for CXNN (RND) — no external dep needed
    rng_seed: u16,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            v: [0; 16],
            i: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
            delay: 0,
            sound: 0,
            halted: false,
            halt_reg: 0,
            rng_seed: 0xACE1,
        }
    }

    pub fn reset(&mut self) {
        self.v = [0; 16];
        self.i = 0;
        self.pc = 0x200;
        self.sp = 0;
        self.stack = [0; 16];
        self.delay = 0;
        self.sound = 0;
        self.halted = false;
        self.halt_reg = 0;
        self.rng_seed = 0xACE1;
    }

    pub fn rand_byte(&mut self) -> u8 {
        // LCG: seed = seed * 75 + 74, return high byte
        self.rng_seed = self.rng_seed.wrapping_mul(75).wrapping_add(74);
        (self.rng_seed >> 8) as u8
    }

    pub fn tick_timers(&mut self) {
        if self.delay > 0 {
            self.delay -= 1;
        }
        if self.sound > 0 {
            self.sound -= 1;
        }
    }

    pub fn stack_push(&mut self, addr: u16) {
        if (self.sp as usize) < self.stack.len() {
            self.stack[self.sp as usize] = addr;
            self.sp += 1;
        }
    }

    pub fn stack_pop(&mut self) -> u16 {
        if self.sp > 0 {
            self.sp -= 1;
            self.stack[self.sp as usize]
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let cpu = Cpu::new();
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.sp, 0);
    }

    #[test]
    fn test_reset() {
        let mut cpu = Cpu::new();
        cpu.v[0] = 42;
        cpu.pc = 0x300;
        cpu.reset();
        assert_eq!(cpu.v[0], 0);
        assert_eq!(cpu.pc, 0x200);
    }

    #[test]
    fn test_stack() {
        let mut cpu = Cpu::new();
        cpu.stack_push(0x234);
        assert_eq!(cpu.sp, 1);
        cpu.stack_push(0x456);
        assert_eq!(cpu.sp, 2);
        assert_eq!(cpu.stack_pop(), 0x456);
        assert_eq!(cpu.sp, 1);
        assert_eq!(cpu.stack_pop(), 0x234);
        assert_eq!(cpu.sp, 0);
    }

    #[test]
    fn test_timers() {
        let mut cpu = Cpu::new();
        cpu.delay = 10;
        cpu.sound = 5;
        cpu.tick_timers();
        assert_eq!(cpu.delay, 9);
        assert_eq!(cpu.sound, 4);
        cpu.tick_timers();
        assert_eq!(cpu.delay, 8);
        assert_eq!(cpu.sound, 3);
    }

    #[test]
    fn test_timers_dont_underflow() {
        let mut cpu = Cpu::new();
        cpu.tick_timers();
        assert_eq!(cpu.delay, 0);
        assert_eq!(cpu.sound, 0);
    }
}
