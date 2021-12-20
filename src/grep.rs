// [[file:../parser.note::ea2eb3e9][ea2eb3e9]]
use gut::prelude::*;
use std::path::{Path, PathBuf};

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
// ea2eb3e9 ends here

// [[file:../parser.note::*mods][mods:1]]
mod internal;
// mods:1 ends here

// [[file:../parser.note::0a0d90f1][0a0d90f1]]
use self::internal::build_matcher_for_literals;
use self::internal::PartSink;
use self::internal::make_searcher;

use std::io::SeekFrom;

/// Quick grep text by marking the line that matching a pattern
#[derive(Debug)]
pub struct GrepReader {
    // A BufReader for File
    reader: BufReader<File>,
    // marked positions
    position_markers: Vec<u64>,
    // current position
    marker_index: usize,
}

impl GrepReader {
    /// Build from file in path
    pub fn try_from_path<P: AsRef<Path>>(p: P) -> Result<Self> {
        let f = File::open(p)?;
        let reader = BufReader::new(f);
        let grep = Self {
            reader,
            position_markers: vec![],
            marker_index: 0,
        };
        Ok(grep)
    }

    /// Mark positions that matching pattern, so that we can seek these
    /// positions later. Return the number of marked positions.
    pub fn mark<B: AsRef<str>>(&mut self, patterns: &[B]) -> Result<usize> {
        let mut n = 0;
        let mut marked = vec![];
        let matcher = build_matcher_for_literals(patterns)?;
        // grep does not know current position of the reader
        let pos_cur = self.reader.stream_position()?;
        make_searcher().search_reader(
            matcher,
            &mut self.reader,
            PartSink(|pos, _line| {
                marked.push(pos + pos_cur);
                n += 1;
                Ok(true)
            }),
        )?;
        // reset markers
        self.position_markers = marked;
        self.marker_index = 0;
        Ok(n)
    }

    /// Goto the start of the reader
    pub fn goto_start(&mut self) {
        self.reader.rewind();
    }

    /// Goto the end of the reader
    pub fn goto_end(&mut self) {
        self.reader.seek(SeekFrom::End(0));
    }

    /// Return the number of marked positions
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

    /// Read `n` lines into `buffer` on success. Return error if reached EOF early.
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
}
// 0a0d90f1 ends here

// [[file:../parser.note::*test][test:1]]
#[test]
fn test_grep() -> Result<()> {
    let path = "./tests/files/multi.xyz";
    let mut reader = GrepReader::try_from_path(path)?;
    let n = reader.mark(&[r"^\s*\d+\s*$"])?;
    assert_eq!(n, 6);

    let _ = reader.goto_next_marker()?;
    let _ = reader.goto_next_marker()?;
    let mut s = String::new();
    let _ = reader.read_lines(1, &mut s)?;
    assert_eq!(s.trim(), "10");

    // we can skip some lines before marking
    reader.goto_start();
    let mut s = String::new();
    reader.read_lines(1, &mut s)?;
    let n = reader.mark(&[r"^\s*\d+\s*$"])?;
    assert_eq!(n, 5);
    let _ = reader.goto_next_marker()?;
    s.clear();
    reader.read_lines(1, &mut s)?;
    assert_eq!(s.trim(), "10");

    // goto the marker directly
    let _ = reader.goto_marker(3)?;
    s.clear();
    reader.read_lines(1, &mut s)?;
    assert_eq!(s.trim(), "16");
    let _ = reader.goto_next_marker()?;
    s.clear();
    reader.read_lines(1, &mut s)?;
    assert_eq!(s.trim(), "13");

    Ok(())
}
// test:1 ends here
