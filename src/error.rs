use thiserror::Error;

#[derive(Error, Debug)]
pub enum TtSmiError {
    #[error("Invalid query field: '{field}'. Use --help-query to see available fields.")]
    InvalidQueryField { field: String },
    
    #[error("Device not found: {device}")]
    DeviceNotFound { device: String },
    
    #[error("Hardware error: {0}")]
    Hardware(#[from] anyhow::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("No devices found")]
    NoDevicesFound,
    
    #[error("Unsupported chip type")]
    UnsupportedChipType,
    
    #[error("Failed to get telemetry: {reason}")]
    TelemetryError { reason: String },
    
    #[error("PCIe information not available for device {device}")]
    PcieInfoUnavailable { device: String },
    
    #[error("Reset operation failed: {reason}")]
    ResetFailed { reason: String },
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Parse error: {0}")]
    Parse(String),
}

pub type Result<T> = std::result::Result<T, TtSmiError>;