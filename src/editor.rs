use crate::{ansi, buffer::Buffer};
use std::io::{self, BufRead, BufReader};

pub(crate) struct Editor {
    cur_row: usize,
    cur_col: usize,
    buf: Buffer,
}

impl Editor {
    pub(crate) fn new() -> Self {
        Self {
            cur_row: 0,
            cur_col: 0,
            buf: Buffer::empty(),
        }
    }

    pub(crate) fn read_buffer(&mut self, rdr: impl io::Read) -> io::Result<()> {
        let buf_rdr = BufReader::new(rdr);
        let mut lines = Vec::new();

        for ln in buf_rdr.lines() {
            let ln = ln?;
            lines.push(ln);
        }

        let buffer = Buffer::from_lines(lines);
        self.buf = buffer;

        Ok(())
    }

    pub(crate) fn write_buffer(&self, mut wrt: impl io::Write) -> io::Result<()> {
        for (_, ln) in self.buf.lines() {
            writeln!(wrt, "{}", ln)?;
        }
        Ok(())
    }

    pub(crate) fn render(&self) {
        print!("{}", ansi::CLEAR_SCREEN);
        for (i, ln) in self.buf.lines() {
            print!("{}{}", ansi::move_to(i, 0), ln);
        }
        print!("{}", ansi::move_to(self.cur_row, self.cur_col));
    }

    pub(crate) fn insert_char(&mut self, ch: char) {
        self.buf.insert_char(ch, self.cur_row, self.cur_col);
        self.cur_col += 1;
    }

    pub(crate) fn remove_char(&mut self) {
        if self.cur_col > 0 {
            self.buf.remove_char(self.cur_row, self.cur_col);
            self.cur_col = self.cur_col.saturating_sub(1);
        } else if self.cur_row > 0 {
            // col == 0, row > 0
            // lines will be joined, save current row column count
            let join_line_cols = self.buf.cols_in_row(self.cur_row);
            self.buf.remove_char(self.cur_row, self.cur_col);

            self.cur_row = self.cur_row.saturating_sub(1);
            self.cur_col = self.buf.cols_in_row(self.cur_row) - join_line_cols;
        }
    }

    pub(crate) fn break_line(&mut self) {
        self.buf.break_line(self.cur_row, self.cur_col);
        self.cur_row += 1;
        self.cur_col = 0;
    }

    pub(crate) fn move_cursor_up(&mut self) {
        self.cur_row = self.cur_row.saturating_sub(1);
        self.cur_col = self.cur_col.min(self.buf.cols_in_row(self.cur_row))
    }

    pub(crate) fn move_cursor_down(&mut self) {
        self.cur_row = self.cur_row.saturating_add(1).min(self.buf.rows() - 1);

        self.cur_col = self.cur_col.min(self.buf.cols_in_row(self.cur_row))
    }

    pub(crate) fn move_cursor_left(&mut self) {
        self.cur_col = self.cur_col.saturating_sub(1);
    }

    pub(crate) fn move_cursor_right(&mut self) {
        self.cur_col = self
            .cur_col
            .saturating_add(1)
            .min(self.buf.cols_in_row(self.cur_row));
    }
}
