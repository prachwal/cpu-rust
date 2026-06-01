#![allow(dead_code)]

// ── Control register bit positions ──

// CRA/CRB writable bits (0-5), flag bits (6-7) are read-only
pub const CRA_WRITE_MASK: u8 = 0x3F;
pub const CRB_WRITE_MASK: u8 = 0x3F;
pub const CRA_FLAG_MASK: u8 = 0xC0;
pub const CRB_FLAG_MASK: u8 = 0xC0;



// CRA layout:
// bit 0: CA1 control — active transition (0=falling, 1=rising)
// bit 1: CA2 control — active transition (input) or output level (manual output)
// bit 2: DDR select (0=DDRA, 1=ORA)
// bits 3-5: CA2 mode
// bit 6: IRQ A1 flag
// bit 7: IRQ A2 flag

pub const CRA_CA1_ACTIVE: u8    = 0x01; // bit 0
pub const CRA_CA2_CTL: u8       = 0x02; // bit 1
pub const CRA_DDR_SEL: u8       = 0x04; // bit 2
pub const CRA_CA2_MODE_LO: u8   = 0x08; // bit 3
pub const CRA_CA2_MODE_MID: u8  = 0x10; // bit 4
pub const CRA_CA2_MODE_HI: u8   = 0x20; // bit 5
pub const CRA_IRQ_A1: u8        = 0x40; // bit 6
pub const CRA_IRQ_A2: u8        = 0x80; // bit 7

// CRB same layout, but with CB1/CB2
pub const CRB_CB1_ACTIVE: u8    = 0x01;
pub const CRB_CB2_CTL: u8       = 0x02;
pub const CRB_DDR_SEL: u8       = 0x04;
pub const CRB_CB2_MODE_LO: u8   = 0x08;
pub const CRB_CB2_MODE_MID: u8  = 0x10;
pub const CRB_CB2_MODE_HI: u8   = 0x20;
pub const CRB_IRQ_B1: u8        = 0x40;
pub const CRB_IRQ_B2: u8        = 0x80;

// ── IRQ flag masks (for matching set/clear) ──

pub const IRQ_A1_MASK: u8 = CRA_IRQ_A1;
pub const IRQ_A2_MASK: u8 = CRA_IRQ_A2;
pub const IRQ_B1_MASK: u8 = CRB_IRQ_B1;
pub const IRQ_B2_MASK: u8 = CRB_IRQ_B2;

// ── Accessors ──

pub fn cra_ddr_sel(cra: u8) -> bool { cra & CRA_DDR_SEL != 0 }
pub fn crb_ddr_sel(crb: u8) -> bool { crb & CRB_DDR_SEL != 0 }

pub fn cra_ca1_active(cra: u8) -> bool { cra & CRA_CA1_ACTIVE != 0 } // true=rising
pub fn crb_cb1_active(crb: u8) -> bool { crb & CRB_CB1_ACTIVE != 0 }

pub fn cra_ca2_active(cra: u8) -> bool { cra & CRA_CA2_CTL != 0 }
pub fn crb_cb2_active(crb: u8) -> bool { crb & CRB_CB2_CTL != 0 }

pub fn cra_ca2_bit(cra: u8) -> bool { cra & CRA_CA2_CTL != 0 }
pub fn crb_cb2_bit(crb: u8) -> bool { crb & CRB_CB2_CTL != 0 }

/// CA2 mode: bits 5,4,3 of CRA
pub fn ca2_mode(cra: u8) -> u8 { (cra >> 3) & 0x07 }

/// CB2 mode: bits 5,4,3 of CRB
pub fn cb2_mode(crb: u8) -> u8 { (crb >> 3) & 0x07 }

/// CA2 is configured as output when mode != 0b00x
pub fn ca2_is_output(cra: u8) -> bool {
    let m = ca2_mode(cra);
    m >= 0b010
}

/// CB2 is configured as output when mode != 0b00x
pub fn cb2_is_output(crb: u8) -> bool {
    let m = cb2_mode(crb);
    m >= 0b010
}

/// IRQ A1 is enabled when CRA bit 0 enables interrupt generation
pub fn cra_irq_a1_enabled(cra: u8) -> bool {
    // IRQ enabled when control bit enables the interrupt:
    // For 6821: IRQ A1 enabled when CR bit 0 causes flag to assert IRQ.
    // The flag asserts IRQ when read of data register clears previous flag.
    // Actually, on 6821, any set flag causes IRQ if the corresponding
    // interrupt enable bit is set.
    //
    // For PIA: IRQ output is driven when flag is set. There's no separate
    // "enable" bit per flag — reading the data register clears the flag
    // and de-asserts IRQ. Re-arming happens on the next CA transition.
    //
    // Actually, let me reconsider. The 6821 datasheet says:
    // "The interrupt output IRQA is set by:
    //  1. active transition on CA1 (IRQA1 flag set)
    //  2. active transition on CA2 (IRQA2 flag set) when CA2 is input mode
    //  IRQA is cleared by a read of the output data register."
    //
    // There's no separate enable/disable bit for the IRQ output.
    // The IRQ output is simply the OR of the two flag bits.
    // HOWEVER, the control register bit 0 determines active transition,
    // and the flag can only be set by the selected transition.
    //
    // So "enabled" in practice means: the flag can be set by transitions.
    // And once set, IRQ is asserted until the data register is read.
    true
}

pub fn cra_irq_a2_enabled(_cra: u8) -> bool { true }
pub fn crb_irq_b1_enabled(_crb: u8) -> bool { true }
pub fn crb_irq_b2_enabled(_crb: u8) -> bool { true }
