mod ansi;
mod buffer;
mod editor;
mod key;
mod raw_mode;

use crate::{editor::Editor, key::Key, raw_mode::RawMode};
use clap::Parser;
use std::{
    fs::File,
    io::{self, Write},
    path::PathBuf,
};

#[derive(Parser)]
#[command(about = "A simple terminal text editor")]
struct Args {
    /// File to open
    file: Option<PathBuf>,
}

fn clear_screen() -> io::Result<()> {
    print!("{}{}", ansi::CLEAR_SCREEN, ansi::move_to(0, 0));
    io::stdout().flush()
}

fn to_caret_notation(buf: &[u8]) -> String {
    buf.iter()
        .map(|&b| match b {
            0..=26 => format!("^{}", (b + 64) as char), // ^@ ^A ... ^Z
            27 => "^[".to_string(),                     // ESC
            28..=31 => format!("^{}", (b + 64) as char),
            b => (b as char).to_string(),
        })
        .collect()
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let _rm = RawMode::new();
    clear_screen()?;

    let mut editor = Editor::new();

    if let Some(path) = &args.file {
        let file = File::open(&path)?;
        editor.read_buffer(file)?;
    }

    editor.render();
    io::stdout().flush()?;

    loop {
        match Key::read()? {
            Key::Ctrl('c') | Key::Escape => break,
            Key::Ctrl('s') => {
                if let Some(path) = &args.file {
                    let mut file = File::create(&path)?;
                    editor.write_buffer(&mut file)?;
                }
            }
            Key::Char(c) => editor.insert_char(c),
            Key::Enter => editor.break_line(),
            Key::Backspace => editor.remove_char(),
            Key::ArrowUp => editor.move_cursor_up(),
            Key::ArrowDown => editor.move_cursor_down(),
            Key::ArrowLeft => editor.move_cursor_left(),
            Key::ArrowRight => editor.move_cursor_right(),
            Key::Unknown(buf) => print!("{}", to_caret_notation(&buf)),
            _ => {}
        }
        editor.render();
        io::stdout().flush()?;
    }

    clear_screen()
}
