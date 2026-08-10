use colored::{Color, Colorize};

/// Renders the PrismOS block-grid logo, left to right, top to bottom.
/// `0` = empty cell, `1` = blue, `2` = magenta/pink, `3` = purple (blend zone).
const LOGO: [[u8; 12]; 14] = [
    [1, 1, 0, 0, 1, 1, 1, 1, 0, 2, 2, 2],
    [1, 1, 0, 0, 1, 0, 1, 1, 0, 2, 0, 2],
    [0, 0, 0, 0, 1, 1, 1, 1, 0, 2, 2, 2],
    [1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 0, 0],
    [1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0],
    [1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 1, 0, 0, 3, 3, 0, 0, 2, 2, 0, 2],
    [1, 1, 0, 0, 3, 3, 0, 0, 2, 2, 0, 2],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1],
    [1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 0],
    [1, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
];

const BLUE: Color = Color::TrueColor { r: 56, g: 139, b: 253 };
const MAGENTA: Color = Color::TrueColor { r: 217, g: 70, b: 239 };
const PURPLE: Color = Color::TrueColor { r: 138, g: 99, b: 246 };

/// Returns the logo as a vector of already-colored, ready-to-print lines.
pub fn logo_lines() -> Vec<String> {
    LOGO.iter()
        .map(|row| {
            row.iter()
                .map(|cell| match cell {
                    1 => "██".color(BLUE).to_string(),
                    2 => "██".color(MAGENTA).to_string(),
                    3 => "██".color(PURPLE).to_string(),
                    _ => "  ".to_string(),
                })
                .collect::<String>()
        })
        .collect()
}
