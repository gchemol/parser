// [[file:../parser.note::03bd258c][03bd258c]]
use super::*;

use regex::RegexBuilder;
use ropey::Rope;
// 03bd258c ends here

// [[file:../parser.note::6c729559][6c729559]]
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use ropey::str_utils::{byte_to_line_idx, line_to_byte_idx};

/// A simple text view mode for quick peeking part of text
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
/// Constructors
impl TextViewer {
    /// Create a view of text string.
    pub fn from_str(txt: &str) -> Self {
        Self::new(txt.to_owned())
    }

    /// Create a view of file context in path `p`
    pub fn try_from_file(p: &Path) -> Result<Self> {
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

    /// Move the cursor to line `n`, counting from line 1 at beginning of the text.
    pub fn goto_line(&mut self, n: usize) {
        self.pos = self.line_pos(n);
    }

    /// Move the cursor to the line matching the pattern
    pub fn search_forward(&mut self, pattern: &str) -> Result<usize> {
        let re = RegexBuilder::new(pattern).multi_line(true).build().context("invalid regex")?;
        self.pos = re
            .find_at(&self.text, self.pos)
            .ok_or(format_err!("pattern not found: {}", pattern))?
            .start();
        Ok(self.pos)
    }

    pub fn peek_line(&self, n: usize) -> &str {
        let beg = self.line_pos(n);
        let end = self.line_pos(n + 1);
        &self.text[beg..end]
    }

    /// Peek the text between line `n` and `m` (including line `m`)
    pub fn peek_lines(&self, n: usize, m: usize) -> &str {
        let beg = self.line_pos(n);
        // including the line `m`
        let end = self.line_pos(m + 1);
        &self.text[beg..end]
    }
}
// 09977f99 ends here

// [[file:../parser.note::c6e19a12][c6e19a12]]
#[test]
fn test_view() -> Result<()> {
    let f = "./tests/files/lammps-test.dump";
    let mut view = TextView::try_from_file(f.as_ref())?;

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

    Ok(())
}
// c6e19a12 ends here
