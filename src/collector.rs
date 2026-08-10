use std::env;
use std::fs;
use std::process::Command;
use sysinfo::{Components, Disks, System};

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
    pub packages: usize,
    pub theme: String,
    pub icons: String,
    pub font: String,
    pub uptime: String,

    pub cpu_pct: f64,
    pub mem_pct: f64,
    pub disk_pct: f64,
    pub disk_used_gb: f64,
    pub disk_total_gb: f64,
    pub temp_c: f64,
}

impl SystemInfo {
    pub fn collect() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        // A second refresh of CPU usage is required because sysinfo needs a
        // delta between two samples to compute a meaningful percentage.
        std::thread::sleep(std::time::Duration::from_millis(200));
        sys.refresh_cpu();

        let os = os_pretty_name();
        let kernel = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
        let shell = shell_name();
        let (compositor, window_manager) = compositor_and_wm();
        let terminal = terminal_name();
        let cpu = cpu_name(&sys);
        let gpu = gpu_name();

        let ram_total_bytes = sys.total_memory();
        let ram_used_bytes = sys.used_memory();
        let ram_total_gb = ram_total_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
        let ram_used_gb = ram_used_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
        let mem_pct = if ram_total_bytes > 0 {
            ram_used_bytes as f64 / ram_total_bytes as f64 * 100.0
        } else {
            0.0
        };

        let packages = package_count();
        let (theme, icons, font) = appearance_settings();
        let uptime = format_uptime(System::uptime());

        let cpu_pct = if !sys.cpus().is_empty() {
            sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() as f64 / sys.cpus().len() as f64
        } else {
            0.0
        };

        let (disk_used_gb, disk_total_gb, disk_pct) = disk_usage(&sys);
        let temp_c = temperature(&sys);

        SystemInfo {
            os,
            kernel,
            shell,
            compositor,
            window_manager,
            terminal,
            cpu,
            gpu,
            ram_used_gb,
            ram_total_gb,
            packages,
            theme,
            icons,
            font,
            uptime,
            cpu_pct,
            mem_pct,
            disk_pct,
            disk_used_gb,
            disk_total_gb,
            temp_c,
        }
    }
}

fn os_pretty_name() -> String {
    if let Ok(contents) = fs::read_to_string("/etc/os-release") {
        for line in contents.lines() {
            if let Some(value) = line.strip_prefix("PRETTY_NAME=") {
                return value.trim_matches('"').to_string();
            }
        }
    }
    "Unknown Linux".to_string()
}

fn shell_name() -> String {
    env::var("SHELL")
        .ok()
        .and_then(|s| s.rsplit('/').next().map(String::from))
        .unwrap_or_else(|| "unknown".to_string())
}

fn compositor_and_wm() -> (String, String) {
    let session_type = env::var("XDG_SESSION_TYPE").unwrap_or_default();
    let desktop = env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| env::var("DESKTOP_SESSION"))
        .unwrap_or_else(|_| "Unknown".to_string());

    let compositor = if session_type.eq_ignore_ascii_case("wayland") {
        "Wayland".to_string()
    } else if session_type.eq_ignore_ascii_case("x11") {
        "X11".to_string()
    } else {
        "Unknown".to_string()
    };

    (compositor, desktop)
}

fn terminal_name() -> String {
    // Walk: $TERM_PROGRAM -> $TERM -> parent process name via /proc
    if let Ok(t) = env::var("TERM_PROGRAM") {
        if !t.is_empty() {
            return t;
        }
    }
    if let Ok(ppid_name) = fs::read_to_string(format!(
        "/proc/{}/comm",
        std::os::unix::process::parent_id()
    )) {
        return ppid_name.trim().to_string();
    }
    env::var("TERM").unwrap_or_else(|_| "unknown".to_string())
}

fn cpu_name(sys: &System) -> String {
    sys.cpus()
        .first()
        .map(|c| c.brand().trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Unknown CPU".to_string())
}

fn gpu_name() -> String {
    // Try lspci first (most Linux systems).
    if let Ok(output) = Command::new("lspci").output() {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            let lower = line.to_lowercase();
            if lower.contains("vga") || lower.contains("3d controller") {
                if let Some(idx) = line.find(": ") {
                    return line[idx + 2..].trim().to_string();
                }
            }
        }
    }
    "Unknown GPU".to_string()
}

fn package_count() -> usize {
    // Try common package managers in order; return the first that works.
    let managers: [(&str, &[&str]); 5] = [
        ("dpkg-query", &["-f", ".\n", "-W"]),
        ("pacman", &["-Qq"]),
        ("rpm", &["-qa"]),
        ("apk", &["info"]),
        ("xbps-query", &["-l"]),
    ];
    for (cmd, args) in managers {
        if let Ok(output) = Command::new(cmd).args(args).output() {
            if output.status.success() {
                let count = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .count();
                if count > 0 {
                    return count;
                }
            }
        }
    }
    0
}

fn appearance_settings() -> (String, String, String) {
    // Reasonable defaults for PrismOS; real values can be read from
    // gsettings on GNOME-based sessions if present.
    let gsettings = |schema: &str, key: &str| -> Option<String> {
        Command::new("gsettings")
            .args(["get", schema, key])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .trim()
                    .trim_matches('\'')
                    .to_string()
            })
    };

    let theme = gsettings("org.gnome.desktop.interface", "gtk-theme")
        .unwrap_or_else(|| "Aurora Glass".to_string());
    let icons = gsettings("org.gnome.desktop.interface", "icon-theme")
        .unwrap_or_else(|| "Prism Icons".to_string());
    let font = gsettings("org.gnome.desktop.interface", "font-name")
        .unwrap_or_else(|| "Inter".to_string());

    (theme, icons, font)
}

fn format_uptime(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    format!("{}h {}m", hours, minutes)
}

fn disk_usage(_sys: &System) -> (f64, f64, f64) {
    let disks = Disks::new_with_refreshed_list();
    let mut best: Option<(u64, u64)> = None; // (total, available)
    for disk in disks.list() {
        if disk.mount_point() == std::path::Path::new("/") {
            best = Some((disk.total_space(), disk.available_space()));
            break;
        }
    }
    // Fall back to the largest disk found if "/" wasn't matched exactly.
    let (total, available) = best.unwrap_or_else(|| {
        disks
            .list()
            .iter()
            .map(|d| (d.total_space(), d.available_space()))
            .max_by_key(|(t, _)| *t)
            .unwrap_or((0, 0))
    });

    let used = total.saturating_sub(available);
    let total_gb = total as f64 / 1024.0 / 1024.0 / 1024.0;
    let used_gb = used as f64 / 1024.0 / 1024.0 / 1024.0;
    let pct = if total > 0 {
        used as f64 / total as f64 * 100.0
    } else {
        0.0
    };
    (used_gb, total_gb, pct)
}

fn temperature(_sys: &System) -> f64 {
    // Average across CPU-related sensors if available.
    let components = Components::new_with_refreshed_list();
    let temps: Vec<f32> = components
        .list()
        .iter()
        .filter(|c| {
            let label = c.label().to_lowercase();
            label.contains("cpu") || label.contains("core") || label.contains("package")
        })
        .map(|c| c.temperature())
        .filter(|t| *t > 0.0)
        .collect();

    if temps.is_empty() {
        0.0
    } else {
        (temps.iter().sum::<f32>() / temps.len() as f32) as f64
    }
}
