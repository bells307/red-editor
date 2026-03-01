pub const CLEAR_SCREEN: &str = "\x1b[2J";
pub const HIDE_CURSOR: &str = "\x1b[?25l";
pub const SHOW_CURSOR: &str = "\x1b[?25h";

pub fn move_to(row: usize, col: usize) -> String {
    format!("\x1b[{};{}H", row + 1, col + 1) // 1-indexed
}

pub fn clear_line() -> &'static str {
    "\x1b[2K"
}
