use crate::{ansi, buffer::Buffer, config::Config, key::Key};
use std::collections::BTreeMap;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader, Write},
    path::PathBuf,
};

/// The top-level editor state. Manages a collection of open buffers and
/// drives the main event loop.
pub struct Editor {
    /// All open buffers, keyed by their unique buffer ID.
    buffers: HashMap<usize, BufferHandle>,
    /// ID of the currently active buffer.
    active_buf: usize,
    /// Editor configuration (key bindings, display options, etc.).
    config: Config,
    /// Monotonically increasing counter used to assign unique IDs to new buffers.
    buf_id_gen: usize,
}

impl Editor {
    /// Creates an [`Editor`] instance with a single buffer.
    ///
    /// If `path` is provided the buffer is loaded from that file;
    /// otherwise an empty scratch buffer is opened.
    pub fn open(config: Config, path: Option<PathBuf>) -> io::Result<Self> {
        let bh = BufferHandle::load_buffer(path)?;
        let buf_id_gen = 0;
        let mut buffers = HashMap::with_capacity(1);
        buffers.insert(buf_id_gen, bh);

        Ok(Self {
            buffers,
            active_buf: 0,
            config,
            buf_id_gen,
        })
    }

    /// Starts the main event loop. Blocks until the user quits.
    pub fn run(&mut self) -> io::Result<()> {
        self.render();
        io::stdout().flush()?;

        loop {
            match Key::read()? {
                Key::Ctrl('c') | Key::Escape => break,
                Key::Ctrl('s') => self.save()?,
                Key::Ctrl('n') => self.load_new_buffer(None)?,
                Key::Tab => {
                    self.switch_to_buffer(self.active_buf.saturating_add(1));
                }
                Key::ShiftTab => {
                    self.switch_to_buffer(self.active_buf.saturating_sub(1));
                }
                Key::Char(c) => self.insert_char(c),
                Key::Enter => self.break_line(),
                Key::Backspace => self.remove_char(),
                Key::ArrowUp => self.move_cursor_up(),
                Key::ArrowDown => self.move_cursor_down(),
                Key::ArrowLeft => self.move_cursor_left(),
                Key::ArrowRight => self.move_cursor_right(),
                Key::Unknown(buf) => print!("{}", to_caret_notation(&buf)),
                _ => {}
            }
            self.render();
            io::stdout().flush()?;
        }

        Ok(())
    }

    /// Opens a new buffer, optionally backed by file, and makes it active.
    fn load_new_buffer(&mut self, path: Option<PathBuf>) -> io::Result<()> {
        let bh = BufferHandle::load_buffer(path)?;
        self.buf_id_gen += 1;

        let id = self.buf_id_gen;
        self.buffers.insert(id, bh);
        self.active_buf = id;
        Ok(())
    }

    /// Switches the active buffer to the one with the given `id`.
    ///
    /// Returns `true` if the buffer exists, `false` otherwise.
    fn switch_to_buffer(&mut self, id: usize) -> bool {
        if self.buffers.contains_key(&id) {
            self.active_buf = id;
            true
        } else {
            false
        }
    }

    /// Writes the active buffer to its associated file.
    ///
    /// Does nothing if the buffer has no file path (scratch buffer).
    fn save(&self) -> io::Result<()> {
        if let Some(bh) = self.active_buf()
            && let Some(path) = &bh.file
        {
            let mut file = File::create(path)?;
            for (_, ln) in bh.buf.lines() {
                writeln!(file, "{}", ln)?;
            }
        }
        Ok(())
    }

    /// Clears the terminal and redraws the active buffer, then repositions the cursor.
    fn render(&self) {
        print!("{}", ansi::CLEAR_SCREEN);

        if let Some(bh) = self.active_buf() {
            for (i, ln) in bh.buf.lines() {
                print!("{}{}", ansi::move_to(i, 0), ln);
            }
            print!("{}", ansi::move_to(bh.cursor.row, bh.cursor.col));
        }
    }

    /// Inserts `ch` at the cursor position.
    fn insert_char(&mut self, ch: char) {
        if let Some(bh) = self.active_buf_mut() {
            bh.buf.insert_char(ch, bh.cursor.row, bh.cursor.col);
            bh.cursor.col += 1;
        }
    }

    /// Deletes the character to the left of the cursor (backspace).
    ///
    /// If the cursor is at the beginning of a line, the current line is
    /// joined with the previous one and the cursor moves to the join point.
    fn remove_char(&mut self) {
        if let Some(bh) = self.active_buf_mut() {
            if bh.cursor.col > 0 {
                bh.buf.remove_char(bh.cursor.row, bh.cursor.col);
                bh.cursor.col = bh.cursor.col.saturating_sub(1);
            } else if bh.cursor.row > 0 {
                let join_line_cols = bh.buf.cols_in_row(bh.cursor.row);
                bh.buf.remove_char(bh.cursor.row, bh.cursor.col);
                bh.cursor.row = bh.cursor.row.saturating_sub(1);
                bh.cursor.col = bh.buf.cols_in_row(bh.cursor.row) - join_line_cols;
            }
        }
    }

    /// Splits the current line at the cursor column and moves the cursor to the start of the new line.
    fn break_line(&mut self) {
        if let Some(bh) = self.active_buf_mut() {
            bh.buf.break_line(bh.cursor.row, bh.cursor.col);
            bh.cursor.row += 1;
            bh.cursor.col = 0;
        }
    }

    /// Moves the cursor one row up, clamping the column to the new line's length.
    fn move_cursor_up(&mut self) {
        if let Some(bh) = self.active_buf_mut() {
            bh.cursor.row = bh.cursor.row.saturating_sub(1);
            bh.cursor.col = bh.cursor.col.min(bh.buf.cols_in_row(bh.cursor.row))
        }
    }

    /// Moves the cursor one row down, clamping to the last row and the new line's length.
    fn move_cursor_down(&mut self) {
        if let Some(bh) = self.active_buf_mut() {
            bh.cursor.row = bh.cursor.row.saturating_add(1).min(bh.buf.rows() - 1);
            bh.cursor.col = bh.cursor.col.min(bh.buf.cols_in_row(bh.cursor.row))
        }
    }

    /// Moves the cursor one column to the left, stopping at column 0.
    fn move_cursor_left(&mut self) {
        if let Some(bh) = self.active_buf_mut() {
            bh.cursor.col = bh.cursor.col.saturating_sub(1);
        }
    }

    /// Moves the cursor one column to the right, stopping at the end of the line.
    fn move_cursor_right(&mut self) {
        if let Some(bh) = self.active_buf_mut() {
            bh.cursor.col = bh
                .cursor
                .col
                .saturating_add(1)
                .min(bh.buf.cols_in_row(bh.cursor.row));
        }
    }

    /// Returns a shared reference to the active [`BufferHandle`], if one exists.
    fn active_buf(&self) -> Option<&BufferHandle> {
        self.buffers.get(&self.active_buf)
    }

    /// Returns a mutable reference to the active [`BufferHandle`], if one exists.
    fn active_buf_mut(&mut self) -> Option<&mut BufferHandle> {
        self.buffers.get_mut(&self.active_buf)
    }
}

/// Pairs a [`Buffer`] with its cursor position and the file it originated from.
struct BufferHandle {
    /// The text content of the buffer.
    buf: Buffer,
    /// Current cursor position within this buffer.
    cursor: Position,
    /// Path to the file on disk, or `None` for scratch buffers.
    file: Option<PathBuf>,
}

impl BufferHandle {
    /// Creates a [`BufferHandle`] for `path`.
    ///
    /// - If `path` is `Some` and the file exists, its contents are read into the buffer.
    /// - If `path` is `Some` but the file is not found, an empty buffer is created and
    ///   the path is retained so a subsequent save will create the file.
    /// - If `path` is `None`, an empty scratch buffer with no associated file is returned.
    fn load_buffer(path: Option<PathBuf>) -> io::Result<BufferHandle> {
        match path {
            Some(path) => match File::open(&path) {
                Ok(file) => {
                    let lines = BufReader::new(file)
                        .lines()
                        .collect::<Result<Vec<_>, _>>()?;

                    Ok(BufferHandle {
                        buf: Buffer::from_lines(lines),
                        cursor: Position::default(),
                        file: Some(path),
                    })
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(BufferHandle {
                    buf: Buffer::scratch(),
                    cursor: Position::default(),
                    file: Some(path),
                }),

                Err(e) => return Err(e),
            },
            None => Ok(BufferHandle {
                buf: Buffer::scratch(),
                cursor: Position::default(),
                file: None,
            }),
        }
    }
}

/// A zero-based (row, column) position in a buffer.
#[derive(Default)]
struct Position {
    /// Zero-based line index.
    row: usize,
    /// Zero-based byte offset within the line.
    col: usize,
}

fn to_caret_notation(buf: &[u8]) -> String {
    buf.iter()
        .map(|&b| match b {
            0..=26 => format!("^{}", (b + 64) as char),
            27 => "^[".to_string(),
            28..=31 => format!("^{}", (b + 64) as char),
            b => (b as char).to_string(),
        })
        .collect()
}
