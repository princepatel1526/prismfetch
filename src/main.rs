mod ascii;
mod collector;
mod render;

use collector::SystemInfo;

fn main() {
    let info = SystemInfo::collect();
    render::print_report(&info);
}
