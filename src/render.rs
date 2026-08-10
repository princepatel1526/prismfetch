use crate::ascii::logo_lines;
use crate::collector::SystemInfo;
use colored::{Color, Colorize};
use whoami::fallible;

const LABEL_COLOR: Color = Color::TrueColor { r: 138, g: 99, b: 246 }; // purple
const VALUE_COLOR: Color = Color::TrueColor { r: 226, g: 232, b: 240 }; // near-white

pub fn print_report(info: &SystemInfo) {
    let user = fallible::username().unwrap_or_else(|_| "user".to_string());
    let host = fallible::hostname().unwrap_or_else(|_| "prismos".to_string());

    let mut right: Vec<String> = Vec::new();
    right.push(format!(
        "{}@{}",
        user.bold().color(Color::TrueColor { r: 56, g: 139, b: 253 }),
        host.bold().color(Color::TrueColor { r: 217, g: 70, b: 239 })
    ));
    right.push("-".repeat(user.len() + host.len() + 1).dimmed().to_string());

    let rows: [(&str, String); 14] = [
        ("OS", info.os.clone()),
        ("Kernel", info.kernel.clone()),
        ("Shell", info.shell.clone()),
        ("Compositor", info.compositor.clone()),
        ("Window Manager", info.window_manager.clone()),
        ("Terminal", info.terminal.clone()),
        ("CPU", info.cpu.clone()),
        ("GPU", info.gpu.clone()),
        (
            "RAM",
            format!("{:.1} GB / {:.1} GB", info.ram_used_gb, info.ram_total_gb),
        ),
        ("Packages", info.packages.to_string()),
        ("Theme", info.theme.clone()),
        ("Icons", info.icons.clone()),
        ("Font", info.font.clone()),
        ("Uptime", info.uptime.clone()),
    ];

    for (label, value) in rows {
        right.push(format!(
            "{}{} {}",
            label.color(LABEL_COLOR).bold(),
            ":".dimmed(),
            value.color(VALUE_COLOR)
        ));
    }

    let logo = logo_lines();
    let logo_width = 12 * 2; // 12 cells, 2 chars each
    let total_lines = logo.len().max(right.len());

    println!();
    for i in 0..total_lines {
        let left = logo.get(i).cloned().unwrap_or_else(|| " ".repeat(logo_width));
        let right_line = right.get(i).cloned().unwrap_or_default();
        println!("  {}   {}", left, right_line);
    }
    println!();

    print_bar("CPU Usage", info.cpu_pct, 100.0, format!("{:.0}%", info.cpu_pct), bar_color(info.cpu_pct));
    print_bar("Memory Usage", info.mem_pct, 100.0, format!("{:.0}%", info.mem_pct), bar_color(info.mem_pct));
    print_bar(
        "Disk Usage",
        info.disk_pct,
        100.0,
        format!("{:.0}%", info.disk_pct),
        bar_color(info.disk_pct),
    );
    if info.temp_c > 0.0 {
        print_bar(
            "Temperature",
            info.temp_c,
            100.0,
            format!("{:.0}°C", info.temp_c),
            temp_color(info.temp_c),
        );
    }
    println!();
}

fn bar_color(pct: f64) -> Color {
    if pct < 50.0 {
        Color::TrueColor { r: 56, g: 189, b: 248 } // cyan
    } else if pct < 80.0 {
        Color::TrueColor { r: 217, g: 70, b: 239 } // magenta
    } else {
        Color::TrueColor { r: 248, g: 113, b: 113 } // red
    }
}

fn temp_color(temp: f64) -> Color {
    if temp < 60.0 {
        Color::TrueColor { r: 251, g: 191, b: 36 } // amber
    } else {
        Color::TrueColor { r: 248, g: 113, b: 113 } // red
    }
}

fn print_bar(label: &str, value: f64, max: f64, suffix: String, color: Color) {
    const WIDTH: usize = 30;
    let ratio = (value / max).clamp(0.0, 1.0);
    let filled = (ratio * WIDTH as f64).round() as usize;
    let empty = WIDTH - filled;

    let bar = format!(
        "{}{}",
        "█".repeat(filled).color(color),
        "░".repeat(empty).dimmed()
    );

    println!(
        "  {:<14} {}  {}",
        label.color(VALUE_COLOR),
        bar,
        suffix.bold().color(color)
    );
}
