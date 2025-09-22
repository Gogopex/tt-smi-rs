use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use log::info;
use std::str::FromStr;

mod app;
mod backend;
mod config;
mod telemetry;
mod ui;
mod utils;

use app::App;

#[derive(ValueEnum, Clone, Debug)]
enum OutputFormat {
    Csv,
    Json,
    Table,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum QueryField {
    Index,
    BusId,
    BoardType,
    BoardId,
    Coords,
    DramStatus,
    DramSpeed,
    PcieSpeed,
    PcieWidth,
    PcieMaxSpeed,
    PcieMaxWidth,
    Voltage,
    Current,
    Power,
    Temperature,
    Aiclk,
    Heartbeat,
    FwBundleVersion,
    TtFlashVersion,
    CmFwVersion,
    CmFwDate,
    EthFwVersion,
    BmBlVersion,
    BmAppVersion,
}

impl FromStr for QueryField {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use QueryField::*;
        Ok(match s {
            "index" => Index,
            "bus_id" => BusId,
            "board_type" => BoardType,
            "board_id" => BoardId,
            "coords" => Coords,
            "dram_status" => DramStatus,
            "dram_speed" => DramSpeed,
            "pcie_speed" => PcieSpeed,
            "pcie_width" => PcieWidth,
            "pcie_max_speed" => PcieMaxSpeed,
            "pcie_max_width" => PcieMaxWidth,
            "voltage" => Voltage,
            "current" => Current,
            "power" => Power,
            "temperature" => Temperature,
            "aiclk" => Aiclk,
            "heartbeat" => Heartbeat,
            "fw_bundle_version" => FwBundleVersion,
            "tt_flash_version" => TtFlashVersion,
            "cm_fw_version" => CmFwVersion,
            "cm_fw_date" => CmFwDate,
            "eth_fw_version" => EthFwVersion,
            "bm_bl_version" => BmBlVersion,
            "bm_app_version" => BmAppVersion,
            _ => {
                return Err(format!(
                    "Invalid query field: '{s}'. Use --help-query to see available fields."
                ));
            }
        })
    }
}

impl QueryField {
    fn as_str(&self) -> &'static str {
        match self {
            QueryField::Index => "index",
            QueryField::BusId => "bus_id",
            QueryField::BoardType => "board_type",
            QueryField::BoardId => "board_id",
            QueryField::Coords => "coords",
            QueryField::DramStatus => "dram_status",
            QueryField::DramSpeed => "dram_speed",
            QueryField::PcieSpeed => "pcie_speed",
            QueryField::PcieWidth => "pcie_width",
            QueryField::PcieMaxSpeed => "pcie_max_speed",
            QueryField::PcieMaxWidth => "pcie_max_width",
            QueryField::Voltage => "voltage",
            QueryField::Current => "current",
            QueryField::Power => "power",
            QueryField::Temperature => "temperature",
            QueryField::Aiclk => "aiclk",
            QueryField::Heartbeat => "heartbeat",
            QueryField::FwBundleVersion => "fw_bundle_version",
            QueryField::TtFlashVersion => "tt_flash_version",
            QueryField::CmFwVersion => "cm_fw_version",
            QueryField::CmFwDate => "cm_fw_date",
            QueryField::EthFwVersion => "eth_fw_version",
            QueryField::BmBlVersion => "bm_bl_version",
            QueryField::BmAppVersion => "bm_app_version",
        }
    }
}

#[derive(Parser)]
#[command(
    name = "tt-smi",
    about = "Tenstorrent System Management Interface - Monitor and manage Tenstorrent devices",
    long_about = "TT-SMI is a command line utility to interact with all Tenstorrent devices on host.\n\
                  Main objective is to provide a simple and easy to use interface to collect and \n\
                  display device, telemetry and firmware information.",
    version,
    author
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long, global = true)]
    verbose: bool,

    #[arg(short, long, global = true)]
    compact: bool,

    #[arg(short, long, global = true)]
    local: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(visible_alias = "ls")]
    List,

    #[command(visible_alias = "s")]
    Snapshot {
        #[arg(short, long)]
        output: Option<String>,
    },

    #[command(visible_alias = "r")]
    Reset {
        devices: Vec<String>,

        #[arg(short, long)]
        force: bool,

        #[arg(long)]
        no_reinit: bool,
    },

    #[command(visible_alias = "g")]
    GenerateResetConfig {
        #[arg(short, long)]
        output: Option<String>,
    },

    #[command(name = "glx-reset")]
    GalaxyReset {
        #[arg(short, long)]
        force: bool,

        #[arg(long)]
        no_reinit: bool,
    },

    #[command(visible_alias = "q")]
    Query {
        fields: String,

        #[arg(long, default_value = "csv", value_enum)]
        format: OutputFormat,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let log_level = if cli.verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .format_timestamp_millis()
        .init();

    info!("Starting tt-smi v{}", env!("CARGO_PKG_VERSION"));

    match cli.command {
        None => {
            let mut app = App::new(cli.compact).await?;
            app.run().await?;
        }
        Some(Commands::List) => {
            list_devices(cli.local).await?;
        }
        Some(Commands::Snapshot { output }) => {
            snapshot_devices(output, cli.local).await?;
        }
        Some(Commands::Reset {
            devices,
            force,
            no_reinit,
        }) => {
            reset_devices(devices, force, no_reinit, cli.local).await?;
        }
        Some(Commands::GenerateResetConfig { output }) => {
            generate_reset_config(output, cli.local).await?;
        }
        Some(Commands::GalaxyReset { force, no_reinit }) => {
            galaxy_reset(force, no_reinit).await?;
        }
        Some(Commands::Query { fields, format }) => {
            query_devices(&fields, format, cli.local).await?;
        }
    }

    Ok(())
}

async fn list_devices(local_only: bool) -> Result<()> {
    let backend = backend::Backend::new().await?;
    let mut telemetry_data = backend.get_initial_data().await?;

    if local_only {
        telemetry_data.retain(|data| !data.device_info.board_type.to_string().ends_with(" R"));
    }

    if telemetry_data.is_empty() {
        println!("No Tenstorrent devices found.");
        return Ok(());
    }

    println!("All available boards on host:");
    println!("┌─────────────┬────────────┬──────────────┬──────────────┐");
    println!("│ PCI Dev ID  │ Board Type │ Device Series│ Board Number │");
    println!("├─────────────┼────────────┼──────────────┼──────────────┤");

    telemetry_data.iter().for_each(|data| {
        let dev = &data.device_info;
        let pci_id = if dev.board_type.to_string().ends_with(" R") {
            "N/A".to_string()
        } else {
            dev.index.to_string()
        };
        println!(
            "│ {:^11} │ {:^10} │ {:^12} │ {:^12} │",
            pci_id,
            dev.board_type.chip_arch(),
            dev.board_type,
            dev.board_id
        );
    });
    println!("└─────────────┴────────────┴──────────────┴──────────────┘");

    println!("\nBoards that can be reset:");
    println!("┌─────────────┬────────────┬──────────────┬──────────────┐");
    println!("│ PCI Dev ID  │ Board Type │ Device Series│ Board Number │");
    println!("├─────────────┼────────────┼──────────────┼──────────────┤");

    telemetry_data
        .iter()
        .filter(|data| !data.device_info.board_type.to_string().ends_with(" R"))
        .for_each(|data| {
            let dev = &data.device_info;
            println!(
                "│ {:^11} │ {:^10} │ {:^12} │ {:^12} │",
                dev.index,
                dev.board_type.chip_arch(),
                dev.board_type,
                dev.board_id
            );
        });
    println!("└─────────────┴────────────┴──────────────┴──────────────┘");

    Ok(())
}

async fn snapshot_devices(output: Option<String>, local_only: bool) -> Result<()> {
    let backend = backend::Backend::new().await?;
    let mut telemetry_data = backend.get_initial_data().await?;

    if local_only {
        telemetry_data.retain(|data| {
            !data.device_info.board_type.to_string().ends_with(" R")
        });
    }

    let host_info = utils::get_host_info();

    let snapshot_data = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "host": {
            "os": host_info.os,
            "kernel": host_info.kernel,
            "driver": host_info.driver,
        },
        "device_count": telemetry_data.len(),
        "devices": telemetry_data.iter().map(|data| {
            serde_json::json!({
                "device_info": {
                    "index": data.device_info.index,
                    "bus_id": data.device_info.bus_id,
                    "board_type": data.device_info.board_type.to_string(),
                    "board_id": data.device_info.board_id,
                    "coords": data.device_info.coords.to_string(),
                    "dram_status": data.device_info.dram_status.to_string(),
                    "dram_speed": data.device_info.dram_speed,
                    "pcie_link_speed": data.device_info.pcie_link_speed.to_string(),
                    "pcie_link_width": data.device_info.pcie_link_width,
                    "pcie_max_speed": data.device_info.pcie_max_speed.to_string(),
                    "pcie_max_width": data.device_info.pcie_max_width,
                },
                "telemetry": {
                    "voltage": data.telemetry.voltage,
                    "current": data.telemetry.current,
                    "aiclk": data.telemetry.aiclk,
                    "power": data.telemetry.power,
                    "temperature": data.telemetry.temperature,
                    "heartbeat": data.telemetry.heartbeat,
                    "arc_health": data.telemetry.arc_health,
                },
                "firmware": {
                    "fw_bundle_version": data.firmware_info.fw_bundle_version,
                    "tt_flash_version": data.firmware_info.tt_flash_version,
                    "cm_fw_version": data.firmware_info.cm_fw_version,
                    "cm_fw_date": data.firmware_info.cm_fw_date,
                    "eth_fw_version": data.firmware_info.eth_fw_version,
                    "bm_bl_version": data.firmware_info.bm_bl_version,
                    "bm_app_version": data.firmware_info.bm_app_version,
                },
                "limits": {
                    "vdd_min": data.limits.vdd_min,
                    "vdd_max": data.limits.vdd_max,
                    "tdp_limit": data.limits.tdp_limit,
                    "tdc_limit": data.limits.tdc_limit,
                    "asic_fmax": data.limits.asic_fmax,
                    "thm_limit": data.limits.thm_limit,
                    "therm_trip_l1_limit": data.limits.therm_trip_l1_limit,
                },
            })
        }).collect::<Vec<_>>(),
    });

    match output {
        Some(path) => {
            std::fs::write(&path, serde_json::to_string_pretty(&snapshot_data)?)?;
            println!("Snapshot saved to: {path}");
        }
        None => {
            println!("{}", serde_json::to_string_pretty(&snapshot_data)?);
        }
    }

    Ok(())
}

async fn reset_devices(
    devices: Vec<String>,
    force: bool,
    no_reinit: bool,
    local_only: bool,
) -> Result<()> {
    use std::io::{self, Write};

    if devices.len() == 1 && devices[0].ends_with(".json") {
        return reset_from_config(&devices[0], no_reinit, local_only).await;
    }

    if !force {
        println!(
            "Reset {} device(s)? This action cannot be undone.",
            devices.len()
        );
        print!("Continue? [y/N] ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Reset cancelled.");
            return Ok(());
        }
    }

    let backend = backend::Backend::new().await?;
    let device_list = backend.get_initial_data().await?;

    for device_spec in devices {
        println!("Resetting device {device_spec}...");

        let device = device_spec
            .parse::<usize>()
            .ok()
            .and_then(|idx| device_list.get(idx))
            .or_else(|| {
                device_list
                    .iter()
                    .find(|d| d.device_info.bus_id == device_spec)
            });

        match device {
            Some(device_data) => match backend.reset_device(device_data.device_info.index).await {
                Ok(_) => println!("Successfully reset device {device_spec}"),
                Err(e) => eprintln!("Failed to reset device {device_spec}: {e}"),
            },
            None => eprintln!("Device {device_spec} not found"),
        }
    }

    if !no_reinit {
        println!("Re-initializing devices after reset...");
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        println!("Reset complete.");
    }

    Ok(())
}

async fn generate_reset_config(output: Option<String>, local_only: bool) -> Result<()> {
    let backend = backend::Backend::new().await?;
    let mut devices = backend.get_initial_data().await?;

    if local_only {
        devices.retain(|data| !data.device_info.board_type.to_string().ends_with(" R"));
    }

    let config = serde_json::json!({
        "re_init_devices": true,
        "wh_link_reset": {
            "pci_index": devices.iter()
                .filter(|d| d.device_info.board_type.chip_arch() == crate::telemetry::ChipArch::Wormhole)
                .map(|d| d.device_info.index)
                .collect::<Vec<_>>()
        },
        "gs_tensix_reset": {
            "pci_index": devices.iter()
                .filter(|d| d.device_info.board_type.chip_arch() == crate::telemetry::ChipArch::Grayskull)
                .map(|d| d.device_info.index)
                .collect::<Vec<_>>()
        },
        "bh_link_reset": {
            "pci_index": devices.iter()
                .filter(|d| d.device_info.board_type.chip_arch() == crate::telemetry::ChipArch::Blackhole)
                .map(|d| d.device_info.index)
                .collect::<Vec<_>>()
        }
    });

    let config_str = serde_json::to_string_pretty(&config)?;

    match output {
        Some(path) => {
            std::fs::write(&path, config_str)?;
            println!("Generated sample reset config file: {path}");
        }
        None => {
            let default_path = std::env::var("HOME")
                .map(|home| format!("{home}/.config/tenstorrent/reset_config.json"))
                .unwrap_or_else(|_| "reset_config.json".to_string());

            if let Some(parent) = std::path::Path::new(&default_path).parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(&default_path, config_str)?;
            println!("Generated sample reset config file: {default_path}");
        }
    }

    println!("Update the generated file and use it as input for the reset command.");
    Ok(())
}

async fn galaxy_reset(force: bool, no_reinit: bool) -> Result<()> {
    use std::io::{self, Write};

    if !force {
        print!("Reset entire Galaxy? This action cannot be undone. [y/N] ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Galaxy reset cancelled.");
            return Ok(());
        }
    }

    println!("Performing Galaxy reset...");

    let backend = backend::Backend::new().await?;
    let devices = backend.get_initial_data().await?;

    let galaxy_devices: Vec<_> = devices
        .iter()
        .filter(|d| {
            d.device_info.board_type.to_string().contains("galaxy")
                || d.device_info.board_type.to_string().contains("wh_4u")
        })
        .collect();

    if galaxy_devices.is_empty() {
        println!("No Galaxy devices found on this host.");
        return Ok(());
    }

    println!(
        "Found {} Galaxy device(s), performing reset...",
        galaxy_devices.len()
    );

    for device in galaxy_devices {
        if let Err(e) = backend.reset_device(device.device_info.index).await {
            eprintln!(
                "Failed to reset Galaxy device {}: {}",
                device.device_info.index, e
            );
        }
    }

    if !no_reinit {
        println!("Waiting 30 seconds for Galaxy systems to reinitialize...");
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        println!("Galaxy reset complete.");
    }

    Ok(())
}

async fn reset_from_config(config_file: &str, no_reinit: bool, _local_only: bool) -> Result<()> {
    println!("Resetting devices from config file: {config_file}");

    let config_content =
        std::fs::read_to_string(config_file).context("Failed to read config file")?;

    let config: serde_json::Value =
        serde_json::from_str(&config_content).context("Failed to parse config file as JSON")?;

    let backend = backend::Backend::new().await?;

    let reset_operations: Vec<(u64, &'static str)> = [
        ("gs_tensix_reset", "Grayskull tensix"),
        ("wh_link_reset", "Wormhole link"),
        ("bh_link_reset", "Blackhole link"),
    ]
    .into_iter()
    .flat_map(|(key, reset_type)| {
        config
            .get(key)
            .and_then(|obj| obj.get("pci_index"))
            .and_then(|v| v.as_array())
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|v| v.as_u64().map(|idx| (idx, reset_type)))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    })
    .collect();

    let mut reset_count = 0;
    for (idx, reset_type) in reset_operations {
        println!("Performing {reset_type} reset on device {idx}");
        match backend.reset_device(idx as usize).await {
            Ok(_) => reset_count += 1,
            Err(e) => eprintln!("Failed to reset {reset_type} device {idx}: {e}"),
        }
    }

    println!("Reset {reset_count} device(s) from config file");

    let should_reinit = if no_reinit {
        false
    } else {
        config
            .get("re_init_devices")
            .and_then(|v| v.as_bool())
            .unwrap_or(true)
    };

    if should_reinit {
        println!("Re-initializing devices after reset...");
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        println!("Reset from config complete.");
    } else {
        println!("Skipping device re-initialization after reset");
    }

    Ok(())
}

async fn query_devices(query_fields: &str, format: OutputFormat, local_only: bool) -> Result<()> {
    let backend = backend::Backend::new().await?;
    let mut devices = backend.get_initial_data().await?;

    if local_only {
        devices.retain(|data| !data.device_info.board_type.to_string().ends_with(" R"));
    }

    let fields: Result<Vec<QueryField>, String> = query_fields
        .split(',')
        .map(|s| QueryField::from_str(s.trim()))
        .collect();

    let fields = fields.map_err(|e| anyhow::anyhow!(e))?;

    match format {
        OutputFormat::Csv => output_csv(&devices, &fields).await?,
        OutputFormat::Json => output_query_json(&devices, &fields).await?,
        OutputFormat::Table => output_table(&devices, &fields).await?,
    }

    Ok(())
}

async fn output_csv(devices: &[telemetry::TelemetryData], fields: &[QueryField]) -> Result<()> {
    let headers: Vec<&str> = fields.iter().map(|f| f.as_str()).collect();
    println!("{}", headers.join(","));

    for device in devices {
        let values: Vec<String> = fields
            .iter()
            .map(|field| get_field_value(device, field))
            .collect();
        println!("{}", values.join(","));
    }

    Ok(())
}

async fn output_query_json(
    devices: &[telemetry::TelemetryData],
    fields: &[QueryField],
) -> Result<()> {
    let output = serde_json::json!({
        "devices": devices.iter().map(|device| {
            fields.iter().map(|field| {
                (field.as_str().to_string(), serde_json::Value::String(get_field_value(device, field)))
            }).collect::<serde_json::Map<String, serde_json::Value>>()
        }).collect::<Vec<_>>()
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

async fn output_table(devices: &[telemetry::TelemetryData], fields: &[QueryField]) -> Result<()> {
    use crossterm::style::Stylize;

    let field_strs: Vec<&str> = fields.iter().map(|f| f.as_str()).collect();
    let mut widths: Vec<usize> = field_strs.iter().map(|f| f.len()).collect();
    for device in devices {
        for (i, field) in fields.iter().enumerate() {
            let value = get_field_value(device, field);
            widths[i] = widths[i].max(value.len());
        }
    }

    for (i, field_str) in field_strs.iter().enumerate() {
        if i > 0 {
            print!(" │ ");
        }
        print!("{:width$}", field_str.bold(), width = widths[i]);
    }
    println!();

    for (i, width) in widths.iter().enumerate() {
        if i > 0 {
            print!("─┼─");
        }
        print!("{}", "─".repeat(*width));
    }
    println!();

    for device in devices {
        for (i, field) in fields.iter().enumerate() {
            if i > 0 {
                print!(" │ ");
            }
            let value = get_field_value(device, field);
            print!("{:width$}", value, width = widths[i]);
        }
        println!();
    }

    Ok(())
}

fn get_field_value(device: &telemetry::TelemetryData, field: &QueryField) -> String {
    match field {
        QueryField::Index => device.device_info.index.to_string(),
        QueryField::BusId => device.device_info.bus_id.clone(),
        QueryField::BoardType => device.device_info.board_type.to_string(),
        QueryField::BoardId => device.device_info.board_id.clone(),
        QueryField::Coords => device.device_info.coords.to_string(),
        QueryField::DramStatus => device.device_info.dram_status.to_string(),
        QueryField::DramSpeed => format!("{} MHz", device.device_info.dram_speed),
        QueryField::PcieSpeed => device.device_info.pcie_link_speed.to_string(),
        QueryField::PcieWidth => format!("x{}", device.device_info.pcie_link_width),
        QueryField::PcieMaxSpeed => device.device_info.pcie_max_speed.to_string(),
        QueryField::PcieMaxWidth => format!("x{}", device.device_info.pcie_max_width),

        QueryField::Voltage => format!("{:.3}", device.telemetry.voltage),
        QueryField::Current => format!("{:.1}", device.telemetry.current),
        QueryField::Power => format!("{:.1}", device.telemetry.power),
        QueryField::Temperature => format!("{:.1}", device.telemetry.temperature),
        QueryField::Aiclk => device.telemetry.aiclk.to_string(),
        QueryField::Heartbeat => device.telemetry.heartbeat.to_string(),

        QueryField::FwBundleVersion => device.firmware_info.fw_bundle_version.clone(),
        QueryField::TtFlashVersion => device.firmware_info.tt_flash_version.clone(),
        QueryField::CmFwVersion => device.firmware_info.cm_fw_version.clone(),
        QueryField::CmFwDate => device.firmware_info.cm_fw_date.clone(),
        QueryField::EthFwVersion => device.firmware_info.eth_fw_version.clone(),
        QueryField::BmBlVersion => device.firmware_info.bm_bl_version.clone(),
        QueryField::BmAppVersion => device.firmware_info.bm_app_version.clone(),
    }
}
