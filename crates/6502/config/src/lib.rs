use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CpuFamily {
    #[serde(rename = "nmos6502")]
    Nmos6502,
    #[serde(rename = "nmos6510")]
    Nmos6510,
    #[serde(rename = "nmos8502")]
    Nmos8502,
    #[serde(rename = "nmos6507")]
    Nmos6507,
    #[serde(rename = "ricoh2a03")]
    Ricoh2A03,
    #[serde(rename = "r65c02")]
    R65C02,
    #[serde(rename = "w65c02")]
    W65C02,
}

impl Default for CpuFamily { fn default() -> Self { Self::Nmos6502 } }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RmwBehavior {
    #[serde(rename = "nmos")]
    Nmos,
    #[serde(rename = "cmos")]
    Cmos,
}

impl Default for RmwBehavior { fn default() -> Self { Self::Nmos } }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    #[serde(alias = "address_space", default = "default_memory_size")]
    pub size: usize,
    #[serde(default)]
    pub bank_switching: bool,
    #[serde(default = "default_num_banks")]
    pub num_banks: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self { size: 65536, bank_switching: false, num_banks: 1 }
    }
}

fn default_memory_size() -> usize { 65536 }
fn default_num_banks() -> usize { 1 }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CpuQuirks {
    pub jmp_indirect_bug: bool,
    pub bcd_available: bool,
    pub kil_halts: bool,
    pub rmw: RmwBehavior,
    pub undocumented_ops: bool,
    pub stp_available: bool,
    pub wai_available: bool,
}

impl Default for CpuQuirks {
    fn default() -> Self {
        Self {
            jmp_indirect_bug: true,
            bcd_available: true,
            kil_halts: true,
            rmw: RmwBehavior::Nmos,
            undocumented_ops: true,
            stp_available: false,
            wai_available: false,
        }
    }
}

impl CpuQuirks {
    pub fn nmos() -> Self { Self::default() }
    pub fn cmos() -> Self {
        Self {
            jmp_indirect_bug: false,
            bcd_available: true,
            kil_halts: true,
            rmw: RmwBehavior::Cmos,
            undocumented_ops: false,
            stp_available: true,
            wai_available: true,
        }
    }
    pub fn ricoh2a03() -> Self {
        Self {
            jmp_indirect_bug: true,
            bcd_available: false,
            kil_halts: true,
            rmw: RmwBehavior::Nmos,
            undocumented_ops: false,
            stp_available: false,
            wai_available: false,
        }
    }
    pub fn r65c02() -> Self {
        Self {
            jmp_indirect_bug: false,
            bcd_available: true,
            kil_halts: true,
            rmw: RmwBehavior::Cmos,
            undocumented_ops: false,
            stp_available: true,
            wai_available: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineConfig {
    pub family: CpuFamily,
    pub label: String,
    pub description: String,
    pub quirks: CpuQuirks,
    pub memory: MemoryConfig,
    pub start_address: u16,
    pub reset_vector: u16,
    pub nmi_vector: u16,
    pub irq_vector: u16,
    #[serde(default)]
    pub is_apple1: bool,
}

impl MachineConfig {
    pub fn nmos6502() -> Self {
        Self {
            family: CpuFamily::Nmos6502,
            label: "MOS 6502 (NMOS)".into(),
            description: "Oryginalny NMOS 6502, 1975. JMP indirect bug, KIL, RMW 2 writes".into(),
            quirks: CpuQuirks::nmos(),
            memory: MemoryConfig::default(),
            start_address: 0x8000,
            reset_vector: 0xFFFC,
            nmi_vector: 0xFFFA,
            irq_vector: 0xFFFE,
            is_apple1: false,
        }
    }
    pub fn ricoh2a03() -> Self {
        Self {
            family: CpuFamily::Ricoh2A03,
            label: "Ricoh 2A03 (NES)".into(),
            description: "NMOS 6502 bez BCD, używany w Nintendo NES/Famicom".into(),
            quirks: CpuQuirks::ricoh2a03(),
            memory: MemoryConfig::default(),
            start_address: 0x8000,
            reset_vector: 0xFFFC,
            nmi_vector: 0xFFFA,
            irq_vector: 0xFFFE,
            is_apple1: false,
        }
    }
    pub fn w65c02() -> Self {
        Self {
            family: CpuFamily::W65C02,
            label: "WDC W65C02 (CMOS)".into(),
            description: "CMOS 6502: JMP fix, STP, WAI, BCD poprawny, RMW 2 reads".into(),
            quirks: CpuQuirks::cmos(),
            memory: MemoryConfig::default(),
            start_address: 0x8000,
            reset_vector: 0xFFFC,
            nmi_vector: 0xFFFA,
            irq_vector: 0xFFFE,
            is_apple1: false,
        }
    }
    pub fn r65c02() -> Self {
        Self {
            family: CpuFamily::R65C02,
            label: "Rockwell R65C02".into(),
            description: "CMOS 6502 z RMB/SMB/BBS/BBR, JMP fix".into(),
            quirks: CpuQuirks::r65c02(),
            memory: MemoryConfig::default(),
            start_address: 0x8000,
            reset_vector: 0xFFFC,
            nmi_vector: 0xFFFA,
            irq_vector: 0xFFFE,
            is_apple1: false,
        }
    }
    pub fn mos6510() -> Self {
        Self {
            family: CpuFamily::Nmos6510,
            label: "MOS 6510 (C64)".into(),
            description: "NMOS 6502 z I/O portem, używany w Commodore 64".into(),
            quirks: CpuQuirks::nmos(),
            memory: MemoryConfig::default(),
            start_address: 0x8000,
            reset_vector: 0xFFFC,
            nmi_vector: 0xFFFA,
            irq_vector: 0xFFFE,
            is_apple1: false,
        }
    }
    pub fn mos6507() -> Self {
        Self {
            family: CpuFamily::Nmos6507,
            label: "MOS 6507 (Atari 2600)".into(),
            description: "NMOS 6502 z 13-bitową szyną adresową (8KB przestrzeni)".into(),
            quirks: CpuQuirks::nmos(),
            memory: MemoryConfig { size: 8192, ..MemoryConfig::default() },
            start_address: 0xF000,
            reset_vector: 0xFFFC,
            nmi_vector: 0xFFFA,
            irq_vector: 0xFFFE,
            is_apple1: false,
        }
    }

    pub fn nmos() -> Self { Self::nmos6502() }
    pub fn cmos() -> Self { Self::w65c02() }
    pub fn nes() -> Self { Self::ricoh2a03() }
    pub fn c64() -> Self { Self::mos6510() }
    pub fn atari2600() -> Self { Self::mos6507() }
    pub fn apple2() -> Self { Self::nmos6502() }
    pub fn apple1() -> Self {
        Self {
            label: "Apple 1".into(),
            description: "Apple 1 — 6502 + PIA 6821 + terminal".into(),
            is_apple1: true,
            ..Self::nmos6502()
        }
    }

    pub fn has_jmp_indirect_bug(&self) -> bool { self.quirks.jmp_indirect_bug }
    pub fn supports_bcd(&self) -> bool { self.quirks.bcd_available }
    pub fn rmw_behavior(&self) -> RmwBehavior { self.quirks.rmw }
    pub fn has_undocumented_ops(&self) -> bool { self.quirks.undocumented_ops }
    pub fn has_stp(&self) -> bool { self.quirks.stp_available }
    pub fn has_wai(&self) -> bool { self.quirks.wai_available }
    pub fn kil_halts(&self) -> bool { self.quirks.kil_halts }

    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}

impl Default for MachineConfig {
    fn default() -> Self { Self::nmos6502() }
}

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
