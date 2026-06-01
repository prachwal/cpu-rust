#![no_std]

mod reg;

pub const IRQ_CA2: u8 = 1 << 0;
pub const IRQ_CA1: u8 = 1 << 1;
pub const IRQ_SR:  u8 = 1 << 2;
pub const IRQ_CB2: u8 = 1 << 3;
pub const IRQ_CB1: u8 = 1 << 4;
pub const IRQ_T2:  u8 = 1 << 5;
pub const IRQ_T1:  u8 = 1 << 6;
pub const IRQ_ALL: u8 = 0x7F;

pub struct Via6522 {
    // port output latches
    ora: u8,
    orb: u8,
    // data direction registers
    ddra: u8,
    ddrb: u8,
    // control registers
    acr: u8,
    pcr: u8,
    // interrupt registers
    ifr: u8,
    ier: u8,
    // shift register
    sr: u8,

    // input latches (from external pins)
    input_a: u8,
    input_b: u8,

    // timer 1
    pub t1_counter: u16,
    t1_latch: u16,
    t1_pb7: bool,

    // timer 2
    pub t2_counter: u16,
    t2_latch: u16,
    t2_pb6_count: bool,

    // CA/CB line state
    ca1: bool, ca2: bool, ca2_out: bool,
    cb1: bool, cb2: bool, cb2_out: bool,

    // shift register state
    sr_bits: u8,
    sr_count: u8,

    // IRQ output
    pub irq: bool,
}

impl Via6522 {
    pub fn new() -> Self {
        Via6522 {
            ora: 0, orb: 0,
            ddra: 0, ddrb: 0,
            acr: 0, pcr: 0,
            ifr: 0, ier: 0,
            sr: 0,
            input_a: 0, input_b: 0,
            t1_counter: 0, t1_latch: 0xFFFF, t1_pb7: false,
            t2_counter: 0, t2_latch: 0xFFFF, t2_pb6_count: false,
            ca1: false, ca2: false, ca2_out: true,
            cb1: false, cb2: false, cb2_out: true,
            sr_bits: 0, sr_count: 0,
            irq: false,
        }
    }

    // ── Public helpers for external wiring ──

    pub fn port_a_output(&self) -> u8 { self.ora & self.ddra }
    pub fn port_b_output(&self) -> u8 { self.orb & self.ddrb }

    /// Set external pin values on port A (for DDRA=0 bits)
    pub fn set_input_a(&mut self, val: u8) { self.input_a = val; }
    pub fn set_input_b(&mut self, val: u8) { self.input_b = val; }

    /// Convenience: trigger CA1 edge (for machine integration)
    pub fn trigger_ca1(&mut self) { self.set_ca1(true); self.set_ca1(false); }
    /// Convenience: trigger CB1 edge (for machine integration)
    pub fn trigger_cb1(&mut self) { self.set_cb1(true); self.set_cb1(false); }

    /// CA1 edge input
    pub fn set_ca1(&mut self, level: bool) {
        if level == self.ca1 { return; }
        let edge = level; // true=rising, false=falling
        if edge == reg::ca1_rising(self.pcr) {
            self.ifr_set(IRQ_CA1);
            self.ca2_handshake_restore();
        }
        self.ca1 = level;
    }

    /// CA2 input (meaningful when CA2 is in input mode)
    pub fn set_ca2(&mut self, level: bool) {
        if reg::ca2_is_input(self.pcr) {
            if level == self.ca2 { return; }
            if level { self.ifr_set(IRQ_CA2); }
        }
        self.ca2 = level;
    }

    /// CB1 edge input
    pub fn set_cb1(&mut self, level: bool) {
        if level == self.cb1 { return; }
        let edge = level;
        if edge == reg::cb1_rising(self.pcr) {
            self.ifr_set(IRQ_CB1);
            self.cb2_handshake_restore();
        }
        self.cb1 = level;
    }

    /// CB2 input (meaningful when CB2 is in input mode)
    pub fn set_cb2(&mut self, level: bool) {
        if reg::cb2_is_input(self.pcr) {
            if level == self.cb2 { return; }
            if level { self.ifr_set(IRQ_CB2); }
        }
        self.cb2 = level;
    }

    /// CA2 output level (when CA2 is in output mode)
    pub fn ca2_output(&self) -> bool { self.ca2_out }
    pub fn cb2_output(&self) -> bool { self.cb2_out }

    // ── IFR/IER helpers (public for tests) ──

    pub fn ifr_set(&mut self, bits: u8) {
        self.ifr |= bits & IRQ_ALL;
        self.update_irq();
    }

    pub fn ifr_clear(&mut self, bits: u8) {
        self.ifr &= !(bits & IRQ_ALL);
        self.update_irq();
    }

    // ── Register read ──

    pub fn read(&mut self, addr: u16) -> u8 {
        match (addr & 0x0F) as usize {
            0 => self.read_portb(),
            1 => self.read_porta(false),
            2 => self.ddrb,
            3 => self.ddra,
            4 => (self.t1_counter & 0xFF) as u8,
            5 => { let h = (self.t1_counter >> 8) as u8; self.ifr_clear(IRQ_T1); h }
            6 => (self.t1_latch & 0xFF) as u8,
            7 => (self.t1_latch >> 8) as u8,
            8 => (self.t2_counter & 0xFF) as u8,
            9 => { let h = (self.t2_counter >> 8) as u8; self.ifr_clear(IRQ_T2); h }
            10 => self.sr_read(),
            11 => self.acr,
            12 => self.pcr,
            13 => self.ifr_read(),
            14 => self.ier | 0x80,
            15 => self.read_porta(true),
            _ => 0,
        }
    }

    fn read_porta(&mut self, no_handshake: bool) -> u8 {
        if !no_handshake && reg::ca2_is_handshake(self.pcr) {
            self.ifr_clear(IRQ_CA1);
            self.ca2_handshake_strobe();
        }
        if self.acr & 0x01 != 0 {
            // input latch enabled
            (self.ora & self.ddra) | (self.input_a & !self.ddra)
        } else {
            (self.ora & self.ddra) | (self.input_a & !self.ddra)
        }
    }

    fn read_portb(&mut self) -> u8 {
        if reg::cb2_is_handshake(self.pcr) {
            self.ifr_clear(IRQ_CB1);
            self.cb2_handshake_strobe();
        }
        (self.orb & self.ddrb) | (self.input_b & !self.ddrb)
    }

    fn ifr_read(&mut self) -> u8 {
        let v = self.ifr;
        // IFR read does NOT clear all flags on real 6522
        // (only specific source clears work)
        v
    }

    fn sr_read(&mut self) -> u8 {
        self.ifr_clear(IRQ_SR);
        self.sr
    }

    // ── Register write ──

    pub fn write(&mut self, addr: u16, val: u8) {
        match (addr & 0x0F) as usize {
            0 => self.write_portb(val),
            1 | 15 => self.write_porta(val, false),
            2 => self.ddrb = val,
            3 => self.ddra = val,
            4 => self.t1_write_lo(val),
            5 => self.t1_write_hi(val),
            6 => self.t1_latch = (self.t1_latch & 0xFF00) | val as u16,
            7 => self.t1_latch = (self.t1_latch & 0x00FF) | ((val as u16) << 8),
            8 => self.t2_write_lo(val),
            9 => self.t2_write_hi(val),
            10 => self.sr_write(val),
            11 => self.acr = val,
            12 => { self.pcr = val; self.apply_pcr(); }
            13 => self.ifr_write(val),
            14 => self.ier_write(val),
            _ => {}
        }
    }

    fn write_porta(&mut self, val: u8, _no_handshake: bool) {
        self.ora = val;
        if reg::ca2_is_handshake(self.pcr) {
            self.ifr_clear(IRQ_CA1);
            self.ca2_handshake_strobe();
        }
    }

    fn write_portb(&mut self, val: u8) {
        self.orb = val;
        if reg::cb2_is_handshake(self.pcr) {
            self.ifr_clear(IRQ_CB1);
            self.cb2_handshake_strobe();
        }
    }

    fn sr_write(&mut self, val: u8) {
        self.sr = val;
        self.sr_bits = 0;
        self.sr_count = 0;
        self.ifr_clear(IRQ_SR);
    }

    fn ifr_write(&mut self, val: u8) {
        // IFR write clears selected bits
        self.ifr &= !(val & IRQ_ALL);
        self.update_irq();
    }

    fn ier_write(&mut self, val: u8) {
        if val & 0x80 != 0 {
            self.ier |= val & IRQ_ALL;
        } else {
            self.ier &= !(val & IRQ_ALL);
        }
        self.update_irq();
    }

    // ── Timer 1 ──

    fn t1_write_lo(&mut self, val: u8) {
        self.t1_latch = (self.t1_latch & 0xFF00) | val as u16;
    }

    fn t1_write_hi(&mut self, val: u8) {
        self.t1_latch = (self.t1_latch & 0x00FF) | ((val as u16) << 8);
        self.t1_counter = self.t1_latch;
        self.ifr_clear(IRQ_T1);
        self.t1_pb7 = false;
    }

    // ── Timer 2 ──

    fn t2_write_lo(&mut self, val: u8) {
        self.t2_latch = (self.t2_latch & 0xFF00) | val as u16;
    }

    fn t2_write_hi(&mut self, val: u8) {
        self.t2_latch = (self.t2_latch & 0x00FF) | ((val as u16) << 8);
        self.t2_counter = self.t2_latch;
        self.ifr_clear(IRQ_T2);
        self.t2_pb6_count = (self.acr & 0x20) != 0;
    }

    // ── PB6 pulse-count input ──

    pub fn set_pb6(&mut self, _level: bool) {
        if !self.t2_pb6_count { return; }
        if self.t2_counter > 0 {
            self.t2_counter -= 1;
            if self.t2_counter == 0 { self.ifr_set(IRQ_T2); }
        }
    }

    // ── Cycle tick ──

    pub fn tick(&mut self, cycles: u64) -> bool {
        let t1_free = (self.acr & 0x80) != 0;
        for _ in 0..cycles {
            // Timer 1
            if self.t1_counter > 0 {
                self.t1_counter -= 1;
                if self.t1_counter == 0 {
                    self.ifr_set(IRQ_T1);
                    self.t1_pb7 = !self.t1_pb7;
                    if t1_free { self.t1_counter = self.t1_latch; }
                }
            }
            // Timer 2 (timed mode only)
            if !self.t2_pb6_count && self.t2_counter > 0 {
                self.t2_counter -= 1;
                if self.t2_counter == 0 { self.ifr_set(IRQ_T2); }
            }
            // Shift register (simplified: immediate shift on SR write)
        }
        self.irq
    }

    // ── PB7 output ──

    /// PB7 level: when ACR bit 7=1 (timer output enabled), PB7 toggles with T1.
    /// When disabled, PB7 is controlled by ORB/DDRB.
    pub fn pb7_output(&self) -> bool {
        if self.acr & 0x80 != 0 {
            self.t1_pb7
        } else {
            (self.orb & self.ddrb & 0x80) != 0
        }
    }

    // ── Internal helpers ──

    fn update_irq(&mut self) {
        let pending = self.ifr & self.ier & IRQ_ALL;
        if pending != 0 {
            self.ifr |= 0x80;
            self.irq = true;
        } else {
            self.ifr &= 0x7F;
            self.irq = false;
        }
    }

    fn apply_pcr(&mut self) {
        // CA2 output mode
        if reg::ca2_is_input(self.pcr) {
            self.ca2_out = true; // input mode: hi-Z / pulled up
        } else if reg::ca2_is_handshake(self.pcr) {
            self.ca2_out = true; // rest state high
        } else if reg::ca2_is_pulse(self.pcr) {
            self.ca2_out = true; // rest state high
        } else {
            self.ca2_out = reg::ca2_manual_level(self.pcr);
        }
        // CB2 output mode
        if reg::cb2_is_input(self.pcr) {
            self.cb2_out = true;
        } else if reg::cb2_is_handshake(self.pcr) {
            self.cb2_out = true;
        } else if reg::cb2_is_pulse(self.pcr) {
            self.cb2_out = true;
        } else {
            self.cb2_out = reg::cb2_manual_level(self.pcr);
        }
    }

    fn ca2_handshake_strobe(&mut self) {
        if reg::ca2_is_handshake(self.pcr) || reg::ca2_is_pulse(self.pcr) {
            self.ca2_out = false;
        }
    }

    fn cb2_handshake_strobe(&mut self) {
        if reg::cb2_is_handshake(self.pcr) || reg::cb2_is_pulse(self.pcr) {
            self.cb2_out = false;
        }
    }

    fn ca2_handshake_restore(&mut self) {
        if reg::ca2_is_handshake(self.pcr) {
            self.ca2_out = true;
        }
        if reg::ca2_is_pulse(self.pcr) {
            self.ca2_out = true;
        }
    }

    fn cb2_handshake_restore(&mut self) {
        if reg::cb2_is_handshake(self.pcr) {
            self.cb2_out = true;
        }
        if reg::cb2_is_pulse(self.pcr) {
            self.cb2_out = true;
        }
    }
}

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
