// [[file:../parser.note::*imports][imports:1]]
use super::*;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
// imports:1 ends here

// [[file:../parser.note::480b544e][480b544e]]
mod grep_lib;
mod grep_bin;
// 480b544e ends here

// [[file:../parser.note::b3c30bcf][b3c30bcf]]
use crate::view::TextViewer;

use std::io::SeekFrom;
use std::path::{Path, PathBuf};

/// Quick grep text by marking the line that matching a pattern,
/// suitable for very large text file. If external rg command exists
/// in system, it will be called preferably for better performance.
#[derive(Debug)]
pub struct GrepReader {
    src: PathBuf,
    // A BufReader for File
    reader: BufReader<File>,
    // marked positions
    position_markers: Vec<u64>,
    // current position
    marker_index: usize,
}

impl GrepReader {
    /// Build from file in path
    pub fn try_from_path(p: &Path) -> Result<Self> {
        let f = File::open(p)?;
        let reader = BufReader::new(f);
        let grep = Self {
            reader,
            src: p.to_owned(),
            position_markers: vec![],
            marker_index: 0,
        };
        Ok(grep)
    }

    /// Mark positions that matching `pattern`, so that we can seek
    /// these positions later. Regex can be used in `pattern`. Return
    /// the number of marked positions.
    ///
    /// # Paramters
    /// * max_count: exits search if max_count matches reached.
    pub fn mark(&mut self, pattern: &str, max_count: impl Into<Option<usize>>) -> Result<usize> {
        let max_count = max_count.into();
        if let Ok(marked) = self::grep_bin::mark_matched_positions_with_ripgrep(pattern, &self.src, max_count) {
            self.position_markers = marked;
        } else {
            debug!("rg bin is not available. Mark with grep lib ...");
            use self::grep_lib::mark_matched_positions_with_ripgrep;
            self.position_markers = mark_matched_positions_with_ripgrep(pattern, &self.src, max_count)?;
        }
        // self.position_markers = self::grep_lib::mark_matched_positions_with_ripgrep(pattern, &self.src, max_count)?;

        self.marker_index = 0;
        Ok(self.position_markers.len())
    }

    /// Goto the start of inner file.
    pub fn goto_start(&mut self) {
        self.reader.rewind();
    }

    /// Goto the end of inner file.
    pub fn goto_end(&mut self) {
        self.reader.seek(SeekFrom::End(0));
    }

    /// Return the number of marked positions.
    pub fn num_markers(&self) -> usize {
        self.position_markers.len()
    }

    /// Goto the next position that marked. Return marker position on success.
    /// Return Err if already reached the last marker or other errors.
    pub fn goto_next_marker(&mut self) -> Result<u64> {
        let n = self.position_markers.len();
        if self.marker_index < n {
            let pos = self.position_markers[self.marker_index];
            self.marker_index += 1;
            let _ = self.reader.seek(SeekFrom::Start(pos))?;
            Ok(pos)
        } else {
            bail!("Already reached the last marker or no marker at all!");
        }
    }

    /// Goto the marked position in `marker_index`. Will panic if marker_index
    /// out of range.
    pub fn goto_marker(&mut self, marker_index: usize) -> Result<u64> {
        let pos = self.position_markers[marker_index];
        let _ = self.reader.seek(SeekFrom::Start(pos))?;
        self.marker_index = marker_index + 1;
        Ok(pos)
    }

    /// Return `n` lines in string on success from current
    /// position. Return error if reached EOF early.
    pub fn read_lines(&mut self, n: usize, buffer: &mut String) -> Result<()> {
        for i in 0..n {
            let nbytes = self.reader.read_line(buffer)?;
            if nbytes == 0 {
                bail!("The stream has reached EOF. Required {} lines, but filled {} lines", n, i);
            }
        }
        Ok(())
    }

    /// Gets a mutable reference to the underlying reader.
    pub fn get_mut(&mut self) -> &mut BufReader<File> {
        &mut self.reader
    }

    /// View next `n` lines like in a normal text viewer. This method
    /// will forward the cursor by `n` lines.
    pub fn view_lines(&mut self, n: usize) -> Result<TextViewer> {
        let mut s = String::new();
        self.read_lines(n, &mut s)?;
        let v = TextViewer::from_str(&s);
        Ok(v)
    }

    /// Return text from current position to the next marker or file
    /// end. It method will forward the cursor to the next marker.
    pub fn read_until_next_marker(&mut self, s: &mut String) -> Result<()> {
        let i = self.marker_index;

        // read until EOF?
        if i < self.position_markers.len() {
            let pos_cur = self.reader.stream_position()?;
            let pos_mark = self.position_markers[i];
            ensure!(pos_cur < pos_mark, "cursor is behind marker");
            let delta = pos_mark - pos_cur;
            let mut nsum = 0;
            for _ in 0.. {
                let n = self.reader.read_line(s)?;
                assert_ne!(n, 0);
                nsum += n as u64;
                if nsum >= delta {
                    break;
                }
            }
            self.marker_index += 1;
        } else {
            while self.reader.read_line(s)? != 0 {
                //
            }
        }
        Ok(())
    }

    /// View all lines until next marker like in a normal text viewer.
    /// It method will forward the cursor to the next marker.
    pub fn view_until_next_marker(&mut self) -> Result<TextViewer> {
        let mut s = String::new();
        self.read_until_next_marker(&mut s)?;
        Ok(TextViewer::from_str(&s))
    }
}
// b3c30bcf ends here

// [[file:../parser.note::3da52855][3da52855]]
#[test]
fn test_grep() -> Result<()> {
    let path = "./tests/files/multi.xyz";
    let mut reader = GrepReader::try_from_path(path.as_ref())?;
    let n = reader.mark(r"^\s*\d+\s*$", 2)?;
    assert_eq!(n, 2);
    let n = reader.mark(r"^\s*\d+\s*$", None)?;
    assert_eq!(n, 6);

    let _ = reader.goto_next_marker()?;
    let _ = reader.goto_next_marker()?;
    let mut s = String::new();
    let _ = reader.read_lines(1, &mut s)?;
    assert_eq!(s.trim(), "10");

    // goto the marker directly
    let _ = reader.goto_marker(4)?;
    s.clear();
    reader.read_lines(1, &mut s)?;
    assert_eq!(s.trim(), "16");
    let _ = reader.goto_next_marker()?;
    s.clear();
    reader.read_lines(1, &mut s)?;
    assert_eq!(s.trim(), "13");

    Ok(())
}

#[test]
fn test_grep_read_until() -> Result<()> {
    let path = "./tests/files/multi.xyz";
    // read until next marker
    let mut reader = GrepReader::try_from_path(path.as_ref())?;
    let n = reader.mark(r"^ Configuration number :", None)?;
    assert_eq!(n, 6);
    let mut s = String::new();
    reader.read_until_next_marker(&mut s)?;
    assert!(s.ends_with("          16\r\n"));
    s.clear();
    reader.goto_next_marker();
    reader.read_until_next_marker(&mut s)?;
    assert!(s.starts_with(" Configuration number :       14"));
    assert!(s.ends_with("          16\r\n"));
    assert_eq!(reader.marker_index, 3);
    reader.goto_marker(5);
    s.clear();
    reader.read_until_next_marker(&mut s)?;
    assert!(s.starts_with(" Configuration number :       42"));
    assert!(s.ends_with("0.97637  -1.60620\r\n"));

    Ok(())
}
// 3da52855 ends here
