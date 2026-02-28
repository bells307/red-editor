use crate::ansi;

pub(crate) struct Buffer {
    lines: Vec<String>,
}

impl Buffer {
    pub(crate) fn empty() -> Self {
        Self {
            lines: vec![String::new()],
        }
    }

    pub(crate) fn insert_char(&mut self, ch: char, row: usize, col: usize) {
        let row = row.min(self.lines.len() - 1);
        let col = col.min(self.lines[row].len());
        self.lines[row].insert(col, ch);
    }

    pub(crate) fn remove_char(&mut self, row: usize, col: usize) {
        if col > 0 {
            (&mut self.lines[row]).remove(col - 1);
        } else if row > 0 {
            let mut s = String::new();
            std::mem::swap(&mut s, &mut self.lines[row]);
            self.lines[row - 1].push_str(&s);
            self.lines.remove(row);
        }
    }

    pub(crate) fn break_line(&mut self, row: usize, col: usize) {
        let right = self.lines[row][col..].to_string();
        self.lines[row].truncate(col);
        self.lines.insert(row + 1, right)
    }

    pub(crate) fn rows(&self) -> usize {
        self.lines.len()
    }

    pub(crate) fn cols_in_row(&self, row: usize) -> usize {
        self.lines[row].len()
    }

    pub(crate) fn lines(&self) -> impl Iterator<Item = (usize, &str)> {
        self.lines.iter().map(String::as_str).enumerate()
    }
}
