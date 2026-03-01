pub mod ansi;

pub use config::Config;
pub use raw_mode::RawMode;
pub use editor::Editor;

mod buffer;
mod config;
mod editor;
mod key;
mod raw_mode;

#[derive(Default)]
pub(crate) struct Position {
    row: usize,
    col: usize,
}

