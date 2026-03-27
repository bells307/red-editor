/// In-memory text buffer stored as a list of lines.
///
/// The buffer always contains at least one line, even when empty.
pub struct Buffer {
    /// The individual lines of text. Never empty.
    lines: Vec<String>,
}

impl Buffer {
    /// Creates an empty buffer containing a single empty line.
    pub fn scratch() -> Self {
        Self {
            lines: vec![String::new()],
        }
    }

    /// Creates a buffer from an existing list of lines.
    ///
    /// If `lines` is empty, falls back to [`Buffer::scratch`].
    pub fn from_lines(lines: Vec<String>) -> Self {
        if lines.is_empty() {
            Self::scratch()
        } else {
            Self { lines }
        }
    }

    /// Inserts `ch` at the given `(row, col)` position.
    ///
    /// Both `row` and `col` are clamped to valid bounds, so out-of-range
    /// values are silently saturated rather than panicking.
    pub fn insert_char(&mut self, ch: char, row: usize, col: usize) {
        let row = row.min(self.lines.len() - 1);
        let col = col.min(self.lines[row].len());
        self.lines[row].insert(col, ch);
    }

    /// Removes the character immediately before `col` on `row` (backspace semantics).
    ///
    /// If `col` is 0 and `row` is greater than 0, `row` is joined with the
    /// previous line and the row is removed.
    pub fn remove_char(&mut self, row: usize, col: usize) {
        if col > 0 {
            (&mut self.lines[row]).remove(col - 1);
        } else if row > 0 {
            let mut s = String::new();
            std::mem::swap(&mut s, &mut self.lines[row]);
            self.lines[row - 1].push_str(&s);
            self.lines.remove(row);
        }
    }

    /// Splits line `row` at `col`, inserting the right half as a new line below.
    pub fn break_line(&mut self, row: usize, col: usize) {
        let right = self.lines[row][col..].to_string();
        self.lines[row].truncate(col);
        self.lines.insert(row + 1, right)
    }

    /// Returns the number of lines in the buffer.
    pub fn rows(&self) -> usize {
        self.lines.len()
    }

    /// Returns the byte length of line `row`.
    pub fn cols_in_row(&self, row: usize) -> usize {
        self.lines[row].len()
    }

    /// Returns an iterator over `(row_index, line_text)` pairs.
    pub fn lines(&self) -> impl Iterator<Item = (usize, &str)> {
        self.lines.iter().map(String::as_str).enumerate()
    }
}
