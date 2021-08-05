// [[file:../parser.note::*imports][imports:1]]
use gut::prelude::*;
use std::path::{Path, PathBuf};
// imports:1 ends here

// [[file:../parser.note::*match][match:1]]
use grep::regex::RegexMatcher;

// Line oriented matches span no more than one line. The given pattern should
// not contain a literal \n.
fn make_matcher(pat: &str) -> Result<RegexMatcher> {
    let matcher = RegexMatcher::new_line_matcher(&pat)?;
    // let matcher = RegexMatcher::new(&pat)?;
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
pub struct TextGrep {
    reader: BufReader<File>,
    position_markers: Vec<u64>,
    // current position
    marker_index: usize,
}

impl TextGrep {
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
    pub fn mark(&mut self, pattern: &str) -> Result<usize> {
        let mut n = 0;
        let mut marked = vec![];
        make_searcher().search_reader(
            make_matcher(pattern)?,
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

    /// Goto the next position that marked. Return None if no marked positions
    /// or already reached the last marker.
    pub fn goto_next_marker(&mut self) -> Option<u64> {
        let n = self.position_markers.len();
        if self.marker_index < n {
            let pos = self.position_markers[self.marker_index];
            self.marker_index += 1;
            match self.reader.seek(SeekFrom::Start(pos)) {
                Ok(_) => Some(pos),
                Err(e) => {
                    dbg!(e);
                    return None;
                }
            }
        } else {
            None
        }
    }

    /// Read `n` lines into buffer in total, and return the buffer. Return error
    /// if can not read in enough number of lines before EOF.
    pub fn read_lines(&mut self, n: usize) -> Result<String> {
        let mut s = String::new();
        for i in 0..n {
            let nbytes = self.reader.read_line(&mut s)?;
            if nbytes == 0 {
                bail!("EOF reached. Expect {} lines, but read in {} lines", n, i);
            }
        }
        Ok(s)
    }
}

/// Do not count line number
fn make_searcher() -> Searcher {
    SearcherBuilder::new()
        // .line_number(false)
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .build()
}
// api:1 ends here

// [[file:../parser.note::*test][test:1]]
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

#[test]
fn test_grep() -> Result<()> {
    let path = "/home/ybyygu/Workspace/Programming/structure-predication/reaction-explore/data/1a/4d21ab-f77a-49c6-a6fe-9b0f1ae4e6c3/TS_dimer/freq/OUTCAR";

    let mut reader = TextGrep::try_from_path(path)?;
    let n = reader.mark(r" Electronic Relaxation|f/i= |NIONS =")?;
    println!("marked {} positions", n);

    dbg!(reader.read_lines(2));
    let n = reader.goto_next_marker();
    dbg!(n);
    dbg!(reader.read_lines(2));
    let n = reader.goto_next_marker();
    dbg!(n);
    dbg!(reader.read_lines(2));

    Ok(())
}
// test:1 ends here
