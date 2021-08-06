// [[file:../parser.note::*imports][imports:1]]
use gut::prelude::*;
use std::path::{Path, PathBuf};

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
// imports:1 ends here

// [[file:../parser.note::*match][match:1]]
use grep::regex::RegexMatcher;

// Line oriented matches span no more than one line. The given pattern should
// not contain a literal \n.
fn make_matcher(pat: &str) -> Result<RegexMatcher> {
    let matcher = RegexMatcher::new_line_matcher(&pat)?;
    Ok(matcher)
}

// Build a new matcher from a plain alternation of literals, substantially
// faster than by joining the patterns with a | and calling build.
fn build_matcher_for_literals<B: AsRef<str>>(literals: &[B]) -> Result<RegexMatcher> {
    let matcher = grep::regex::RegexMatcherBuilder::new()
        .line_terminator(Some(b'\n'))
        .multi_line(true) // allow ^ matches the beginning of lines and $ matches the end of lines
        .build_literals(literals)?;

    Ok(matcher)
}
// match:1 ends here

// [[file:../parser.note::*sink][sink:1]]
use grep::searcher::{Sink, SinkError, SinkMatch};

/// The closure accepts two parameters: the absolute position of matched line
/// and a UTF-8 string containing the matched data. The closure returns a
/// `std::io::Result<bool>`. If the `bool` is `false`, then the search stops
/// immediately. Otherwise, searching continues.
#[derive(Clone, Debug)]
struct PartSink<F>(pub F)
where
    F: FnMut(u64, &str) -> std::io::Result<bool>;

impl<F> Sink for PartSink<F>
where
    F: FnMut(u64, &str) -> std::io::Result<bool>,
{
    type Error = std::io::Error;

    fn matched(&mut self, _searcher: &Searcher, mat: &SinkMatch<'_>) -> std::io::Result<bool> {
        let matched = match std::str::from_utf8(mat.bytes()) {
            Ok(matched) => matched,
            Err(err) => return Err(std::io::Error::error_message(err)),
        };
        // dbg!(mat.line_number());
        (self.0)(mat.absolute_byte_offset(), &matched)
    }
}
// sink:1 ends here

// [[file:../parser.note::*api][api:1]]
use grep::searcher::{BinaryDetection, Searcher, SearcherBuilder};
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
        make_searcher().search_reader(
            matcher,
            &mut self.reader,
            PartSink(|pos, matched| {
                marked.push(pos);
                n += 1;
                Ok(true)
            }),
        )?;
        self.position_markers = marked;
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
    /// Return None if already reached the last marker or other errors.
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

/// Do not count line number
fn make_searcher() -> Searcher {
    SearcherBuilder::new()
        .line_number(false)
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .build()
}
// api:1 ends here

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
    let _ = reader.get_mut().read_line(&mut s)?;
    assert_eq!(s.trim(), "10");

    Ok(())
}
// test:1 ends here
