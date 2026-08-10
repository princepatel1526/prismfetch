use crate::ascii::{logo_width, LOGO};
use crate::collector::SystemInfo;
use std::env;
use std::io::{self, IsTerminal};

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const WHITE: (u8, u8, u8) = (229, 231, 235);
const GRAY: (u8, u8, u8) = (156, 163, 175);
const CYAN: (u8, u8, u8) = (41, 213, 255);
const BLUE: (u8, u8, u8) = (66, 133, 255);
const INDIGO: (u8, u8, u8) = (99, 102, 241);
const VIOLET: (u8, u8, u8) = (139, 92, 246);
const PURPLE: (u8, u8, u8) = (168, 85, 247);
const MAGENTA: (u8, u8, u8) = (217, 70, 239);
const PINK: (u8, u8, u8) = (236, 72, 153);
const ORANGE: (u8, u8, u8) = (245, 158, 11);

struct Terminal {
    color: bool,
    unicode: bool,
    width: usize,
}

impl Terminal {
    fn detect() -> Self {
        let tty = io::stdout().is_terminal();
        let width = env::var("COLUMNS")
            .ok()
            .and_then(|v| v.parse().ok())
            .filter(|v| *v >= 20)
            .or_else(terminal_columns)
            .unwrap_or(100);
        let term = env::var("TERM").unwrap_or_default();
        let locale = env::var("LC_ALL")
            .or_else(|_| env::var("LC_CTYPE"))
            .or_else(|_| env::var("LANG"))
            .unwrap_or_default()
            .to_ascii_lowercase();
        Self {
            color: tty && env::var_os("NO_COLOR").is_none() && term != "dumb",
            unicode: term != "linux" && (locale.contains("utf-8") || locale.contains("utf8")),
            width: width.clamp(20, 160),
        }
    }

    fn paint(&self, text: &str, color: (u8, u8, u8), bold: bool) -> String {
        if !self.color {
            return text.to_string();
        }
        format!(
            "{}\x1b[38;2;{};{};{}m{}{}",
            if bold { BOLD } else { "" },
            color.0,
            color.1,
            color.2,
            text,
            RESET
        )
    }
}

#[cfg(target_os = "linux")]
fn terminal_columns() -> Option<usize> {
    #[repr(C)]
    struct WinSize {
        rows: u16,
        cols: u16,
        xpixel: u16,
        ypixel: u16,
    }
    unsafe extern "C" {
        fn ioctl(fd: i32, request: u64, ...) -> i32;
    }
    let mut size = WinSize {
        rows: 0,
        cols: 0,
        xpixel: 0,
        ypixel: 0,
    };
    // TIOCGWINSZ is part of Linux's stable userspace ABI. stdout is fd 1.
    let result = unsafe { ioctl(1, 0x5413, &mut size) };
    (result == 0 && size.cols > 0).then_some(size.cols as usize)
}

#[cfg(not(target_os = "linux"))]
fn terminal_columns() -> Option<usize> {
    None
}

pub fn print_report(info: &SystemInfo) {
    let t = Terminal::detect();
    let rows = info_rows(info);
    let rule = if t.unicode { "─" } else { "-" };
    println!();
    println!("{}", t.paint(&rule.repeat(t.width), GRAY, false));
    if t.width >= 100 {
        render_side_by_side(&t, &rows);
    } else {
        render_stacked(&t, &rows);
    }
    println!("{}", t.paint(&rule.repeat(t.width), GRAY, false));
    render_meters(&t, info);
    println!();
}

fn info_rows(info: &SystemInfo) -> Vec<(&'static str, String, (u8, u8, u8))> {
    vec![
        ("OS", info.os.clone(), CYAN),
        ("Kernel", format!("Linux {}", info.kernel), CYAN),
        ("Shell", info.shell.clone(), CYAN),
        ("Compositor", info.compositor.clone(), CYAN),
        ("Window Manager", info.window_manager.clone(), CYAN),
        ("Terminal", info.terminal.clone(), CYAN),
        ("CPU", info.cpu.clone(), INDIGO),
        ("GPU", info.gpu.clone(), PURPLE),
        (
            "RAM",
            format!("{:.1} GB / {:.1} GB", info.ram_used_gb, info.ram_total_gb),
            PURPLE,
        ),
        (
            "Packages",
            info.packages
                .map_or_else(|| "N/A".into(), |v| v.to_string()),
            PURPLE,
        ),
        ("Theme", info.theme.clone(), VIOLET),
        ("Icons", info.icons.clone(), VIOLET),
        ("Font", info.font.clone(), BLUE),
        ("Uptime", info.uptime.clone(), BLUE),
    ]
}

fn render_side_by_side(t: &Terminal, rows: &[(&str, String, (u8, u8, u8))]) {
    let left_width = logo_width();
    let divider = if t.unicode { "│" } else { "|" };
    let logo_offset = (rows.len().saturating_sub(LOGO.len())) / 2;
    for i in 0..rows.len() {
        let raw = i
            .checked_sub(logo_offset)
            .and_then(|j| LOGO.get(j))
            .copied()
            .unwrap_or("");
        let logo = gradient_logo(t, raw);
        let padding = " ".repeat(left_width.saturating_sub(raw.chars().count()));
        let (label, value, color) = &rows[i];
        let fixed = 1 + left_width + 2 + 3 + label.chars().count() + 2;
        let value = shorten(value, t.width.saturating_sub(fixed));
        println!(
            " {}{}  {}  {} {}",
            logo,
            padding,
            t.paint(divider, GRAY, false),
            t.paint(&format!("{}:", label), *color, true),
            t.paint(&value, WHITE, false)
        );
    }
}

fn render_stacked(t: &Terminal, rows: &[(&str, String, (u8, u8, u8))]) {
    for line in LOGO {
        println!(" {}", gradient_logo(t, line.trim_end()));
    }
    println!();
    for (label, value, color) in rows {
        let value = shorten(value, t.width.saturating_sub(label.chars().count() + 4));
        println!(
            " {} {}",
            t.paint(&format!("{}:", label), *color, true),
            t.paint(&value, WHITE, false)
        );
    }
}

fn shorten(value: &str, width: usize) -> String {
    if value.chars().count() <= width {
        return value.to_string();
    }
    if width < 2 {
        return String::new();
    }
    let mut text: String = value.chars().take(width - 1).collect();
    text.push('…');
    text
}

fn gradient_logo(t: &Terminal, line: &str) -> String {
    let colors = [CYAN, BLUE, INDIGO, VIOLET, PURPLE, MAGENTA, PINK];
    let width = logo_width().max(1);
    line.chars()
        .enumerate()
        .map(|(i, ch)| {
            let color = colors[(i * colors.len() / width).min(colors.len() - 1)];
            t.paint(&ch.to_string(), color, true)
        })
        .collect()
}

fn render_meters(t: &Terminal, info: &SystemInfo) {
    let temp = if info.temp_c.is_some() {
        info.temp_c
    } else {
        None
    };
    meter(
        t,
        "CPU Usage",
        info.cpu_pct,
        format!("{:.0}%", info.cpu_pct),
        CYAN,
        BLUE,
    );
    meter(
        t,
        "Memory Usage",
        info.mem_pct,
        format!("{:.0}%", info.mem_pct),
        MAGENTA,
        VIOLET,
    );
    meter(
        t,
        "Disk Usage",
        info.disk_pct,
        format!("{:.0}%", info.disk_pct),
        (74, 168, 255),
        BLUE,
    );
    meter(
        t,
        "Temperature",
        temp.unwrap_or(0.0),
        temp.map_or_else(|| "N/A".into(), |v| format!("{v:.0}°C")),
        ORANGE,
        (180, 83, 9),
    );
}

fn meter(
    t: &Terminal,
    label: &str,
    value: f64,
    suffix: String,
    start: (u8, u8, u8),
    end: (u8, u8, u8),
) {
    let label_width = if t.width < 80 { 13 } else { 17 };
    let suffix_width = 5;
    let bar_width = t
        .width
        .saturating_sub(label_width + suffix_width + 8)
        .clamp(8, 64);
    let filled = ((value.clamp(0.0, 100.0) / 100.0) * bar_width as f64).round() as usize;
    let (full, empty) = if t.unicode {
        ('█', '░')
    } else {
        ('#', '-')
    };
    let first = filled / 2;
    let bar = format!(
        "{}{}{}",
        t.paint(&full.to_string().repeat(first), start, false),
        t.paint(&full.to_string().repeat(filled - first), end, false),
        if t.color {
            format!(
                "{DIM}{}{RESET}",
                empty.to_string().repeat(bar_width - filled)
            )
        } else {
            empty.to_string().repeat(bar_width - filled)
        }
    );
    println!(
        " {:<label_width$} [{}] {:>suffix_width$}",
        label, bar, suffix
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn logo_has_consistent_max_width() {
        assert!(logo_width() <= 45);
        assert_eq!(LOGO.len(), 7);
    }
    #[test]
    fn redirected_output_disables_color() {
        env::set_var("NO_COLOR", "1");
        assert!(!Terminal::detect().color);
    }
}
