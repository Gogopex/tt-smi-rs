use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

pub mod history;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryData {
    pub device_info: DeviceInfo,
    pub telemetry: Telemetry,
    pub firmware_info: FirmwareInfo,
    pub limits: Limits,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub index: usize,
    pub bus_id: String,
    pub board_type: BoardType,
    pub board_id: String,
    pub coords: Coordinates,
    pub dram_status: DramStatus,
    pub dram_speed: u32,
    pub pcie_link_speed: PcieSpeed,
    pub pcie_link_width: PcieWidth,
    pub pcie_max_speed: PcieSpeed,
    pub pcie_max_width: PcieWidth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Telemetry {
    pub voltage: f32,
    pub current: f32,
    pub aiclk: u32,
    pub power: f32,
    pub temperature: f32,
    pub heartbeat: u32,
    pub arc_health: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareInfo {
    pub fw_bundle_version: String,
    pub tt_flash_version: String,
    pub cm_fw_version: String,
    pub cm_fw_date: String,
    pub eth_fw_version: String,
    pub bm_bl_version: String,
    pub bm_app_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limits {
    pub vdd_min: f32,
    pub vdd_max: f32,
    pub tdp_limit: f32,
    pub tdc_limit: f32,
    pub asic_fmax: u32,
    pub thm_limit: f32,
    pub therm_trip_l1_limit: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoardType {
    Grayskull,
    Wormhole,
    Blackhole,
    GalaxyE75,
    GalaxyE150,
    GalaxyN150,
    GalaxyN300Local,
    GalaxyN300Remote,
}

impl fmt::Display for BoardType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BoardType::Grayskull => "Grayskull",
            BoardType::Wormhole => "Wormhole",
            BoardType::Blackhole => "Blackhole",
            BoardType::GalaxyE75 => "e75",
            BoardType::GalaxyE150 => "e150",
            BoardType::GalaxyN150 => "n150",
            BoardType::GalaxyN300Local => "n300 L",
            BoardType::GalaxyN300Remote => "n300 R",
        };
        write!(f, "{s}")
    }
}

impl BoardType {
    pub fn is_blackhole(&self) -> bool {
        matches!(self, BoardType::Blackhole)
    }

    pub fn is_local(&self) -> bool {
        matches!(self, BoardType::GalaxyN300Local)
    }

    pub fn chip_arch(&self) -> ChipArch {
        match self {
            BoardType::Grayskull | BoardType::GalaxyE75 | BoardType::GalaxyE150 => {
                ChipArch::Grayskull
            }
            BoardType::Wormhole
            | BoardType::GalaxyN150
            | BoardType::GalaxyN300Local
            | BoardType::GalaxyN300Remote => ChipArch::Wormhole,
            BoardType::Blackhole => ChipArch::Blackhole,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipArch {
    Grayskull,
    Wormhole,
    Blackhole,
}

impl fmt::Display for ChipArch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ChipArch::Grayskull => "Grayskull",
            ChipArch::Wormhole => "Wormhole",
            ChipArch::Blackhole => "Blackhole",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    pub x: u8,
    pub y: u8,
    pub rack: Option<u8>,
    pub shelf: Option<u8>,
}

impl fmt::Display for Coordinates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let (Some(rack), Some(shelf)) = (self.rack, self.shelf) {
            write!(f, "({},{},{},{})", rack, shelf, self.x, self.y)
        } else {
            write!(f, "({},{})", self.x, self.y)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DramStatus {
    Trained,
    NotTrained,
    Unknown,
}

impl fmt::Display for DramStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DramStatus::Trained => "Trained",
            DramStatus::NotTrained => "Not Trained",
            DramStatus::Unknown => "Unknown",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PcieSpeed {
    Gen1,
    Gen2,
    Gen3,
    Gen4,
    Gen5,
    Unknown,
    NA,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PcieWidth {
    Width(u8),
    #[serde(rename = "N/A")]
    NA,
}

impl fmt::Display for PcieSpeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PcieSpeed::Gen1 => "2.5 GT/s",
            PcieSpeed::Gen2 => "5.0 GT/s",
            PcieSpeed::Gen3 => "8.0 GT/s",
            PcieSpeed::Gen4 => "16.0 GT/s",
            PcieSpeed::Gen5 => "32.0 GT/s",
            PcieSpeed::Unknown => "Unknown",
            PcieSpeed::NA => "N/A",
        };
        write!(f, "{s}")
    }
}

impl fmt::Display for PcieWidth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PcieWidth::Width(w) => write!(f, "{w}"),
            PcieWidth::NA => write!(f, "N/A"),
        }
    }
}

impl Telemetry {
    pub fn calculate_heartbeat(&self, chip_arch: ChipArch) -> u32 {
        match chip_arch {
            ChipArch::Grayskull => self.arc_health / 1000,
            ChipArch::Wormhole => self.arc_health / 5,
            ChipArch::Blackhole => self.arc_health / 6,
        }
    }

    fn threshold_status(
        value: f32,
        critical_threshold: f32,
        warning_threshold: f32,
    ) -> ValueStatus {
        if value > critical_threshold {
            ValueStatus::Critical
        } else if value > warning_threshold {
            ValueStatus::Warning
        } else {
            ValueStatus::Normal
        }
    }

    pub fn voltage_status(&self, limits: &Limits) -> ValueStatus {
        if self.voltage < limits.vdd_min {
            ValueStatus::Critical
        } else if self.voltage > limits.vdd_max {
            ValueStatus::Warning
        } else {
            ValueStatus::Normal
        }
    }

    pub fn current_status(&self, limits: &Limits) -> ValueStatus {
        Self::threshold_status(self.current, limits.tdc_limit, limits.tdc_limit * 0.9)
    }

    pub fn power_status(&self, limits: &Limits) -> ValueStatus {
        Self::threshold_status(self.power, limits.tdp_limit, limits.tdp_limit * 0.9)
    }

    pub fn temperature_status(&self, limits: &Limits) -> ValueStatus {
        Self::threshold_status(
            self.temperature,
            limits.therm_trip_l1_limit,
            limits.thm_limit,
        )
    }

    pub fn aiclk_status(&self, limits: &Limits) -> ValueStatus {
        if self.aiclk > limits.asic_fmax {
            ValueStatus::Warning
        } else {
            ValueStatus::Normal
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueStatus {
    Normal,
    Warning,
    Critical,
}

impl DeviceInfo {
    pub fn pcie_status(&self) -> ValueStatus {
        if self.pcie_link_speed < self.pcie_max_speed || self.pcie_link_width < self.pcie_max_width
        {
            ValueStatus::Warning
        } else {
            ValueStatus::Normal
        }
    }
}
