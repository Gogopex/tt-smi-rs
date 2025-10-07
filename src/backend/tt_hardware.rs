use anyhow::Result;
use luwen_if::chip::{Chip, ChipImpl};
use luwen_ref::detect_chips;

use super::HardwareInterface;
use crate::telemetry::*;

trait ChipArchOps {
    const DRAM_CHANNELS: usize;
    const DRAM_TRAINED_STATUS: u32;
    const HEARTBEAT_MODULO: u32;

    fn calculate_temperature(telem: &luwen_if::chip::Telemetry) -> f32;
    fn extract_dram_speed(telem: &luwen_if::chip::Telemetry) -> String;
    fn check_dram_trained(ddr_status: u32) -> bool;
}

struct GrayskullOps;

impl ChipArchOps for GrayskullOps {
    const DRAM_CHANNELS: usize = 6;
    const DRAM_TRAINED_STATUS: u32 = 1;
    const HEARTBEAT_MODULO: u32 = 1000;

    fn calculate_temperature(telem: &luwen_if::chip::Telemetry) -> f32 {
        ((telem.asic_temperature & 0xFFFF) as f32) / 16.0
    }

    fn extract_dram_speed(telem: &luwen_if::chip::Telemetry) -> String {
        format!("{}G", telem.ddr_speed.unwrap_or(0))
    }

    fn check_dram_trained(ddr_status: u32) -> bool {
        let first_channel_status = ddr_status & 0xF;
        first_channel_status == Self::DRAM_TRAINED_STATUS
    }
}

struct WormholeOps;

impl ChipArchOps for WormholeOps {
    const DRAM_CHANNELS: usize = 8;
    const DRAM_TRAINED_STATUS: u32 = 2;
    const HEARTBEAT_MODULO: u32 = 5;

    fn calculate_temperature(telem: &luwen_if::chip::Telemetry) -> f32 {
        ((telem.asic_temperature & 0xFFFF) as f32) / 16.0
    }

    fn extract_dram_speed(telem: &luwen_if::chip::Telemetry) -> String {
        let speed_bits = (telem.ddr_status >> 24) & 0xFF;
        speed_bits_to_ghz_string(speed_bits)
    }

    fn check_dram_trained(ddr_status: u32) -> bool {
        let first_channel_status = ddr_status & 0xF;
        first_channel_status == Self::DRAM_TRAINED_STATUS
    }
}

struct BlackholeOps;

impl ChipArchOps for BlackholeOps {
    const DRAM_CHANNELS: usize = 8;
    const DRAM_TRAINED_STATUS: u32 = 2;
    const HEARTBEAT_MODULO: u32 = 6;

    fn calculate_temperature(telem: &luwen_if::chip::Telemetry) -> f32 {
        let temp_raw = telem.asic_temperature as i32;
        let integer_part = (temp_raw >> 16) as f32;
        let fractional_part = (temp_raw & 0xFFFF) as f32 / 65536.0;
        integer_part + fractional_part
    }

    fn extract_dram_speed(telem: &luwen_if::chip::Telemetry) -> String {
        let speed_bits = (telem.ddr_status >> 24) & 0xFF;
        speed_bits_to_ghz_string(speed_bits)
    }

    fn check_dram_trained(ddr_status: u32) -> bool {
        let first_channel_status = ddr_status & 0xF;
        first_channel_status == Self::DRAM_TRAINED_STATUS
    }
}

pub struct TTHardware {
    devices: Vec<Chip>,
}

const VOLTAGE_SCALE: f32 = 1000.0;
const LOWER_16_BIT_MASK: u32 = 0xFFFF;

fn speed_bits_to_ghz_string(speed_bits: u32) -> String {
    match speed_bits {
        0 => "16G".to_string(),
        1 => "14G".to_string(),
        2 => "12G".to_string(),
        3 => "10G".to_string(),
        4 => "8G".to_string(),
        _ => "N/A".to_string(),
    }
}

impl TTHardware {
    pub fn new() -> Result<Self> {
        log::info!("Detecting Tenstorrent devices...");

        let devices =
            detect_chips().map_err(|e| anyhow::anyhow!("Failed to detect devices: {:?}", e))?;

        if devices.is_empty() {
            log::warn!("No Tenstorrent devices found");
        } else {
            log::info!("Found {} Tenstorrent device(s)", devices.len());
        }

        Ok(Self { devices })
    }

    fn get_chip_telemetry(chip: &Chip) -> Result<luwen_if::chip::Telemetry> {
        chip.as_gs()
            .map(|gs| gs.get_telemetry())
            .or_else(|| chip.as_wh().map(|wh| wh.get_telemetry()))
            .or_else(|| chip.as_bh().map(|bh| bh.get_telemetry()))
            .ok_or_else(|| anyhow::anyhow!("Unsupported chip type"))?
            .map_err(|e| anyhow::anyhow!("Failed to get telemetry: {:?}", e))
    }

    fn calculate_arch_specific_values(
        arch: luwen_core::Arch,
        telem: &luwen_if::chip::Telemetry,
    ) -> (f32, u32) {
        let temperature = match arch {
            luwen_core::Arch::Grayskull => GrayskullOps::calculate_temperature(telem),
            luwen_core::Arch::Wormhole => WormholeOps::calculate_temperature(telem),
            luwen_core::Arch::Blackhole => BlackholeOps::calculate_temperature(telem),
        };

        let heartbeat = match arch {
            luwen_core::Arch::Grayskull => telem.arc0_health / GrayskullOps::HEARTBEAT_MODULO,
            luwen_core::Arch::Wormhole => telem.arc3_health / WormholeOps::HEARTBEAT_MODULO,
            luwen_core::Arch::Blackhole => telem.timer_heartbeat / BlackholeOps::HEARTBEAT_MODULO,
        };

        (temperature, heartbeat)
    }

    fn chip_to_device_info(&self, chip: &Chip, index: usize) -> Result<DeviceInfo> {
        let arch = chip.get_arch();

        let (bus_id, pcie_link_speed, pcie_link_width, pcie_max_speed, pcie_max_width) = chip
            .get_device_info()
            .ok()
            .flatten()
            .map(|info| {
                let bus_id = format!(
                    "{:04x}:{:02x}:{:02x}.{}",
                    info.domain, info.bus, info.slot, info.function
                );

                let (pcie_link_speed, pcie_link_width, pcie_max_speed, pcie_max_width) =
                    get_pcie_info_from_device(&info).unwrap_or((
                        PcieSpeed::Unknown,
                        PcieWidth::Width(0),
                        PcieSpeed::Unknown,
                        PcieWidth::Width(0),
                    ));

                (
                    bus_id,
                    pcie_link_speed,
                    pcie_link_width,
                    pcie_max_speed,
                    pcie_max_width,
                )
            })
            .unwrap_or_else(|| {
                let bus_id = "N/A".to_string();
                (
                    bus_id,
                    PcieSpeed::NA,
                    PcieWidth::NA,
                    PcieSpeed::NA,
                    PcieWidth::NA,
                )
            });

        let telem = Self::get_chip_telemetry(chip).ok();

        let board_type = if let Some(t) = &telem {
            let board_id = format!("{:x}", t.board_serial_number());
            let mut board_type = get_board_type_from_id(&board_id).unwrap_or_else(|| match arch {
                luwen_core::Arch::Grayskull => BoardType::Grayskull,
                luwen_core::Arch::Wormhole => BoardType::Wormhole,
                luwen_core::Arch::Blackhole => BoardType::Blackhole,
            });

            // Override with remote/local detection for n300 specifically
            if matches!(board_type, BoardType::GalaxyN300Local) {
                if let Some(wh) = chip.as_wh() {
                    if wh.is_remote {
                        board_type = BoardType::GalaxyN300Remote;
                    }
                }
            }
            board_type
        } else {
            match arch {
                luwen_core::Arch::Grayskull => BoardType::Grayskull,
                luwen_core::Arch::Wormhole => BoardType::Wormhole,
                luwen_core::Arch::Blackhole => BoardType::Blackhole,
            }
        };

        let board_id = if let Some(t) = &telem {
            format!("{:x}", t.board_serial_number())
        } else {
            format!("DEV{index:06X}")
        };

        let coords = if let Some(wh_chip) = chip.as_wh() {
            wh_chip
                .get_local_chip_coord()
                .map(|eth_addr| Coordinates {
                    x: eth_addr.shelf_x,
                    y: eth_addr.shelf_y,
                    rack: Some(eth_addr.rack_x),
                    shelf: Some(eth_addr.rack_y),
                })
                .unwrap_or_else(|_| Self::default_coordinates(index))
        } else {
            Self::default_coordinates(index)
        };

        let dram_status = match &telem {
            Some(t) => {
                let all_trained = match arch {
                    luwen_core::Arch::Grayskull => GrayskullOps::check_dram_trained(t.ddr_status),
                    luwen_core::Arch::Wormhole => WormholeOps::check_dram_trained(t.ddr_status),
                    luwen_core::Arch::Blackhole => BlackholeOps::check_dram_trained(t.ddr_status),
                };

                if all_trained {
                    DramStatus::Trained
                } else {
                    DramStatus::NotTrained
                }
            }
            None => DramStatus::Unknown,
        };

        let dram_speed = match (&telem, arch) {
            (Some(t), luwen_core::Arch::Grayskull) => GrayskullOps::extract_dram_speed(t),
            (Some(t), luwen_core::Arch::Wormhole) => WormholeOps::extract_dram_speed(t),
            (Some(t), luwen_core::Arch::Blackhole) => BlackholeOps::extract_dram_speed(t),
            (None, _) => "N/A".to_string(),
        };

        Ok(DeviceInfo {
            index,
            bus_id,
            board_type,
            board_id,
            coords,
            dram_status,
            dram_speed,
            pcie_link_speed,
            pcie_link_width,
            pcie_max_speed,
            pcie_max_width,
        })
    }

    fn default_coordinates(index: usize) -> Coordinates {
        Coordinates {
            x: 0,
            y: index as u8,
            rack: None,
            shelf: None,
        }
    }

    fn extract_current_value(packed: u32) -> u32 {
        packed & LOWER_16_BIT_MASK
    }

    fn extract_limit_value(packed: u32) -> u32 {
        (packed >> 16) & LOWER_16_BIT_MASK
    }
}

impl HardwareInterface for TTHardware {
    fn list_devices(&self) -> Result<Vec<DeviceInfo>> {
        self.devices
            .iter()
            .enumerate()
            .map(|(index, chip)| self.chip_to_device_info(chip, index))
            .collect()
    }

    fn get_telemetry(&self, device_index: usize) -> Result<Telemetry> {
        let chip = self
            .devices
            .get(device_index)
            .ok_or_else(|| anyhow::anyhow!("Invalid device index"))?;

        let telem = Self::get_chip_telemetry(chip)?;

        let aiclk_current = Self::extract_current_value(telem.aiclk);
        let tdc_current = Self::extract_current_value(telem.tdc) as f32;
        let tdp_current = Self::extract_current_value(telem.tdp) as f32;

        let (temperature, heartbeat) =
            Self::calculate_arch_specific_values(chip.get_arch(), &telem);

        Ok(Telemetry {
            voltage: telem.vcore as f32 / VOLTAGE_SCALE,
            current: tdc_current,
            aiclk: aiclk_current,
            power: tdp_current,
            temperature,
            heartbeat,
            arc_health: telem.arc0_health,
        })
    }

    fn get_firmware_info(&self, device_index: usize) -> Result<FirmwareInfo> {
        let chip = self
            .devices
            .get(device_index)
            .ok_or_else(|| anyhow::anyhow!("Invalid device index"))?;

        let telem = Self::get_chip_telemetry(chip)?;

        Ok(FirmwareInfo {
            fw_bundle_version: format_m3_fw_version(telem.fw_bundle_version),
            tt_flash_version: format_m3_fw_version(telem.tt_flash_version),
            cm_fw_version: format_m3_fw_version(telem.arc0_fw_version),
            cm_fw_date: telem.firmware_date(),
            eth_fw_version: telem.eth_fw_version(),
            bm_bl_version: format_m3_fw_version(telem.m3_bl_fw_version),
            bm_app_version: format_m3_fw_version(telem.m3_app_fw_version),
        })
    }

    fn get_limits(&self, device_index: usize) -> Result<Limits> {
        let chip = self
            .devices
            .get(device_index)
            .ok_or_else(|| anyhow::anyhow!("Invalid device index"))?;

        let telem = Self::get_chip_telemetry(chip)?;

        let vdd_min = Self::extract_current_value(telem.vdd_limits) as f32 / VOLTAGE_SCALE;
        let vdd_max = Self::extract_limit_value(telem.vdd_limits) as f32 / VOLTAGE_SCALE;
        let tdp_limit = Self::extract_limit_value(telem.tdp) as f32;
        let tdc_limit = Self::extract_limit_value(telem.tdc) as f32;
        let asic_fmax = Self::extract_limit_value(telem.aiclk);
        let thm_limit = Self::extract_current_value(telem.thm_limits) as f32;
        let therm_trip_l1_limit = Self::extract_limit_value(telem.thm_limits) as f32;

        Ok(Limits {
            vdd_min,
            vdd_max,
            tdp_limit,
            tdc_limit,
            asic_fmax,
            thm_limit,
            therm_trip_l1_limit,
        })
    }

    fn get_device_info(&self, device_index: usize) -> Result<DeviceInfo> {
        let chip = self
            .devices
            .get(device_index)
            .ok_or_else(|| anyhow::anyhow!("Invalid device index"))?;
        self.chip_to_device_info(chip, device_index)
    }

    fn reset_device(&self, device_index: usize) -> Result<()> {
        let chip = self
            .devices
            .get(device_index)
            .ok_or_else(|| anyhow::anyhow!("Invalid device index"))?;

        let interface_id = chip
            .get_device_info()
            .map_err(|_| anyhow::anyhow!("Failed to get device info"))?
            .ok_or_else(|| anyhow::anyhow!("Device info not available"))?
            .interface_id as usize;

        if chip.get_arch() == luwen_core::Arch::Grayskull {
            log::info!("Skipping reset for Grayskull device {device_index}");
            return Ok(());
        }

        let fd = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(format!("/dev/tenstorrent/{interface_id}"))
            .map_err(|e| anyhow::anyhow!("Failed to open device: {}", e))?;

        let mut reset_device = ttkmd_if::ioctl::ResetDevice {
            input: ttkmd_if::ioctl::ResetDeviceIn {
                flags: ttkmd_if::ioctl::RESET_DEVICE_RESET_PCIE_LINK,
                ..Default::default()
            },
            ..Default::default()
        };

        use std::os::fd::AsRawFd;
        unsafe {
            ttkmd_if::ioctl::reset_device(fd.as_raw_fd(), &mut reset_device)
                .map_err(|e| anyhow::anyhow!("Failed to reset device: {:?}", e))?;
        }

        if reset_device.output.result != 0 {
            return Err(anyhow::anyhow!(
                "Device reset failed with result: {}",
                reset_device.output.result
            ));
        }

        log::info!("Successfully reset device {device_index}");
        Ok(())
    }
}

fn format_m3_fw_version(version: u32) -> String {
    if version == 0xFFFFFFFF || version == 0 {
        "N/A".to_string()
    } else {
        let major = (version >> 24) & 0xFF;
        let minor = (version >> 16) & 0xFF;
        let patch = (version >> 8) & 0xFF;
        let build = version & 0xFF;
        format!("{major}.{minor}.{patch}.{build}")
    }
}

fn get_pcie_info_from_device(
    device_info: &luwen_if::DeviceInfo,
) -> Result<(PcieSpeed, PcieWidth, PcieSpeed, PcieWidth)> {
    let current_gen =
        std::panic::catch_unwind(|| device_info.pcie_current_link_gen()).unwrap_or(-1);
    let current_width =
        std::panic::catch_unwind(|| device_info.pcie_current_link_width()).unwrap_or(0) as u8;
    let max_gen = std::panic::catch_unwind(|| device_info.pcie_max_link_gen()).unwrap_or(-1);
    let max_width =
        std::panic::catch_unwind(|| device_info.pcie_max_link_width()).unwrap_or(0) as u8;

    let current_speed = match current_gen {
        1 => PcieSpeed::Gen1,
        2 => PcieSpeed::Gen2,
        3 => PcieSpeed::Gen3,
        4 => PcieSpeed::Gen4,
        5 => PcieSpeed::Gen5,
        _ => PcieSpeed::Unknown,
    };

    let max_speed = match max_gen {
        1 => PcieSpeed::Gen1,
        2 => PcieSpeed::Gen2,
        3 => PcieSpeed::Gen3,
        4 => PcieSpeed::Gen4,
        5 => PcieSpeed::Gen5,
        _ => PcieSpeed::Unknown,
    };

    Ok((
        current_speed,
        PcieWidth::Width(current_width),
        max_speed,
        PcieWidth::Width(max_width),
    ))
}

fn get_board_type_from_id(board_id: &str) -> Option<BoardType> {
    if let Ok(serial_num) = u64::from_str_radix(board_id, 16) {
        let upi = (serial_num >> 36) & 0xFFFFF;

        match upi {
            // Grayskull cards
            0x3 => Some(BoardType::GalaxyE150),
            0xA => Some(BoardType::GalaxyE150), // e300 maps to GalaxyE150 for now
            0x7 => Some(BoardType::GalaxyE75),

            // Wormhole cards
            0x8 => Some(BoardType::Wormhole),         // nb_cb
            0xB => Some(BoardType::Wormhole),         // wh_4u
            0x14 => Some(BoardType::GalaxyN300Local), // n300
            0x18 => Some(BoardType::GalaxyN150),      // n150
            0x35 => Some(BoardType::Wormhole),        // tt-galaxy-wh

            // Blackhole cards
            0x36 => Some(BoardType::Blackhole), // bh-scrappy
            0x43 => Some(BoardType::Blackhole), // p100a
            0x40 => Some(BoardType::Blackhole), // p150a
            0x41 => Some(BoardType::Blackhole), // p150b
            0x42 => Some(BoardType::Blackhole), // p150c
            0x44 => Some(BoardType::Blackhole), // p300b
            0x45 => Some(BoardType::Blackhole), // p300a
            0x46 => Some(BoardType::Blackhole), // p300c
            0x47 => Some(BoardType::Blackhole), // tt-galaxy-bh
            _ => None,
        }
    } else {
        None
    }
}
