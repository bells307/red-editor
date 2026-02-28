pub(crate) const CLEAR_SCREEN: &str = "\x1b[2J";
pub(crate) const HIDE_CURSOR: &str = "\x1b[?25l";
pub(crate) const SHOW_CURSOR: &str = "\x1b[?25h";

pub(crate) fn move_to(row: usize, col: usize) -> String {
    format!("\x1b[{};{}H", row + 1, col + 1) // 1-indexed
}

pub(crate) fn clear_line() -> &'static str {
    "\x1b[2K"
}
