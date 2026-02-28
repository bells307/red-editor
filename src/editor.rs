use crate::{ansi, buffer::Buffer};

pub(crate) struct Editor {
    cursor_row: usize,
    cursor_col: usize,
    buffer: Buffer,
}

impl Editor {
    pub(crate) fn new() -> Self {
        Self {
            cursor_row: 0,
            cursor_col: 0,
            buffer: Buffer::empty(),
        }
    }

    pub(crate) fn render(&self) {
        print!("{}", ansi::CLEAR_SCREEN);
        for (i, ln) in self.buffer.lines() {
            print!("{}{}", ansi::move_to(i, 0), ln);
        }
        print!("{}", ansi::move_to(self.cursor_row, self.cursor_col));
    }

    pub(crate) fn insert_char(&mut self, ch: char) {
        self.buffer
            .insert_char(ch, self.cursor_row, self.cursor_col);
        self.cursor_col += 1;
    }

    pub(crate) fn remove_char(&mut self) {
        if self.cursor_col > 0 {
            self.buffer.remove_char(self.cursor_row, self.cursor_col);
            self.cursor_col = self.cursor_col.saturating_sub(1);
        } else if self.cursor_row > 0 {
            // col == 0, row > 0
            // lines will be joined, save current row column count
            let join_line_cols = self.buffer.cols_in_row(self.cursor_row);
            self.buffer.remove_char(self.cursor_row, self.cursor_col);

            self.cursor_row = self.cursor_row.saturating_sub(1);
            self.cursor_col = self.buffer.cols_in_row(self.cursor_row) - join_line_cols;
        }
    }

    pub(crate) fn break_line(&mut self) {
        self.buffer.break_line(self.cursor_row, self.cursor_col);
        self.cursor_row += 1;
        self.cursor_col = 0;
    }

    pub(crate) fn move_cursor_up(&mut self) {
        self.cursor_row = self.cursor_row.saturating_sub(1);
        self.cursor_col = self
            .cursor_col
            .min(self.buffer.cols_in_row(self.cursor_row))
    }

    pub(crate) fn move_cursor_down(&mut self) {
        self.cursor_row = self
            .cursor_row
            .saturating_add(1)
            .min(self.buffer.rows() - 1);

        self.cursor_col = self
            .cursor_col
            .min(self.buffer.cols_in_row(self.cursor_row))
    }

    pub(crate) fn move_cursor_left(&mut self) {
        self.cursor_col = self.cursor_col.saturating_sub(1);
    }

    pub(crate) fn move_cursor_right(&mut self) {
        self.cursor_col = self
            .cursor_col
            .saturating_add(1)
            .min(self.buffer.cols_in_row(self.cursor_row));
    }
}
