/// 7-segment LED display patterns and pixel rendering.
///
/// Segment bit positions (bit0=A, bit1=B, bit2=C, bit3=D, bit4=E, bit5=F, bit6=G):
/// ```
///   AAA
///  F   B
///  F   B
///   GGG
///  E   C
///  E   C
///   DDD
/// ```

/// Hex digit → segment bitmask (bit0=A … bit6=G)
pub const HEX_PATTERNS: [u8; 16] = [
    0x3F, // 0: ABCDEF
    0x06, // 1: BC
    0x5B, // 2: ABDEG
    0x4F, // 3: ABCD G
    0x66, // 4: BCFG
    0x6D, // 5: ACDFG
    0x7D, // 6: ACDEFG
    0x07, // 7: ABC
    0x7F, // 8: all
    0x6F, // 9: ABCDFG
    0x77, // A: ABCEFG
    0x7C, // b: CDEFG
    0x39, // C: ADEF
    0x5E, // d: BCDEG
    0x79, // E: ADEFG
    0x71, // F: AEFG
];

/// Blank (all segments off)
pub const BLANK: u8 = 0x00;
/// Minus sign (segment G only)
pub const MINUS: u8 = 0x40;
/// Degree symbol (segments A, B, F, G)
pub const DEGREE: u8 = 0x63;

/// Render one 7-segment digit into a pixel buffer.
pub fn render(
    segments: u8,
    buf: &mut [u8],
    stride: usize,
    digit_w: usize,
    digit_h: usize,
) {
    let on = |bit: u8| -> u8 { if segments & (1 << bit) != 0 { 1 } else { 0 } };
    let w = digit_w;
    let h = digit_h;
    if w < 5 || h < 7 { return; }
    let mid = h / 2;
    let gap_l = 2;
    let gap_r = w.saturating_sub(3);

    if h > 0 { for x in gap_l..gap_r { buf[0 * stride + x] = on(0); } }       // A
    for y in 1..mid {
        buf[y * stride + 0] = on(5); buf[y * stride + 1] = on(5);              // F
        if gap_r < w { buf[y * stride + gap_r] = on(1); if gap_r+1<w { buf[y*stride+gap_r+1]=on(1); } } // B
    }
    if mid < h { for x in gap_l..gap_r { buf[mid * stride + x] = on(6); } }    // G
    for y in (mid + 1)..(h - 1) {
        buf[y * stride + 0] = on(4); buf[y * stride + 1] = on(4);              // E
        if gap_r < w { buf[y * stride + gap_r] = on(2); if gap_r+1<w { buf[y*stride+gap_r+1]=on(2); } } // C
    }
    if h > 1 { for x in gap_l..gap_r { buf[(h - 1) * stride + x] = on(3); } } // D
}

/// Render a hex digit into a pixel buffer.
pub fn render_hex(digit: u8, buf: &mut [u8], stride: usize, w: usize, h: usize) {
    let pattern = HEX_PATTERNS[(digit & 0x0F) as usize];
    render(pattern, buf, stride, w, h);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patterns_have_correct_count() { assert_eq!(HEX_PATTERNS.len(), 16); }

    #[test]
    fn test_pattern_0_has_A() {
        assert!(HEX_PATTERNS[0] & 0x01 != 0, "0 should have segment A (bit0)");
    }

    #[test]
    fn test_pattern_0_no_G() {
        assert_eq!(HEX_PATTERNS[0] & 0x40, 0, "0 should not have segment G");
    }

    #[test]
    fn test_pattern_1_only_BC() {
        assert_eq!(HEX_PATTERNS[1] & 0x06, 0x06, "1 should have B+C only");
        assert_eq!(HEX_PATTERNS[1] & 0x01, 0, "1 should not have A");
    }

    #[test]
    fn test_pattern_8_all_on() {
        assert_eq!(HEX_PATTERNS[8], 0x7F, "8 should have all segments");
    }

    #[test]
    fn test_render_all_on() {
        let mut buf = [0u8; 13 * 13];
        render(0x7F, &mut buf, 13, 13, 13);
        assert!(buf[0 * 13 + 5] != 0);  // A
        assert!(buf[6 * 13 + 5] != 0);  // G
        assert!(buf[12 * 13 + 5] != 0); // D
    }

    #[test]
    fn test_render_all_off() {
        let mut buf = [0u8; 13 * 13];
        render(0x00, &mut buf, 13, 13, 13);
        assert!(buf.iter().all(|&p| p == 0));
    }
}
