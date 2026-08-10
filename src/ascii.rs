/// Seven-line, terminal-safe PRISMOS wordmark.  The renderer colors each
/// two-column slice, giving the mark an Aurora cyan-to-pink gradient.
pub const LOGO: [&str; 7] = [
    "████  ████  ███  ████  █   █  ███   ███ ",
    "█   █ █   █  █  █     ██ ██ █   █ █    ",
    "████  ████   █   ███  █ █ █ █   █  ███ ",
    "█     █ █    █      █ █   █ █   █     █",
    "█     █  █  ███ ████  █   █  ███  ████ ",
    "                                             ",
    "              P R I S M O S                  ",
];

pub fn logo_width() -> usize {
    LOGO.iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0)
}
