use crate::{ansi, buffer::Buffer, config::Config, key::Key};
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader, Write},
    path::PathBuf,
};

pub struct Editor {
    /// List of buffers. The active buffer is always first.
    buffers: HashMap<usize, BufferHandle>,
    active_buf: usize,
    config: Config,
    buf_id_gen: usize,
}

impl Editor {
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

    fn load_new_buffer(&mut self, path: Option<PathBuf>) -> io::Result<()> {
        let bh = BufferHandle::load_buffer(path)?;
        self.buf_id_gen += 1;

        let id = self.buf_id_gen;
        self.buffers.insert(id, bh);
        self.active_buf = id;
        Ok(())
    }

    fn switch_to_buffer(&mut self, id: usize) -> bool {
        if self.buffers.contains_key(&id) {
            self.active_buf = id;
            true
        } else {
            false
        }
    }

    fn save(&self) -> io::Result<()> {
        if let Some(bh) = self.active_buf() {
            if let Some(path) = &bh.file {
                let mut file = File::create(path)?;
                for (_, ln) in bh.buf.lines() {
                    writeln!(file, "{}", ln)?;
                }
            }
        }
        Ok(())
    }

    fn render(&self) {
        print!("{}", ansi::CLEAR_SCREEN);

        if let Some(bh) = self.active_buf() {
            for (i, ln) in bh.buf.lines() {
                print!("{}{}", ansi::move_to(i, 0), ln);
            }
            print!("{}", ansi::move_to(bh.pos.row, bh.pos.col));
        }
    }

    fn insert_char(&mut self, ch: char) {
        if let Some(bh) = self.active_buf_mut() {
            bh.buf.insert_char(ch, bh.pos.row, bh.pos.col);
            bh.pos.col += 1;
        }
    }

    fn remove_char(&mut self) {
        if let Some(bh) = self.active_buf_mut() {
            if bh.pos.col > 0 {
                bh.buf.remove_char(bh.pos.row, bh.pos.col);
                bh.pos.col = bh.pos.col.saturating_sub(1);
            } else if bh.pos.row > 0 {
                let join_line_cols = bh.buf.cols_in_row(bh.pos.row);
                bh.buf.remove_char(bh.pos.row, bh.pos.col);
                bh.pos.row = bh.pos.row.saturating_sub(1);
                bh.pos.col = bh.buf.cols_in_row(bh.pos.row) - join_line_cols;
            }
        }
    }

    fn break_line(&mut self) {
        if let Some(bh) = self.active_buf_mut() {
            bh.buf.break_line(bh.pos.row, bh.pos.col);
            bh.pos.row += 1;
            bh.pos.col = 0;
        }
    }

    fn move_cursor_up(&mut self) {
        if let Some(bh) = self.active_buf_mut() {
            bh.pos.row = bh.pos.row.saturating_sub(1);
            bh.pos.col = bh.pos.col.min(bh.buf.cols_in_row(bh.pos.row))
        }
    }

    fn move_cursor_down(&mut self) {
        if let Some(bh) = self.active_buf_mut() {
            bh.pos.row = bh.pos.row.saturating_add(1).min(bh.buf.rows() - 1);
            bh.pos.col = bh.pos.col.min(bh.buf.cols_in_row(bh.pos.row))
        }
    }

    fn move_cursor_left(&mut self) {
        if let Some(bh) = self.active_buf_mut() {
            bh.pos.col = bh.pos.col.saturating_sub(1);
        }
    }

    fn move_cursor_right(&mut self) {
        if let Some(bh) = self.active_buf_mut() {
            bh.pos.col = bh
                .pos
                .col
                .saturating_add(1)
                .min(bh.buf.cols_in_row(bh.pos.row));
        }
    }

    fn active_buf(&self) -> Option<&BufferHandle> {
        self.buffers.get(&self.active_buf)
    }

    fn active_buf_mut(&mut self) -> Option<&mut BufferHandle> {
        self.buffers.get_mut(&self.active_buf)
    }
}

struct BufferHandle {
    buf: Buffer,
    pos: Position,
    file: Option<PathBuf>,
}

impl BufferHandle {
    fn load_buffer(path: Option<PathBuf>) -> io::Result<BufferHandle> {
        match path {
            Some(path) => match File::open(&path) {
                Ok(file) => {
                    let lines = BufReader::new(file)
                        .lines()
                        .collect::<Result<Vec<_>, _>>()?;

                    Ok(BufferHandle {
                        buf: Buffer::from_lines(lines),
                        pos: Position::default(),
                        file: Some(path),
                    })
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(BufferHandle {
                    buf: Buffer::empty(),
                    pos: Position::default(),
                    file: Some(path),
                }),

                Err(e) => return Err(e),
            },
            None => Ok(BufferHandle {
                buf: Buffer::empty(),
                pos: Position::default(),
                file: None,
            }),
        }
    }
}

#[derive(Default)]
struct Position {
    row: usize,
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
