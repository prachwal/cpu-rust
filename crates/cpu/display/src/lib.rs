// ── Display Configuration ──

/// Describes the physical pixel layout and color capabilities.
#[derive(Clone, Debug)]
pub struct DisplayConfig {
    /// Logical width in pixels (e.g. 176 for VIC-20, 64 for CHIP-8)
    pub width: u32,
    /// Logical height in pixels (e.g. 184 for VIC-20, 32 for CHIP-8)
    pub height: u32,
    /// RGBA palette: index → [r, g, b, a]. First entry is usually background.
    pub palette: Vec<[u8; 4]>,
    /// Pixel aspect ratio width : height (e.g. (1,1) square, (8,7) VIC-20 NTSC)
    pub pixel_aspect: (u32, u32),
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            width: 64,
            height: 32,
            palette: vec![[0, 0, 0, 255], [255, 255, 255, 255]],
            pixel_aspect: (1, 1),
        }
    }
}

impl DisplayConfig {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height, ..Default::default() }
    }

    /// VIC-20 NTSC: 176×184, 16-colour palette, 8:7 pixel aspect
    pub fn vic20_ntsc() -> Self {
        Self {
            width: 176,
            height: 184,
            palette: VIC20_PALETTE.to_vec(),
            pixel_aspect: (8, 7),
        }
    }

    /// VIC-20 PAL: 176×184, 16-colour palette, 11:13 pixel aspect
    pub fn vic20_pal() -> Self {
        Self {
            width: 176,
            height: 184,
            palette: VIC20_PALETTE.to_vec(),
            pixel_aspect: (11, 13),
        }
    }

    /// C64: 320×200, 16-colour palette
    pub fn c64() -> Self {
        Self {
            width: 320,
            height: 200,
            palette: C64_PALETTE.to_vec(),
            pixel_aspect: (1, 1),
        }
    }

    /// CHIP-8: 64×32 monochrome
    pub fn chip8() -> Self {
        Self {
            width: 64,
            height: 32,
            palette: vec![[0, 0, 0, 255], [255, 255, 255, 255]],
            pixel_aspect: (1, 1),
        }
    }

    /// Apple 1: 40×24 characters at 5×7 px
    pub fn apple1() -> Self {
        Self {
            width: 200,  // 40 * 5
            height: 168, // 24 * 7
            palette: vec![[0, 0, 0, 255], [0, 200, 0, 255]],
            pixel_aspect: (1, 1),
        }
    }

    /// PET 2001: 40×25 characters (text mode)
    pub fn pet() -> Self {
        Self {
            width: 320,  // 40 * 8
            height: 200, // 25 * 8
            palette: vec![[0, 0, 0, 255], [0, 200, 0, 255]],
            pixel_aspect: (1, 1),
        }
    }

    pub fn palette_index(&self, color: u8) -> &[u8; 4] {
        let i = color as usize;
        if i < self.palette.len() { &self.palette[i] } else { &self.palette[0] }
    }
}

// ── Palettes ──
// VIC-20 / C64 16-colour palette (approximate)
pub const VIC20_PALETTE: [[u8; 4]; 16] = [
    [0, 0, 0, 255],       // 0 black
    [255, 255, 255, 255], // 1 white
    [160, 0, 0, 255],     // 2 red
    [0, 200, 255, 255],   // 3 cyan
    [160, 0, 200, 255],   // 4 purple
    [0, 200, 0, 255],     // 5 green
    [0, 0, 200, 255],     // 6 blue
    [200, 200, 0, 255],   // 7 yellow
    [200, 100, 0, 255],   // 8 orange
    [100, 60, 0, 255],    // 9 brown
    [200, 100, 100, 255], // 10 light red
    [80, 80, 80, 255],    // 11 dark grey
    [140, 140, 140, 255], // 12 grey
    [100, 200, 100, 255], // 13 light green
    [100, 100, 200, 255], // 14 light blue
    [160, 160, 160, 255], // 15 light grey
];

pub const C64_PALETTE: [[u8; 4]; 16] = VIC20_PALETTE;

// ── Font ──

/// How font data is organized.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FontDataFormat {
    /// Raw bitmap: each character is `char_width * char_height` bits,
    /// rows packed LSB-first (bit 0 = leftmost pixel).
    Raw,
    /// C64 / VIC-20 format: each char = 8 bytes, 8×8 pixels.
    /// Bit 7 = leftmost pixel, byte = one row, column-major within bytes.
    C64,
    /// PET 2001: similar to C64 but with different ordering / gaps.
    Pet,
}

/// How byte values map to character indices in the font.
#[derive(Clone, Debug)]
pub enum FontMapping {
    /// Byte value = font index directly (e.g. ASCII).
    Direct,
    /// PETSCII: maps PET-2001 screen codes to font indices.
    PetAscii,
    /// Apple 1: ASCII → shifted for display.
    Apple1,
    /// Custom mapping function.
    Custom(fn(u8) -> u8),
}

impl FontMapping {
    pub fn map(&self, byte: u8) -> u8 {
        match self {
            FontMapping::Direct => byte,
            FontMapping::PetAscii => petascii_to_idx(byte),
            FontMapping::Apple1 => apple1_to_idx(byte),
            FontMapping::Custom(f) => f(byte),
        }
    }
}

fn petascii_to_idx(byte: u8) -> u8 {
    // PETSCII mapping (simplified):
    // $00-$1F: screen codes for special chars → font indices
    // $20-$3F: shifted alphabet (uppercase) → font indices 0x20-0x3F
    // $40-$5F: lowercase → font indices 0x40-0x5F
    // $60-$7E: reverse video → font indices 0x60-0x7E
    // $80-$FF: shifted screen codes → font indices
    match byte {
        0x00..=0x1F => byte + 0x80, // special chars map to upper 128
        0x20..=0x3F => byte,        // punctuation / numbers
        0x40..=0x5F => byte - 0x20, // lowercase → uppercase
        0x60..=0x7E => byte - 0x40, // reverse → normal
        0x80..=0x9F => byte - 0x80, // shifted graphics
        0xA0..=0xBF => byte,        // shifted graphics
        0xC0..=0xFE => byte - 0x80, // shifted chars
        _ => 0,
    }
}

fn apple1_to_idx(byte: u8) -> u8 {
    // Apple 1: ASCII with bit 7 = inverse; display uses $00-$5F
    // Uppercase A-Z = $01-$1A, digits = $1B-$24, etc.
    match byte & 0x7F {
        0x20 => 0,                    // space
        b'A'..=b'Z' => byte - 0x40,  // A-Z → $01-$1A
        b'0'..=b'9' => byte - 0x25,  // 0-9 → $1B-$24
        _ => byte & 0x3F,
    }
}

/// A loaded bitmap font for text-mode rendering.
#[derive(Clone, Debug)]
pub struct Font {
    /// Raw pixel data for all characters.
    /// Layout: for each char, `char_height` bytes, each byte = one row of `char_width` bits.
    pub data: Vec<u8>,
    pub char_width: u8,
    pub char_height: u8,
    /// First character index in the data (e.g., 0x20 for space)
    pub first: u8,
    /// Last character index (inclusive)
    pub last: u8,
    /// Number of characters stored
    pub count: u16,
    /// Bytes per character row
    row_stride: u8,
}

impl Font {
    /// Load a raw bitmap font.
    ///
    /// Each character consists of `char_height` rows, each row is `char_width` bits
    /// packed MSB-first (bit 7 = leftmost pixel). Bits beyond `char_width` should be 0.
    ///
    /// `first` and `last` define the range of character codes covered.
    pub fn load_raw(data: &[u8], char_width: u8, char_height: u8, first: u8, last: u8) -> Self {
        let count = (last - first + 1) as u16;
        let bits_per_row = char_width;
        let row_bytes = (bits_per_row as u16 + 7) / 8;
        let expected = count as usize * char_height as usize * row_bytes as usize;
        assert!(data.len() >= expected, "Font data too short: {} < {}", data.len(), expected);

        Self {
            data: data.to_vec(),
            char_width,
            char_height,
            first,
            last,
            count,
            row_stride: row_bytes as u8,
        }
    }

    /// Load a C64/VIC-20 style font (8×8 pixels per char, 8 bytes per char).
    /// `data` should contain 256 or 128 characters × 8 bytes.
    /// Bit 7 = leftmost pixel, bit 0 = rightmost pixel.
    pub fn load_c64(data: &[u8], count: u16) -> Self {
        assert_eq!(data.len(), count as usize * 8, "C64 font: expected {} bytes", count * 8);
        Self {
            data: data.to_vec(),
            char_width: 8,
            char_height: 8,
            first: 0,
            last: (count - 1) as u8,
            count,
            row_stride: 1,
        }
    }

    /// Load a PET 2001 style font (8×8 pixels).
    /// PET char ROM has 512 chars × 8 bytes (first 256 normal, next 256 inverse).
    pub fn load_pet(data: &[u8]) -> Self {
        assert!(data.len() >= 512 * 8, "PET font: expected at least 4096 bytes");
        Self {
            data: data.to_vec(),
            char_width: 8,
            char_height: 8,
            first: 0,
            last: 255,
            count: 512,
            row_stride: 1,
        }
    }

    /// Generate an Apple 1 style 5×7 bitmap font for ASCII 0x20-0x5F (64 chars).
    /// Each character is 8 bytes (5×7 in 8×8 cell), MSB = leftmost pixel.
    /// Based on the 2513 character generator used in the Apple 1.
    pub fn apple1_5x7() -> Self {
        let count = 64;
        let mut data = vec![0u8; count * 8];
        // Each glyph: 7 data rows + 1 blank row, 5-bit wide, left-aligned in byte
        let glyphs: &[&[u8]] = &[
            // 0x20 space
            &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            // 0x21 !
            &[0x20, 0x20, 0x20, 0x20, 0x00, 0x20, 0x00, 0x00],
            // 0x22 "
            &[0x50, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            // 0x23 #
            &[0x50, 0xF8, 0x50, 0xF8, 0x50, 0x00, 0x00, 0x00],
            // 0x24 $
            &[0x20, 0x78, 0xA0, 0x70, 0x28, 0xF0, 0x20, 0x00],
            // 0x25 %
            &[0xC0, 0xC8, 0x10, 0x20, 0x40, 0x98, 0x18, 0x00],
            // 0x26 &
            &[0x40, 0xA0, 0xA0, 0x40, 0xA8, 0x90, 0x68, 0x00],
            // 0x27 '
            &[0x20, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            // 0x28 (
            &[0x10, 0x20, 0x40, 0x40, 0x40, 0x20, 0x10, 0x00],
            // 0x29 )
            &[0x40, 0x20, 0x10, 0x10, 0x10, 0x20, 0x40, 0x00],
            // 0x2A *
            &[0x00, 0x20, 0xA8, 0x70, 0xA8, 0x20, 0x00, 0x00],
            // 0x2B +
            &[0x00, 0x20, 0x20, 0xF8, 0x20, 0x20, 0x00, 0x00],
            // 0x2C ,
            &[0x00, 0x00, 0x00, 0x00, 0x30, 0x20, 0x40, 0x00],
            // 0x2D -
            &[0x00, 0x00, 0x00, 0xF8, 0x00, 0x00, 0x00, 0x00],
            // 0x2E .
            &[0x00, 0x00, 0x00, 0x00, 0x00, 0x60, 0x60, 0x00],
            // 0x2F /
            &[0x08, 0x10, 0x10, 0x20, 0x40, 0x40, 0x80, 0x00],
            // 0x30 0
            &[0x70, 0x88, 0x98, 0xA8, 0xC8, 0x88, 0x70, 0x00],
            // 0x31 1
            &[0x20, 0x60, 0x20, 0x20, 0x20, 0x20, 0x70, 0x00],
            // 0x32 2
            &[0x70, 0x88, 0x08, 0x30, 0x40, 0x80, 0xF8, 0x00],
            // 0x33 3
            &[0xF8, 0x10, 0x20, 0x10, 0x08, 0x88, 0x70, 0x00],
            // 0x34 4
            &[0x10, 0x30, 0x50, 0x90, 0xF8, 0x10, 0x10, 0x00],
            // 0x35 5
            &[0xF8, 0x80, 0xF0, 0x08, 0x08, 0x88, 0x70, 0x00],
            // 0x36 6
            &[0x70, 0x88, 0x80, 0xF0, 0x88, 0x88, 0x70, 0x00],
            // 0x37 7
            &[0xF8, 0x08, 0x10, 0x20, 0x40, 0x40, 0x40, 0x00],
            // 0x38 8
            &[0x70, 0x88, 0x88, 0x70, 0x88, 0x88, 0x70, 0x00],
            // 0x39 9
            &[0x70, 0x88, 0x88, 0x78, 0x08, 0x88, 0x70, 0x00],
            // 0x3A :
            &[0x00, 0x00, 0x60, 0x60, 0x00, 0x60, 0x60, 0x00],
            // 0x3B ;
            &[0x00, 0x20, 0x70, 0x20, 0x00, 0x60, 0x20, 0x40],
            // 0x3C <
            &[0x08, 0x10, 0x20, 0x40, 0x20, 0x10, 0x08, 0x00],
            // 0x3D =
            &[0x00, 0x00, 0xF8, 0x00, 0xF8, 0x00, 0x00, 0x00],
            // 0x3E >
            &[0x80, 0x40, 0x20, 0x10, 0x20, 0x40, 0x80, 0x00],
            // 0x3F ?
            &[0x70, 0x88, 0x08, 0x30, 0x20, 0x00, 0x20, 0x00],
            // 0x40 @
            &[0x70, 0x88, 0x08, 0x68, 0xA8, 0xA8, 0x70, 0x00],
            // 0x41 A
            &[0x70, 0x88, 0x88, 0xF8, 0x88, 0x88, 0x88, 0x00],
            // 0x42 B
            &[0xF0, 0x88, 0x88, 0xF0, 0x88, 0x88, 0xF0, 0x00],
            // 0x43 C
            &[0x70, 0x88, 0x80, 0x80, 0x80, 0x88, 0x70, 0x00],
            // 0x44 D
            &[0xE0, 0x90, 0x88, 0x88, 0x88, 0x90, 0xE0, 0x00],
            // 0x45 E
            &[0xF8, 0x80, 0x80, 0xF0, 0x80, 0x80, 0xF8, 0x00],
            // 0x46 F
            &[0xF8, 0x80, 0x80, 0xF0, 0x80, 0x80, 0x80, 0x00],
            // 0x47 G
            &[0x70, 0x88, 0x80, 0xB8, 0x88, 0x88, 0x78, 0x00],
            // 0x48 H
            &[0x88, 0x88, 0x88, 0xF8, 0x88, 0x88, 0x88, 0x00],
            // 0x49 I
            &[0x70, 0x20, 0x20, 0x20, 0x20, 0x20, 0x70, 0x00],
            // 0x4A J
            &[0x38, 0x10, 0x10, 0x10, 0x10, 0x90, 0x60, 0x00],
            // 0x4B K
            &[0x88, 0x90, 0xA0, 0xC0, 0xA0, 0x90, 0x88, 0x00],
            // 0x4C L
            &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0xF8, 0x00],
            // 0x4D M
            &[0x88, 0xD8, 0xA8, 0xA8, 0x88, 0x88, 0x88, 0x00],
            // 0x4E N
            &[0x88, 0xC8, 0xA8, 0x98, 0x88, 0x88, 0x88, 0x00],
            // 0x4F O
            &[0x70, 0x88, 0x88, 0x88, 0x88, 0x88, 0x70, 0x00],
            // 0x50 P
            &[0xF0, 0x88, 0x88, 0xF0, 0x80, 0x80, 0x80, 0x00],
            // 0x51 Q
            &[0x70, 0x88, 0x88, 0x88, 0xA8, 0x90, 0x68, 0x00],
            // 0x52 R
            &[0xF0, 0x88, 0x88, 0xF0, 0xA0, 0x90, 0x88, 0x00],
            // 0x53 S
            &[0x70, 0x88, 0x80, 0x70, 0x08, 0x88, 0x70, 0x00],
            // 0x54 T
            &[0xF8, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x00],
            // 0x55 U
            &[0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x70, 0x00],
            // 0x56 V
            &[0x88, 0x88, 0x88, 0x88, 0x50, 0x50, 0x20, 0x00],
            // 0x57 W
            &[0x88, 0x88, 0x88, 0xA8, 0xA8, 0xD8, 0x88, 0x00],
            // 0x58 X
            &[0x88, 0x88, 0x50, 0x20, 0x50, 0x88, 0x88, 0x00],
            // 0x59 Y
            &[0x88, 0x88, 0x50, 0x20, 0x20, 0x20, 0x20, 0x00],
            // 0x5A Z
            &[0xF8, 0x08, 0x10, 0x20, 0x40, 0x80, 0xF8, 0x00],
            // 0x5B [
            &[0x70, 0x40, 0x40, 0x40, 0x40, 0x40, 0x70, 0x00],
            // 0x5C \
            &[0x80, 0x40, 0x40, 0x20, 0x10, 0x10, 0x08, 0x00],
            // 0x5D ]
            &[0x70, 0x10, 0x10, 0x10, 0x10, 0x10, 0x70, 0x00],
            // 0x5E ^
            &[0x20, 0x50, 0x88, 0x00, 0x00, 0x00, 0x00, 0x00],
            // 0x5F _
            &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF8, 0x00],
        ];
        for (i, &glyph) in glyphs.iter().enumerate() {
            let base = i * 8;
            for (j, &row) in glyph.iter().enumerate() {
                data[base + j] = row;
            }
        }
        Self { data, char_width: 5, char_height: 7, first: 0x20, last: 0x5F, count: count as u16, row_stride: 1 }
    }

    /// Generate a minimal 8×8 bitmap font for ASCII 0x20-0x7F (96 chars).
    /// Each character is 8 bytes, one byte per row, MSB = leftmost pixel.
    pub fn ascii_8x8() -> Self {
        let count = 96;
        let mut data = vec![0u8; count * 8];
        for i in 0..count as u8 {
            let ch = i + 0x20;
            let base = i as usize * 8;
            match ch {
                0x20 => {} // space = blank
                // Simple box characters for all printable ASCII
                _ => {
                    // Top and bottom rows = borders
                    data[base] = 0xFF;
                    data[base + 7] = 0xFF;
                    // Middle rows: sides only, fill for dense chars
                    let filled = ch.is_ascii_uppercase() || ch.is_ascii_lowercase()
                        || ch.is_ascii_digit() || matches!(ch, b'@' | b'#' | b'$' | b'%' | b'&');
                    for row in 1..7 {
                        let r = if filled { 0xFF } else { 0x81 };
                        data[base + row as usize] = r;
                    }
                }
            }
        }
        Self { data, char_width: 8, char_height: 8, first: 0x20, last: 0x7F, count: 96, row_stride: 1 }
    }

    /// Get the pixel at (x, y) within character `ch`.
    /// `ch` is first mapped through the `first` offset.
    /// Returns `true` if the pixel is set.
    pub fn pixel(&self, ch: u8, x: u8, y: u8) -> bool {
        if x >= self.char_width || y >= self.char_height {
            return false;
        }
        // Clamp char to available range
        let idx = if ch < self.first { self.first } else if ch > self.last { self.last } else { ch };
        let char_index = (idx - self.first) as usize;
        let row = y as usize;
        let bit_pos = 7 - x; // MSB = leftmost pixel
        let byte_index = char_index * self.char_height as usize * self.row_stride as usize
            + row * self.row_stride as usize
            + bit_pos as usize / 8;
        let bit_mask = 1 << (bit_pos as u8 % 8);
        if byte_index < self.data.len() {
            self.data[byte_index] & bit_mask != 0
        } else {
            false
        }
    }

    /// Get a character row as a byte of pixel data, MSB-left.
    pub fn row_bits(&self, ch: u8, row: u8) -> u8 {
        if row >= self.char_height { return 0; }
        let idx = if ch < self.first { self.first } else if ch > self.last { self.last } else { ch };
        let char_index = (idx - self.first) as usize;
        let byte_index = char_index * self.char_height as usize * self.row_stride as usize
            + row as usize * self.row_stride as usize;
        if byte_index < self.data.len() {
            self.data[byte_index]
        } else {
            0
        }
    }
}

// ── Display ──

/// Operating mode of the display.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DisplayMode {
    /// Raw pixel graphics. `set_pixel` / `get_pixel` operate on pixel coordinates.
    Graphics,
    /// Text mode: grid of characters rendered via a font.
    /// Parameters: (cols, rows).
    Text(u16, u16),
}

/// Universal display buffer supporting graphics and text modes.
pub struct Display {
    config: DisplayConfig,
    mode: DisplayMode,

    // Graphics mode
    pixels: Vec<u8>, // palette index per pixel

    // Text mode
    chars: Vec<u8>,      // character code per cell
    char_fg: Vec<u8>,    // foreground colour index per cell
    char_bg: Vec<u8>,    // background colour index per cell
    cursor_row: u16,     // current write cursor row
    cursor_col: u16,     // current write cursor column

    // Font (for text mode)
    font: Option<Font>,
    mapping: FontMapping,

    // Rendered RGBA output
    rendered: Vec<u8>,
    dirty: bool,
}

impl Display {
    // ── Constructors ──

    /// Create a new display in graphics mode.
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            config: DisplayConfig::new(width as u32, height as u32),
            mode: DisplayMode::Graphics,
            pixels: vec![0; size],
            chars: Vec::new(),
            char_fg: Vec::new(),
            char_bg: Vec::new(),
            cursor_row: 0,
            cursor_col: 0,
            font: None,
            mapping: FontMapping::Direct,
            rendered: Vec::new(),
            dirty: true,
        }
    }

    pub fn from_config(config: DisplayConfig) -> Self {
        let size = (config.width as usize) * (config.height as usize);
        Self {
            config,
            mode: DisplayMode::Graphics,
            pixels: vec![0; size],
            chars: Vec::new(),
            char_fg: Vec::new(),
            char_bg: Vec::new(),
            cursor_row: 0,
            cursor_col: 0,
            font: None,
            mapping: FontMapping::Direct,
            rendered: Vec::new(),
            dirty: true,
        }
    }

    /// Create a text-mode display.
    pub fn new_text(config: DisplayConfig, cols: u16, rows: u16, font: Font, mapping: FontMapping) -> Self {
        let cell_count = (cols as usize) * (rows as usize);
        let buf_size = (config.width as usize) * (config.height as usize) * 4;
        Self {
            config,
            mode: DisplayMode::Text(cols, rows),
            pixels: Vec::new(),
            chars: vec![0x20; cell_count],
            char_fg: vec![1; cell_count],
            char_bg: vec![0; cell_count],
            cursor_row: 0,
            cursor_col: 0,
            font: Some(font),
            mapping,
            rendered: vec![0; buf_size],
            dirty: true,
        }
    }

    // ── Config accessors ──

    pub fn config(&self) -> &DisplayConfig { &self.config }

    pub fn mode(&self) -> DisplayMode { self.mode }

    pub fn font(&self) -> Option<&Font> { self.font.as_ref() }

    pub fn font_mapping(&self) -> &FontMapping { &self.mapping }

    pub fn set_font_mapping(&mut self, mapping: FontMapping) { self.mapping = mapping; self.dirty = true; }

    // ── Graphics mode (legacy API) ──

    pub fn width(&self) -> u16 { self.config.width as u16 }

    pub fn height(&self) -> u16 { self.config.height as u16 }

    pub fn clear(&mut self) {
        match self.mode {
            DisplayMode::Graphics => self.pixels.fill(0),
            DisplayMode::Text(..) => {
                self.chars.fill(0x20);
                self.char_fg.fill(1);
                self.char_bg.fill(0);
            }
        }
        self.dirty = true;
    }

    /// Set a pixel in graphics mode.
    pub fn set_pixel(&mut self, x: u8, y: u8, value: u8) {
        if let DisplayMode::Graphics = self.mode {
            if (x as u16) < self.config.width as u16 && (y as u16) < self.config.height as u16 {
                let idx = (y as usize) * (self.config.width as usize) + (x as usize);
                self.pixels[idx] = value;
                self.dirty = true;
            }
        }
    }

    /// Get a pixel in graphics mode.
    pub fn get_pixel(&self, x: u8, y: u8) -> u8 {
        if let DisplayMode::Graphics = self.mode {
            if (x as u16) < self.config.width as u16 && (y as u16) < self.config.height as u16 {
                let idx = (y as usize) * (self.config.width as usize) + (x as usize);
                return self.pixels[idx];
            }
        }
        0
    }

    /// Access the raw pixel buffer (monochrome / palette-index). Graphics mode only.
    pub fn pixels_mut(&mut self) -> Option<&mut [u8]> {
        match self.mode {
            DisplayMode::Graphics => {
                self.dirty = true;
                Some(&mut self.pixels)
            }
            _ => None,
        }
    }

    pub fn pixels(&self) -> Option<&[u8]> {
        match self.mode {
            DisplayMode::Graphics => Some(&self.pixels),
            _ => None,
        }
    }

    // ── Text mode API ──

    pub fn cols(&self) -> u16 {
        match self.mode { DisplayMode::Text(c, _) => c, _ => 0 }
    }

    pub fn rows(&self) -> u16 {
        match self.mode { DisplayMode::Text(_, r) => r, _ => 0 }
    }

    /// Set character at (col, row). 0-indexed.
    pub fn set_char(&mut self, col: u16, row: u16, ch: u8) {
        if let DisplayMode::Text(cols, rows) = self.mode {
            if col < cols && row < rows {
                let idx = (row as usize) * (cols as usize) + (col as usize);
                self.chars[idx] = ch;
                self.dirty = true;
            }
        }
    }

    pub fn get_char(&self, col: u16, row: u16) -> u8 {
        if let DisplayMode::Text(cols, rows) = self.mode {
            if col < cols && row < rows {
                return self.chars[(row as usize) * (cols as usize) + (col as usize)];
            }
        }
        0
    }

    /// Set character foreground colour index.
    pub fn set_fg(&mut self, col: u16, row: u16, color: u8) {
        if let DisplayMode::Text(cols, rows) = self.mode {
            if col < cols && row < rows {
                let idx = (row as usize) * (cols as usize) + (col as usize);
                self.char_fg[idx] = color;
                self.dirty = true;
            }
        }
    }

    /// Set character background colour index.
    pub fn set_bg(&mut self, col: u16, row: u16, color: u8) {
        if let DisplayMode::Text(cols, rows) = self.mode {
            if col < cols && row < rows {
                let idx = (row as usize) * (cols as usize) + (col as usize);
                self.char_bg[idx] = color;
                self.dirty = true;
            }
        }
    }

    pub fn char_fg(&self, col: u16, row: u16) -> u8 {
        if let DisplayMode::Text(cols, rows) = self.mode {
            if col < cols && row < rows {
                return self.char_fg[(row as usize) * (cols as usize) + (col as usize)];
            }
        }
        0
    }

    pub fn char_bg(&self, col: u16, row: u16) -> u8 {
        if let DisplayMode::Text(cols, rows) = self.mode {
            if col < cols && row < rows {
                return self.char_bg[(row as usize) * (cols as usize) + (col as usize)];
            }
        }
        0
    }

    /// Write a character at the current cursor position, advancing cursor.
    /// Handles newline, wrapping, and scrolling.
    pub fn put_char(&mut self, ch: u8) {
        if let DisplayMode::Text(cols, rows) = self.mode {
            match ch {
                b'\r' | b'\n' => {
                    self.cursor_col = 0;
                    self.cursor_row += 1;
                    if self.cursor_row >= rows {
                        self.scroll_up(1);
                        self.cursor_row = rows - 1;
                    }
                }
                0x08 | 0x7F => { // backspace
                    if self.cursor_col > 0 {
                        self.cursor_col -= 1;
                    }
                    self.set_char(self.cursor_col, self.cursor_row, 0x20);
                }
                _ => {
                    self.set_char(self.cursor_col, self.cursor_row, ch);
                    self.cursor_col += 1;
                    if self.cursor_col >= cols {
                        self.cursor_col = 0;
                        self.cursor_row += 1;
                        if self.cursor_row >= rows {
                            self.scroll_up(1);
                            self.cursor_row = rows - 1;
                        }
                    }
                }
            }
        }
    }

    /// Scroll text display up by `lines` rows.
    pub fn scroll_up(&mut self, lines: u16) {
        if let DisplayMode::Text(cols, rows) = self.mode {
            let c = cols as usize;
            let r = rows as usize;
            let shift = (lines as usize).min(r);
            self.chars.copy_within(shift * c.., 0);
            self.char_fg.copy_within(shift * c.., 0);
            self.char_bg.copy_within(shift * c.., 0);
            // Clear bottom rows
            let clear_start = (r - shift) * c;
            self.chars[clear_start..].fill(0x20);
            self.dirty = true;
        }
    }

    /// Access the character buffer directly.
    pub fn chars_mut(&mut self) -> Option<&mut [u8]> {
        match self.mode {
            DisplayMode::Text(..) => {
                self.dirty = true;
                Some(&mut self.chars)
            }
            _ => None,
        }
    }

    // ── Rendering ──

    /// Render current state into the RGBA output buffer.
    /// Returns a reference to the RGBA pixel data.
    pub fn render(&mut self) -> &[u8] {
        if !self.dirty { return &self.rendered; }

        let w = self.config.width as usize;
        let h = self.config.height as usize;
        let _aspect = self.config.pixel_aspect;

        self.rendered.resize(w * h * 4, 0);
        self.rendered.fill(0);

        match self.mode {
            DisplayMode::Graphics => {
                for y in 0..h {
                    for x in 0..w {
                        let src = y * w + x;
                        let val = if src < self.pixels.len() { self.pixels[src] } else { 0 };
                        let (r, g, b, a) = self.resolve_color(val);
                        let dst = y * w + x;
                        let offset = dst * 4;
                        self.rendered[offset] = r;
                        self.rendered[offset + 1] = g;
                        self.rendered[offset + 2] = b;
                        self.rendered[offset + 3] = a;
                    }
                }
            }
            DisplayMode::Text(cols, rows) => {
                let font = match &self.font {
                    Some(f) => f,
                    None => return &self.rendered,
                };
                let char_dx = font.char_width as usize;
                let char_dy = font.char_height as usize;

                for row in 0..(rows as usize) {
                    for col in 0..(cols as usize) {
                        let cell_idx = row * (cols as usize) + col;
                        let ch = if cell_idx < self.chars.len() { self.chars[cell_idx] } else { 0x20 };
                        let fg = if cell_idx < self.char_fg.len() { self.char_fg[cell_idx] } else { 1 };
                        let bg = if cell_idx < self.char_bg.len() { self.char_bg[cell_idx] } else { 0 };

                        let mapped = self.mapping.map(ch);

                        for cy in 0..char_dy {
                            let row_bits = font.row_bits(mapped, cy as u8) as u16;
                            for cx in 0..char_dx {
                                let pixel_set = (row_bits >> (7 - cx)) & 1 != 0;
                                let color_idx = if pixel_set { fg } else { bg };
                                let (r, g, b, a) = self.resolve_color(color_idx);

                                let px = col * char_dx + cx;
                                let py = row * char_dy + cy;
                                if px < w && py < h {
                                    let offset = (py * w + px) * 4;
                                    self.rendered[offset] = r;
                                    self.rendered[offset + 1] = g;
                                    self.rendered[offset + 2] = b;
                                    self.rendered[offset + 3] = a;
                                }
                            }
                        }
                    }
                }
            }
        }

        self.dirty = false;
        &self.rendered
    }

    /// Get the rendered RGBA buffer.
    pub fn rendered(&self) -> &[u8] { &self.rendered }

    fn resolve_color(&self, idx: u8) -> (u8, u8, u8, u8) {
        let pal = &self.config.palette;
        let i = idx as usize;
        if i < pal.len() {
            (pal[i][0], pal[i][1], pal[i][2], pal[i][3])
        } else {
            (0, 0, 0, 255)
        }
    }

    // ── Legacy buffer access ──

    pub fn get_buffer(&self) -> Vec<u8> {
        match self.mode {
            DisplayMode::Graphics => self.pixels.clone(),
            DisplayMode::Text(..) => self.chars.clone(),
        }
    }

    pub fn buffer_ptr(&self) -> *const u8 {
        match self.mode {
            DisplayMode::Graphics => self.pixels.as_ptr(),
            DisplayMode::Text(..) => self.chars.as_ptr(),
        }
    }

    pub fn buffer_len(&self) -> usize {
        match self.mode {
            DisplayMode::Graphics => self.pixels.len(),
            DisplayMode::Text(..) => self.chars.len(),
        }
    }
}

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
