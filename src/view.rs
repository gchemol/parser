// [[file:../parser.note::03bd258c][03bd258c]]
use super::*;
// 03bd258c ends here

// [[file:../parser.note::6c729559][6c729559]]
use ropey::str_utils::{byte_to_line_idx, char_to_byte_idx, line_to_byte_idx};

/// A simple line-based text viewer for quick peeking part of text
#[derive(Debug, Clone)]
pub struct TextViewer {
    text: String,
    pos: usize,
}

impl TextViewer {
    fn new(text: String) -> Self {
        Self { text, pos: 0 }
    }

    /// Return byte index from line number in string.
    fn line_pos(&self, line_num: usize) -> usize {
        assert!(line_num >= 1, "invalid line number: {}", line_num);
        line_to_byte_idx(&self.text, line_num - 1)
    }

    /// Return line number from byte index `pos`
    fn pos_to_line_num(&self, pos: usize) -> usize {
        byte_to_line_idx(&self.text, pos) + 1
    }
}
// 6c729559 ends here

// [[file:../parser.note::09977f99][09977f99]]
use regex::RegexBuilder;

/// Constructor a `TextViewer` like a text reader in read-only mode,
/// suitable for small file that can be fully read into memory.
impl TextViewer {
    /// Create a view of text string.
    pub fn from_str(txt: &str) -> Self {
        Self::new(txt.to_owned())
    }

    /// Create a view of file context in path `p`
    pub fn try_from_path(p: &Path) -> Result<Self> {
        let text = gut::fs::read_file(p)?;
        let view = Self::new(text);
        Ok(view)
    }
}

/// Core methods
impl TextViewer {
    /// Total number of lines
    pub fn num_lines(&self) -> usize {
        self.pos_to_line_num(self.text.len())
    }

    /// Get line number at cursor
    pub fn current_line_num(&self) -> usize {
        self.pos_to_line_num(self.pos)
    }

    /// Return str slice of inner text.
    pub fn text(&self) -> &str {
        self.text.as_str()
    }

    /// Peek the line at cursor
    pub fn current_line(&self) -> &str {
        self.peek_line(self.current_line_num())
    }

    /// Move the cursor to line `n`, counting from line 1 at beginning of the text.
    pub fn goto_line(&mut self, n: usize) {
        self.pos = self.line_pos(n);
    }

    /// Move the cursor to the beginning of the first line.
    pub fn goto_first_line(&mut self) {
        self.goto_line(1);
    }

    /// Move the cursor to the beginning of the last line.
    pub fn goto_last_line(&mut self) {
        self.goto_line(self.num_lines());
    }

    /// Move the cursor to the beginning of the next line.
    pub fn goto_next_line(&mut self) {
        self.goto_line(self.current_line_num() + 1);
    }

    /// Move the cursor to the beginning of the previous line.
    pub fn goto_previous_line(&mut self) {
        self.goto_line(self.current_line_num() - 1);
    }

    /// Move the cursor to the line matching `pattern`. Regex pattern
    /// is allowed. Return current line number after search.
    pub fn search_forward(&mut self, pattern: &str) -> Result<usize> {
        let re = RegexBuilder::new(pattern).multi_line(true).build().context("invalid regex")?;
        self.pos = re
            .find_at(&self.text, self.pos)
            .ok_or(format_err!("pattern not found: {}", pattern))?
            .start();
        Ok(self.current_line_num())
    }

    /// Search backward from current point for `pattern`. Return
    /// current line number after search.
    pub fn search_backward(&mut self, pattern: &str) -> Result<usize> {
        let n = self.current_line_num();
        let s = self.peek_lines(1, n);
        let re = RegexBuilder::new(pattern).multi_line(true).build().context("invalid regex")?;
        self.pos = re.find_iter(s).last().ok_or(format_err!("pattern not found: {}", pattern))?.start();
        Ok(self.current_line_num())
    }

    /// Peek line `n` without moving cursor.
    pub fn peek_line(&self, n: usize) -> &str {
        let beg = self.line_pos(n);
        let end = self.line_pos(n + 1);
        &self.text[beg..end]
    }

    /// Peek the text between line `n` and `m` (including line `m`),
    /// without moving cursor.
    pub fn peek_lines(&self, n: usize, m: usize) -> &str {
        let beg = self.line_pos(n);
        // including the line `m`
        let end = self.line_pos(m + 1);
        &self.text[beg..end]
    }

    /// Select the next `n` lines from current point, including current line.
    pub fn selection(&self, n: usize) -> &str {
        let m = self.current_line_num();
        self.peek_lines(m, m + n - 1)
    }

    /// Select part of the string in next `n` lines (including
    /// currrent line), in a rectangular area surrounded by columns in
    /// `col_beg`--`col_end`.
    pub fn column_selection(&self, n: usize, col_beg: usize, col_end: usize) -> String {
        assert!(col_beg <= col_end, "invalid column data: {:?}", (col_beg, col_end));
        let line_beg = self.current_line_num();
        let line_end = line_beg + n - 1;

        let lines = self.peek_lines(line_beg, line_end);
        let mut selection = vec![];
        for x in lines.lines() {
            let p1 = char_to_byte_idx(x, col_beg);
            let p2 = char_to_byte_idx(x, col_end);
            selection.push(&x[p1..p2]);
        }
        selection.join("\n")
    }
}
// 09977f99 ends here

// [[file:../parser.note::c6e19a12][c6e19a12]]
#[test]
fn test_view() -> Result<()> {
    let f = "./tests/files/lammps-test.dump";
    let mut view = TextViewer::try_from_path(f.as_ref())?;

    assert_eq!(view.num_lines(), 1639);
    view.goto_line(2);
    assert_eq!(view.current_line_num(), 2);

    assert_eq!(view.peek_line(1), "ITEM: TIMESTEP\n");
    assert_eq!(view.peek_lines(3, 5), "ITEM: NUMBER OF ATOMS\n537\nITEM: BOX BOUNDS pp pp pp\n");

    view.search_forward(r"TIMESTEP$")?;
    let n = view.current_line_num();
    assert_eq!(n, 547);
    view.search_forward(r"^ITEM: NU.*$")?;
    let n = view.current_line_num();
    assert_eq!(n, 549);

    // search back
    view.goto_last_line();
    let l = view.current_line();
    assert!(l.is_empty());
    view.search_backward("^523 ");
    let l = view.current_line();
    assert!(l.starts_with("523 1 5.00268"));
    assert_eq!(view.current_line_num(), 1624);

    Ok(())
}

#[test]
fn test_column_selection() -> Result<()> {
    let f = "./tests/files/multi.xyz";
    let mut view = TextViewer::try_from_path(f.as_ref())?;
    view.goto_line(3);
    let s = view.selection(2);
    assert_eq!(s.lines().count(), 2);
    let s = view.column_selection(3, 4, 100);
    assert_eq!(s.lines().count(), 3);
    let s = view.column_selection(3, 4, 24);
    assert_eq!(s.lines().next().unwrap().split_whitespace().count(), 2);
    println!("{}", s);

    Ok(())
}
// c6e19a12 ends here
