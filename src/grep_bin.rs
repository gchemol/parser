// [[file:../parser.note::*imports][imports:1]]
use super::*;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
// imports:1 ends here

// [[file:../parser.note::88a60571][88a60571]]
mod rg {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    struct GrepJsonOut {
        r#type: String,
        data: Data,
    }

    #[derive(Deserialize, Debug)]
    struct Data {
        absolute_offset: u64,
    }

    pub fn mark_matched_positions_with_ripgrep(pattern: &str, path: &Path) -> Result<Vec<u64>> {
        use gut::cli::duct::cmd;

        let json_out = cmd!("rg", "--no-line-number", "--json", pattern, path).read()?;

        let mut marked_positions = vec![];
        for line in json_out.lines() {
            if let Some(d) = serde_json::from_str::<GrepJsonOut>(line).ok() {
                marked_positions.push(d.data.absolute_offset);
            }
        }

        Ok(marked_positions)
    }

    #[test]
    fn test_json() {
        let marked = mark_matched_positions_with_ripgrep("^ITEM: NUMBER OF ATOMS", "./tests/files/lammps-test.dump".as_ref()).unwrap();
        assert_eq!(marked.len(), 3);
    }
}
// 88a60571 ends here

// [[file:../parser.note::b3c30bcf][b3c30bcf]]
use std::io::SeekFrom;
use std::path::{Path, PathBuf};

/// Quick grep text by marking the line that matching a pattern,
/// suitable for very large text file.
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
    pub fn mark(&mut self, pattern: &str) -> Result<usize> {
        use self::rg::mark_matched_positions_with_ripgrep;

        self.position_markers = mark_matched_positions_with_ripgrep(pattern, &self.src)?;
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
}
// b3c30bcf ends here

// [[file:../parser.note::3da52855][3da52855]]
#[test]
fn test_grep() -> Result<()> {
    let path = "./tests/files/multi.xyz";
    let mut reader = GrepReader::try_from_path(path.as_ref())?;
    let n = reader.mark(r"^\s*\d+\s*$")?;
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
// 3da52855 ends here
