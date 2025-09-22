use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::telemetry::*;

mod tt_hardware;

pub use tt_hardware::TTHardware;

pub trait HardwareInterface: Send + Sync {
    fn list_devices(&self) -> Result<Vec<DeviceInfo>>;
    fn get_telemetry(&self, device_index: usize) -> Result<Telemetry>;
    fn get_firmware_info(&self, device_index: usize) -> Result<FirmwareInfo>;
    fn get_limits(&self, device_index: usize) -> Result<Limits>;
    fn reset_device(&self, device_index: usize) -> Result<()>;
    fn get_device_info(&self, device_index: usize) -> Result<DeviceInfo>;
}

pub struct Backend {
    hw_interface: Arc<Mutex<Box<dyn HardwareInterface>>>,
    devices: Vec<DeviceInfo>,
}

impl Backend {
    pub async fn new() -> Result<Self> {
        log::info!("Initializing hardware backend");

        let hw_interface: Box<dyn HardwareInterface> = Box::new(TTHardware::new()?);
        let devices = hw_interface.list_devices()?;

        if devices.is_empty() {
            anyhow::bail!(
                "No Tenstorrent devices detected! Please check your hardware and try again."
            );
        }

        log::info!("Found {} Tenstorrent device(s)", devices.len());

        Ok(Self {
            hw_interface: Arc::new(Mutex::new(hw_interface)),
            devices,
        })
    }

    async fn collect_telemetry_data(&self) -> Result<Vec<TelemetryData>> {
        let hw = self.hw_interface.lock().await;
        let timestamp = Utc::now();

        self.devices
            .iter()
            .map(|device| {
                Ok(TelemetryData {
                    device_info: hw.get_device_info(device.index)?,
                    telemetry: hw.get_telemetry(device.index)?,
                    firmware_info: hw.get_firmware_info(device.index)?,
                    limits: hw.get_limits(device.index)?,
                    timestamp,
                })
            })
            .collect()
    }

    pub async fn get_initial_data(&self) -> Result<Vec<TelemetryData>> {
        self.collect_telemetry_data().await
    }

    pub async fn get_telemetry_update(&self) -> Result<Vec<TelemetryData>> {
        self.collect_telemetry_data().await
    }

    pub async fn reset_device(&self, device_index: usize) -> Result<()> {
        let hw = self.hw_interface.lock().await;
        hw.reset_device(device_index)
    }
}
