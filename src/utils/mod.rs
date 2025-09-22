use sysinfo::System;

pub struct HostInfo {
    pub os: String,
    pub kernel: String,
    pub driver: String,
}

pub fn get_host_info() -> HostInfo {
    let mut sys = System::new_all();
    sys.refresh_all();

    let os = format!(
        "{} {}",
        System::name().unwrap_or_else(|| "Unknown".to_string()),
        System::os_version().unwrap_or_default()
    );

    let kernel = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());

    let driver = std::fs::read_to_string("/sys/module/tenstorrent/version")
        .map(|v| format!("tt-kmd {}", v.trim()))
        .unwrap_or_else(|_| "tt-kmd unknown".to_string());

    HostInfo { os, kernel, driver }
}
