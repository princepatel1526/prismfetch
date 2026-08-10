use std::env;
use std::ffi::CString;
use std::fs;
use std::os::raw::{c_char, c_int, c_ulong};
use std::process::Command;

pub struct SystemInfo {
    pub os: String,
    pub kernel: String,
    pub shell: String,
    pub compositor: String,
    pub window_manager: String,
    pub terminal: String,
    pub cpu: String,
    pub gpu: String,
    pub ram_used_gb: f64,
    pub ram_total_gb: f64,
    pub packages: Option<usize>,
    pub theme: String,
    pub icons: String,
    pub font: String,
    pub uptime: String,
    pub cpu_pct: f64,
    pub mem_pct: f64,
    pub disk_pct: f64,
    pub temp_c: Option<f64>,
}

impl SystemInfo {
    pub fn collect() -> Self {
        let (ram_used_gb, ram_total_gb, mem_pct) = memory();
        let (theme, icons, font) = appearance_settings();
        let (compositor, window_manager) = compositor_and_wm();
        Self {
            os: os_name(),
            kernel: read_trimmed("/proc/sys/kernel/osrelease").unwrap_or_else(|| "Unknown".into()),
            shell: env::var("PRISM_SHELL_NAME").unwrap_or_else(|_| "Prism Shell".into()),
            compositor,
            window_manager,
            terminal: terminal_name(),
            cpu: cpu_name(),
            gpu: gpu_name(),
            ram_used_gb,
            ram_total_gb,
            packages: package_count(),
            theme,
            icons,
            font,
            uptime: uptime(),
            cpu_pct: cpu_usage(),
            mem_pct,
            disk_pct: disk_usage(),
            temp_c: temperature(),
        }
    }
}

fn read_trimmed(path: &str) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().into())
        .filter(|s: &String| !s.is_empty())
}

fn os_name() -> String {
    let release = fs::read_to_string("/etc/os-release").unwrap_or_default();
    let is_prism = release
        .lines()
        .any(|l| l == "ID=prismos" || l == "ID=\"prismos\"");
    if is_prism {
        release
            .lines()
            .find_map(|l| {
                l.strip_prefix("PRETTY_NAME=")
                    .map(|v| v.trim_matches('"').into())
            })
            .unwrap_or_else(|| "PrismOS 1.0 Aurora".into())
    } else {
        "PrismOS 1.0 Aurora".into()
    }
}

fn compositor_and_wm() -> (String, String) {
    let compositor = match env::var("XDG_SESSION_TYPE")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "x11" => "X11",
        _ => "Wayland",
    }
    .into();
    let wm = env::var("PRISM_WM_NAME")
        .or_else(|_| env::var("XDG_CURRENT_DESKTOP"))
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Prism Shell".into());
    (compositor, wm)
}

fn terminal_name() -> String {
    env::var("PRISM_TERMINAL_NAME")
        .or_else(|_| env::var("TERM_PROGRAM"))
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Prism Terminal".into())
}

fn cpu_name() -> String {
    fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|s| {
            s.lines().find_map(|l| {
                let (key, value) = l.split_once(':')?;
                matches!(key.trim(), "model name" | "Hardware").then(|| value.trim().into())
            })
        })
        .unwrap_or_else(|| "Unknown CPU".into())
}

fn gpu_name() -> String {
    Command::new("lspci")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .find(|l| {
                    let l = l.to_ascii_lowercase();
                    l.contains("vga") || l.contains("3d controller")
                })
                .and_then(|l| l.split_once(": ").map(|(_, v)| v.trim().into()))
        })
        .unwrap_or_else(|| "Unknown GPU".into())
}

fn memory() -> (f64, f64, f64) {
    let text = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let kb = |name: &str| {
        text.lines()
            .find(|l| l.starts_with(name))
            .and_then(|l| l.split_whitespace().nth(1)?.parse::<u64>().ok())
            .unwrap_or(0)
    };
    let total = kb("MemTotal:");
    let available = kb("MemAvailable:");
    let used = total.saturating_sub(available);
    let gb = 1024.0 * 1024.0;
    (
        used as f64 / gb,
        total as f64 / gb,
        if total > 0 {
            used as f64 * 100.0 / total as f64
        } else {
            0.0
        },
    )
}

fn package_count() -> Option<usize> {
    [
        ("pacman", &["-Qq"][..]),
        ("dpkg-query", &["-f", ".\n", "-W"]),
    ]
    .iter()
    .find_map(|(cmd, args)| {
        Command::new(cmd)
            .args(*args)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
            .filter(|n| *n > 0)
    })
}

fn appearance_settings() -> (String, String, String) {
    let setting = |key: &str, default: &str| {
        env::var(key)
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| default.into())
    };
    (
        setting("PRISM_THEME", "Aurora Glass"),
        setting("PRISM_ICON_THEME", "Prism Icons"),
        setting("PRISM_FONT", "Inter"),
    )
}

fn uptime() -> String {
    let seconds = fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse::<f64>().ok())
        .unwrap_or(0.0) as u64;
    let days = seconds / 86400;
    let hours = seconds % 86400 / 3600;
    let minutes = seconds % 3600 / 60;
    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else {
        format!("{hours}h {minutes}m")
    }
}

fn cpu_usage() -> f64 {
    let cores = fs::read_to_string("/proc/cpuinfo")
        .map(|s| s.lines().filter(|l| l.starts_with("processor")).count())
        .unwrap_or(1)
        .max(1);
    fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse::<f64>().ok())
        .map(|load| (load * 100.0 / cores as f64).clamp(0.0, 100.0))
        .unwrap_or(0.0)
}

#[repr(C)]
struct StatVfs {
    bsize: c_ulong,
    frsize: c_ulong,
    blocks: c_ulong,
    bfree: c_ulong,
    bavail: c_ulong,
    files: c_ulong,
    ffree: c_ulong,
    favail: c_ulong,
    fsid: c_ulong,
    flag: c_ulong,
    namemax: c_ulong,
    spare: [c_int; 6],
}
unsafe extern "C" {
    fn statvfs(path: *const c_char, buf: *mut StatVfs) -> c_int;
}
fn disk_usage() -> f64 {
    let mut stat: StatVfs = unsafe { std::mem::zeroed() };
    let path = CString::new("/").unwrap();
    if unsafe { statvfs(path.as_ptr(), &mut stat) } != 0 || stat.blocks == 0 {
        return 0.0;
    }
    ((stat.blocks - stat.bavail) as f64 * 100.0 / stat.blocks as f64).clamp(0.0, 100.0)
}

fn temperature() -> Option<f64> {
    let roots = ["/sys/class/thermal", "/sys/class/hwmon"];
    for root in roots {
        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                let candidates = [path.join("temp"), path.join("temp1_input")];
                for file in candidates {
                    if let Ok(raw) = fs::read_to_string(file) {
                        if let Ok(mut value) = raw.trim().parse::<f64>() {
                            if value > 1000.0 {
                                value /= 1000.0;
                            }
                            if (1.0..=130.0).contains(&value) {
                                return Some(value);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}
