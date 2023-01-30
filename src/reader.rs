// [[file:../parser.note::*imports][imports:1]]
use gut::fs::*;
use gut::prelude::*;

use std::io::Cursor;
// imports:1 ends here

// [[file:../parser.note::3f27d680][3f27d680]]
type FileReader = BufReader<File>;

fn text_file_reader<P: AsRef<Path>>(p: P) -> Result<FileReader> {
    let p = p.as_ref();
    debug!("Reader for file: {}", p.display());
    let f = File::open(p).with_context(|| format!("Failed to open file {:?}", p))?;

    let reader = BufReader::new(f);
    Ok(reader)
}

#[derive(Debug)]
/// A stream reader for large text file
pub struct TextReader<R> {
    inner: R,
}

impl TextReader<FileReader> {
    /// Build a text reader for file from path `p`.
    pub fn try_from_path(p: &Path) -> Result<Self> {
        let reader = text_file_reader(p)?;
        let parser = Self { inner: reader };
        Ok(parser)
    }
}

impl<'a> TextReader<Cursor<&'a str>> {
    /// Build a text reader for string slice.
    pub fn from_str(s: &'a str) -> Self {
        let r = Cursor::new(s);
        TextReader { inner: r }
    }
}

impl<R: Read> TextReader<BufReader<R>> {
    /// Build a text reader from a struct implementing Read trait.
    pub fn new(r: R) -> Self {
        Self { inner: BufReader::new(r) }
    }
}

impl<R: BufRead> TextReader<R> {
    /// Read a new line into buf.
    ///
    /// # NOTE
    /// - The new line is forced to use unix style line ending.
    /// - This function will return the total number of bytes read.
    /// - If this function returns None, the stream has reached EOF.
    pub fn read_line(&mut self, buf: &mut String) -> Option<usize> {
        match self.inner.read_line(buf) {
            Ok(0) => {
                return None;
            }
            Err(e) => {
                // discard any read in buf
                error!("Read line failure: {:?}", e);
                return None;
            }
            Ok(mut n) => {
                // force to use Unix line ending
                if buf.ends_with("\r\n") {
                    let i = buf.len() - 2;
                    buf.remove(i);
                    n -= 1;
                }
                return Some(n);
            }
        }
    }

    /// Returns an iterator over the lines of this reader. Each string returned
    /// will not have a line ending.
    pub fn lines(self) -> impl Iterator<Item = String> {
        // silently ignore UTF-8 error
        self.inner.lines().filter_map(|s| if let Ok(line) = s { Some(line) } else { None })
    }

    /// Read all text into string `buf` (Note: out of memory issue for large
    /// file)
    pub fn read_to_string(&mut self, buf: &mut String) -> Result<usize> {
        let n = self.inner.read_to_string(buf)?;
        Ok(n)
    }
}
// 3f27d680 ends here

// [[file:../parser.note::95fe0e8a][95fe0e8a]]
use std::io::SeekFrom;

impl<R: BufRead + Seek> TextReader<R> {
    /// Skip reading until finding a matched line. Return the position before
    /// the matched line. Return error if not found.
    pub fn seek_line<F>(&mut self, mut f: F) -> Result<u64>
    where
        F: FnMut(&str) -> bool,
    {
        let mut line = String::new();
        let mut m = 0u64;
        loop {
            let n = self.inner.read_line(&mut line)?;
            if n == 0 {
                // EOF
                bail!("no matched line found!");
            } else {
                // reverse the reading of the line
                if f(&line) {
                    let _ = self.inner.seek(std::io::SeekFrom::Current(-1 * n as i64))?;

                    return Ok(m);
                }
            }
            m += n as u64;
            line.clear();
        }

        Ok(m)
    }

    /// Goto the start of inner file.
    pub fn goto_start(&mut self) {
        self.inner.rewind();
    }

    /// Goto the end of inner file.
    pub fn goto_end(&mut self) {
        self.inner.seek(SeekFrom::End(0));
    }
}
// 95fe0e8a ends here

// [[file:../parser.note::b7e82299][b7e82299]]
#[test]
fn test_reader() -> Result<()> {
    // test lines
    let f = "./tests/files/multi.xyz";
    let reader = TextReader::try_from_path(f.as_ref())?;
    let line = reader.lines().skip(1).next().unwrap();
    assert_eq!(line, " Configuration number :        7");

    // test seeking
    let f = "./tests/files/ch3f.mol2";
    let mut reader = TextReader::try_from_path(f.as_ref())?;
    let _ = reader.seek_line(|line| line.starts_with("@<TRIPOS>"))?;
    let line = reader.lines().next().expect("ch3f test");
    assert_eq!(line, "@<TRIPOS>MOLECULE");

    // test from_str
    let s = "abc\nabcd\r\nabcde\n";
    let reader = TextReader::from_str(s);
    let line = reader.lines().next().unwrap();
    assert_eq!(line, "abc");

    Ok(())
}
// b7e82299 ends here
