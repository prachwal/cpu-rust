#![no_std]

mod reg;

/// MOS 6821 PIA — binary-compatible emulation.
///
/// ## Register map (4 addresses per PIA)
/// | Addr&3 | CRA/CRB bit 2=0 | CRA/CRB bit 2=1 |
/// |--------|------------------|------------------|
/// |    0   | DDRA (r/w)      | ORA (r/w)        |
/// |    1   | CRA             | CRA              |
/// |    2   | DDRB (r/w)      | ORB (r/w)        |
/// |    3   | CRB             | CRB              |
///
/// ## IRQ flags
/// Read ORA/ORB (when control bit 2 = 1) clears pending IRQ flags.
/// Reading the data direction register (when control bit 2 = 0) does NOT.
pub struct Pia6821 {
    ora: u8,
    orb: u8,
    ddra: u8,
    ddrb: u8,
    cra: u8,
    crb: u8,

    // input latches (visible on DDR=0 bits)
    input_a: u8,
    input_b: u8,

    // line state
    ca1: bool,
    ca2: bool,
    cb1: bool,
    cb2: bool,

    // CA2/CB2 output driven by PIA (for output modes)
    ca2_out: bool,
    cb2_out: bool,

    // IRQ output state (derived from enabled pending flags)
    irq_a: bool,
    irq_b: bool,
}

impl Pia6821 {
    pub fn new() -> Self {
        Pia6821 {
            ora: 0, orb: 0,
            ddra: 0, ddrb: 0,
            cra: 0, crb: 0,
            input_a: 0, input_b: 0,
            ca1: false, ca2: false, cb1: false, cb2: false,
            ca2_out: true, cb2_out: true,
            irq_a: false, irq_b: false,
        }
    }

    // ── Register read ──

    pub fn read(&mut self, addr: u16, port_a_input: u8, port_b_input: u8) -> u8 {
        match addr & 3 {
            0 => self.read_port_a(port_a_input),
            1 => self.read_cra(),
            2 => self.read_port_b(port_b_input),
            3 => self.read_crb(),
            _ => 0,
        }
    }

    fn read_port_a(&mut self, pin_input: u8) -> u8 {
        if reg::cra_ddr_sel(self.cra) {
            // CRA bit 2 = 1: read ORA
            // Reading ORA clears IRQ A1 and IRQ A2 flags
            self.cra &= !reg::IRQ_A1_MASK;
            self.cra &= !reg::IRQ_A2_MASK;
            self.update_irq_a();
            (self.ora & self.ddra) | (pin_input & !self.ddra)
        } else {
            // CRA bit 2 = 0: read DDRA
            self.ddra
        }
    }

    fn read_port_b(&mut self, pin_input: u8) -> u8 {
        if reg::crb_ddr_sel(self.crb) {
            // CRB bit 2 = 1: read ORB
            // Reading ORB clears IRQ B1 and IRQ B2 flags
            self.crb &= !reg::IRQ_B1_MASK;
            self.crb &= !reg::IRQ_B2_MASK;
            self.update_irq_b();
            (self.orb & self.ddrb) | (pin_input & !self.ddrb)
        } else {
            // CRB bit 2 = 0: read DDRB
            self.ddrb
        }
    }

    // ── Control register read — preserves flag bits ──

    fn read_cra(&self) -> u8 {
        self.cra
    }

    fn read_crb(&self) -> u8 {
        self.crb
    }

    // ── Register write ──

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr & 3 {
            0 => self.write_port_a(val),
            1 => self.write_cra(val),
            2 => self.write_port_b(val),
            3 => self.write_crb(val),
            _ => {}
        }
    }

    fn write_port_a(&mut self, val: u8) {
        if reg::cra_ddr_sel(self.cra) {
            // CRA bit 2 = 1: write ORA
            self.ora = val;
            self.update_ca2_handshake();
        } else {
            // CRA bit 2 = 0: write DDRA
            self.ddra = val;
        }
    }

    fn write_port_b(&mut self, val: u8) {
        if reg::crb_ddr_sel(self.crb) {
            // CRB bit 2 = 1: write ORB
            self.orb = val;
            self.update_cb2_handshake();
        } else {
            // CRB bit 2 = 0: write DDRB
            self.ddrb = val;
        }
    }

    fn write_cra(&mut self, val: u8) {
        // Only bits 0-5 are writable; bits 6-7 (IRQ flags) are read-only.
        self.cra = (self.cra & reg::CRA_FLAG_MASK) | (val & reg::CRA_WRITE_MASK);
        self.apply_ca2_mode();
        self.update_irq_a();
    }

    fn write_crb(&mut self, val: u8) {
        self.crb = (self.crb & reg::CRB_FLAG_MASK) | (val & reg::CRB_WRITE_MASK);
        self.apply_cb2_mode();
        self.update_irq_b();
    }

    // ── Pin setters (called by external hardware) ──

    /// Drive CA1 input line.
    pub fn set_ca1(&mut self, level: bool) {
        if level == self.ca1 { return; }
        let edge = level; // true = rising edge
        if edge == reg::cra_ca1_active(self.cra) {
            self.cra |= reg::IRQ_A1_MASK;
            self.update_irq_a();
        }
        self.ca1 = level;
    }

    /// Drive CA2 input line (only relevant when CA2 is in input mode).
    pub fn set_ca2(&mut self, level: bool) {
        if !reg::ca2_is_output(self.cra) {
            if level == self.ca2 { return; }
            let edge = level;
            if edge == reg::cra_ca2_active(self.cra) {
                self.cra |= reg::IRQ_A2_MASK;
                self.update_irq_a();
            }
        }
        self.ca2 = level;
    }

    /// Drive CB1 input line.
    pub fn set_cb1(&mut self, level: bool) {
        if level == self.cb1 { return; }
        let edge = level;
        if edge == reg::crb_cb1_active(self.crb) {
            self.crb |= reg::IRQ_B1_MASK;
            self.update_irq_b();
        }
        self.cb1 = level;
    }

    /// Drive CB2 input line (only relevant when CB2 is in input mode).
    pub fn set_cb2(&mut self, level: bool) {
        if !reg::cb2_is_output(self.crb) {
            if level == self.cb2 { return; }
            let edge = level;
            if edge == reg::crb_cb2_active(self.crb) {
                self.crb |= reg::IRQ_B2_MASK;
                self.update_irq_b();
            }
        }
        self.cb2 = level;
    }

    /// Set DDRA (data direction register A) directly.
    pub fn set_ddra(&mut self, val: u8) { self.ddra = val; }

    /// Set DDRB (data direction register B) directly.
    pub fn set_ddrb(&mut self, val: u8) { self.ddrb = val; }

    /// Set external pin values on port A (affects DDR=0 input bits).
    pub fn set_input_a(&mut self, val: u8) {
        self.input_a = val;
    }

    /// Set external pin values on port B (affects DDR=0 input bits).
    pub fn set_input_b(&mut self, val: u8) {
        self.input_b = val;
    }

    // ── Output getters (for external hardware) ──

    /// Value driven on port A output pins (DDR=1 bits only).
    pub fn output_a(&self) -> u8 {
        self.ora & self.ddra
    }

    /// Value driven on port B output pins (DDR=1 bits only).
    pub fn output_b(&self) -> u8 {
        self.orb & self.ddrb
    }

    /// CA2 output line level (meaningful when CA2 is in output mode).
    pub fn ca2_output(&self) -> bool {
        self.ca2_out
    }

    /// CB2 output line level (meaningful when CB2 is in output mode).
    pub fn cb2_output(&self) -> bool {
        self.cb2_out
    }

    /// IRQ A output — asserted when any enabled port A interrupt flag is pending.
    pub fn irq_a(&self) -> bool {
        self.irq_a
    }

    /// IRQ B output — asserted when any enabled port B interrupt flag is pending.
    pub fn irq_b(&self) -> bool {
        self.irq_b
    }

    // ── Internal helpers ──

    fn update_irq_a(&mut self) {
        let pending = (self.cra & reg::IRQ_A1_MASK != 0 && reg::cra_irq_a1_enabled(self.cra))
                    || (self.cra & reg::IRQ_A2_MASK != 0 && reg::cra_irq_a2_enabled(self.cra));
        self.irq_a = pending;
    }

    fn update_irq_b(&mut self) {
        let pending = (self.crb & reg::IRQ_B1_MASK != 0 && reg::crb_irq_b1_enabled(self.crb))
                    || (self.crb & reg::IRQ_B2_MASK != 0 && reg::crb_irq_b2_enabled(self.crb));
        self.irq_b = pending;
    }

    fn apply_ca2_mode(&mut self) {
        match reg::ca2_mode(self.cra) {
            // manual output
            m if m == 0b010 || m == 0b011 => {
                self.ca2_out = reg::cra_ca2_bit(self.cra);
            }
            // pulse or handshake — output goes low on read/write, restored by CA1
            _ => {}
        }
    }

    fn apply_cb2_mode(&mut self) {
        match reg::cb2_mode(self.crb) {
            m if m == 0b010 || m == 0b011 => {
                self.cb2_out = reg::crb_cb2_bit(self.crb);
            }
            _ => {}
        }
    }

    fn update_ca2_handshake(&mut self) {
        // In pulse or handshake output modes, writing ORA drives CA2 low.
        match reg::ca2_mode(self.cra) {
            0b100 | 0b101 | 0b110 | 0b111 => {
                self.ca2_out = false;
            }
            _ => {}
        }
    }

    fn update_cb2_handshake(&mut self) {
        match reg::cb2_mode(self.crb) {
            0b100 | 0b101 | 0b110 | 0b111 => {
                self.cb2_out = false;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
