mod args;

use crate::args::Args;
use red_core::{Config, Editor, RawMode};
use std::io::{self, Write};

fn clear_screen() -> io::Result<()> {
    print!(
        "{}{}",
        red_core::ansi::CLEAR_SCREEN,
        red_core::ansi::move_to(0, 0)
    );
    io::stdout().flush()
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let config = Config::load();
    let _rm = RawMode::new();

    let mut editor = Editor::open(config, args.file)?;

    clear_screen()?;
    editor.run()?;
    clear_screen()
}
