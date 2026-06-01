// ── PCR bit positions and mode helpers ──

// CA1 control (PCR bits 0)
pub const PCR_CA1_ACTIVE: u8 = 0x01;

// CA2 control (PCR bits 3..1)
pub const PCR_CA2_MODE: u8   = 0x0E; // bits 1-3
pub const PCR_CA2_INPUT: u8  = 0x00; // 000
pub const PCR_CA2_IND: u8    = 0x02; // 001 independent interrupt input
pub const PCR_CA2_PULSE: u8  = 0x08; // 100 pulse output
pub const PCR_CA2_MAN_L: u8  = 0x0A; // 101 manual low
pub const PCR_CA2_MAN_H: u8  = 0x0C; // 110 manual high
pub const PCR_CA2_HS: u8     = 0x0E; // 111 handshake output

// CB1 control (PCR bit 4)
pub const PCR_CB1_ACTIVE: u8 = 0x10;

// CB2 control (PCR bits 7..5)
pub const PCR_CB2_MODE: u8   = 0xE0; // bits 5-7
pub const PCR_CB2_INPUT: u8  = 0x00; // 000
pub const PCR_CB2_IND: u8    = 0x20; // 001 independent interrupt input
pub const PCR_CB2_PULSE: u8  = 0x80; // 100 pulse output
pub const PCR_CB2_MAN_L: u8  = 0xA0; // 101 manual low
pub const PCR_CB2_MAN_H: u8  = 0xC0; // 110 manual high
pub const PCR_CB2_HS: u8     = 0xE0; // 111 handshake output

pub fn ca1_rising(pcr: u8) -> bool { pcr & PCR_CA1_ACTIVE != 0 }
pub fn cb1_rising(pcr: u8) -> bool { pcr & PCR_CB1_ACTIVE != 0 }

pub fn ca2_is_input(pcr: u8) -> bool { (pcr & PCR_CA2_MODE) <= PCR_CA2_IND }
pub fn cb2_is_input(pcr: u8) -> bool { (pcr & PCR_CB2_MODE) <= PCR_CB2_IND }

pub fn ca2_manual_level(pcr: u8) -> bool {
    match pcr & PCR_CA2_MODE {
        PCR_CA2_MAN_H => true,
        PCR_CA2_MAN_L => false,
        _ => true, // default high
    }
}

pub fn cb2_manual_level(pcr: u8) -> bool {
    match pcr & PCR_CB2_MODE {
        PCR_CB2_MAN_H => true,
        PCR_CB2_MAN_L => false,
        _ => true,
    }
}

pub fn ca2_is_handshake(pcr: u8) -> bool { (pcr & PCR_CA2_MODE) == PCR_CA2_HS }
pub fn cb2_is_handshake(pcr: u8) -> bool { (pcr & PCR_CB2_MODE) == PCR_CB2_HS }
pub fn ca2_is_pulse(pcr: u8) -> bool { (pcr & PCR_CA2_MODE) == PCR_CA2_PULSE }
pub fn cb2_is_pulse(pcr: u8) -> bool { (pcr & PCR_CB2_MODE) == PCR_CB2_PULSE }
