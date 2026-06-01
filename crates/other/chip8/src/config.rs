use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineConfig {
    pub machine: String,
    pub description: String,
    pub cpu: CpuConfig,
    pub display: DisplayConfig,
    pub memory: MemoryConfig,
    pub keyboard: KeyboardConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuConfig {
    pub r#type: String,
    pub quirks: QuirksConfig,
    pub timer_frequency: u32,
    pub instructions_per_second: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuirksConfig {
    pub shift_vy: bool,
    pub memory_inc_i: bool,
    pub vf_reset: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub width: u16,
    pub height: u16,
    pub pixel_style: String,
    pub pixel_width: u16,
    pub pixel_height: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub size: u32,
    pub rom_offset: u16,
    pub font_offset: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardConfig {
    pub wait_for_key_blocks: bool,
    pub layout: String,
}

impl Default for MachineConfig {
    fn default() -> Self {
        MachineConfig {
            machine: "CHIP-8 (standard)".into(),
            description: "Standardowy CHIP-8 — 64×32, 35 opcodów, 4KB RAM".into(),
            cpu: CpuConfig {
                r#type: "chip8".into(),
                quirks: QuirksConfig {
                    shift_vy: false,
                    memory_inc_i: false,
                    vf_reset: true,
                },
                timer_frequency: 60,
                instructions_per_second: 30_000,
            },
            display: DisplayConfig {
                width: 64,
                height: 32,
                pixel_style: "mono".into(),
                pixel_width: 1,
                pixel_height: 1,
            },
            memory: MemoryConfig {
                size: 4096,
                rom_offset: 0x200,
                font_offset: 0x050,
            },
            keyboard: KeyboardConfig {
                wait_for_key_blocks: true,
                layout: "qwerty".into(),
            },
        }
    }
}

impl MachineConfig {
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    pub fn schip() -> Self {
        MachineConfig {
            machine: "SUPER-CHIP 1.1".into(),
            description: "SUPER-CHIP 1.1 — 128×64, dodatkowe opcody, duży font".into(),
            cpu: CpuConfig {
                r#type: "schip".into(),
                quirks: QuirksConfig {
                    shift_vy: false,
                    memory_inc_i: false,
                    vf_reset: true,
                },
                timer_frequency: 60,
                instructions_per_second: 30_000,
            },
            display: DisplayConfig {
                width: 128,
                height: 64,
                pixel_style: "mono".into(),
                pixel_width: 1,
                pixel_height: 1,
            },
            memory: MemoryConfig {
                size: 4096,
                rom_offset: 0x200,
                font_offset: 0x050,
            },
            keyboard: KeyboardConfig {
                wait_for_key_blocks: true,
                layout: "qwerty".into(),
            },
        }
    }

    pub fn xochip() -> Self {
        MachineConfig {
            machine: "XO-CHIP".into(),
            description: "XO-CHIP — 128×64, 16 kolorów, dodatkowe opcody dźwięku".into(),
            cpu: CpuConfig {
                r#type: "xochip".into(),
                quirks: QuirksConfig {
                    shift_vy: true,
                    memory_inc_i: true,
                    vf_reset: false,
                },
                timer_frequency: 60,
                instructions_per_second: 30_000,
            },
            display: DisplayConfig {
                width: 128,
                height: 64,
                pixel_style: "color".into(),
                pixel_width: 1,
                pixel_height: 1,
            },
            memory: MemoryConfig {
                size: 4096,
                rom_offset: 0x200,
                font_offset: 0x050,
            },
            keyboard: KeyboardConfig {
                wait_for_key_blocks: true,
                layout: "qwerty".into(),
            },
        }
    }

    pub fn quirks(&self) -> (bool, bool, bool) {
        (
            self.cpu.quirks.shift_vy,
            self.cpu.quirks.memory_inc_i,
            self.cpu.quirks.vf_reset,
        )
    }

    pub fn display_size(&self) -> (u16, u16) {
        (self.display.width, self.display.height)
    }

    pub fn pixel_scale(&self) -> (u16, u16) {
        (self.display.pixel_width, self.display.pixel_height)
    }

    pub fn display_area(&self) -> usize {
        (self.display.width as usize) * (self.display.height as usize)
    }

    pub fn instructions_per_second(&self) -> u32 {
        self.cpu.instructions_per_second
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_json_roundtrip() {
        let c = MachineConfig::default();
        let json = c.to_json();
        let c2 = MachineConfig::from_json(&json).unwrap();
        assert_eq!(c.machine, c2.machine);
        assert_eq!(c.cpu.quirks.shift_vy, c2.cpu.quirks.shift_vy);
    }

    #[test]
    fn test_schip_config() {
        let c = MachineConfig::schip();
        assert_eq!(c.display.width, 128);
        assert_eq!(c.cpu.r#type, "schip");
    }

    #[test]
    fn test_xochip_config() {
        let c = MachineConfig::xochip();
        assert_eq!(c.cpu.quirks.shift_vy, true);
        assert_eq!(c.cpu.quirks.memory_inc_i, true);
        assert_eq!(c.cpu.quirks.vf_reset, false);
    }

    #[test]
    fn test_display_area() {
        let c = MachineConfig::default();
        assert_eq!(c.display_area(), 2048);
        let c2 = MachineConfig::schip();
        assert_eq!(c2.display_area(), 8192);
    }

    #[test]
    fn test_quirks_tuple() {
        let c = MachineConfig::default();
        let (a, b, c_val) = c.quirks();
        assert_eq!(a, false);
        assert_eq!(b, false);
        assert_eq!(c_val, true);
    }

    #[test]
    fn test_invalid_json() {
        let r = MachineConfig::from_json("not json");
        assert!(r.is_err());
    }
}
