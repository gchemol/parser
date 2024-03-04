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
    /// - This function will return the total number of bytes read.
    /// - If this function returns Ok(0), the stream has reached EOF.
    pub fn read_line(&mut self, buf: &mut String) -> Result<usize> {
        self.inner.read_line(buf).map_err(|e| anyhow!("Read line failure"))
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
    /// Peek next line without moving cursor.
    pub fn peek_line(&mut self) -> Option<String> {
        let mut buf = String::new();
        match self.inner.read_line(&mut buf) {
            Err(_) => None,
            Ok(0) => None,
            Ok(n) => {
                self.goto_relative(-1 * n as i64).expect("peek line go back");
                Some(buf)
            }
        }
    }

    /// Skip reading until finding a matched line. Return the number
    /// of bytes read in before the matched line. Return error if not
    /// found.
    pub fn seek_line<F>(&mut self, mut f: F) -> Result<usize>
    where
        F: FnMut(&str) -> bool,
    {
        let mut line = String::new();
        let mut m = 0;
        loop {
            let n = self.inner.read_line(&mut line)?;
            if n == 0 {
                // EOF
                bail!("no matched line found!");
            } else {
                // back to line start position
                if f(&line) {
                    let _ = self.goto_relative(-1 * n as i64)?;
                    return Ok(m);
                }
            }
            m += n;
            line.clear();
        }

        Ok(m)
    }

    /// Read lines into `buf` until `f` closure predicates true. Return
    /// total bytes read into `buf`.
    ///
    /// # NOTE
    /// - the line matching predicate is not included into `buf`
    pub fn read_until<F>(&mut self, buf: &mut String, mut f: F) -> Result<usize>
    where
        F: FnMut(&str) -> bool,
    {
        let mut m = 0;
        loop {
            let n = self.inner.read_line(buf)?;
            if n == 0 {
                // EOF
                bail!("no matched line found!");
            }
            let line = &buf[m..];
            if f(line) {
                self.goto_relative(-1 * n as i64)?;
                buf.drain(m..);
                return Ok(m);
            }
            m += n;
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

    /// Returns the current seek position from the start of the stream.
    pub fn get_current_position(&mut self) -> Result<u64> {
        let pos = self.inner.stream_position()?;
        Ok(pos)
    }

    /// Goto to an absolute position, in bytes, in a text stream.
    pub fn goto(&mut self, pos: u64) -> Result<()> {
        let pos = self.inner.seek(SeekFrom::Start(pos))?;
        Ok(())
    }

    /// Sets the offset to the current position plus the specified
    /// number of bytes. If the seek operation completed successfully,
    /// this method returns the new position from the start of the
    /// stream.
    pub fn goto_relative(&mut self, offset: i64) -> Result<u64> {
        let pos = self.inner.seek(SeekFrom::Current(offset))?;
        Ok(pos)
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

    // test read line until
    let s = "abc\nhere\r\nabcde\nhere\n\r";
    let mut reader = TextReader::from_str(s);
    let mut buf = String::new();
    let n = reader.read_until(&mut buf, |line| line.starts_with("here"))?;
    assert_eq!(buf, "abc\n");
    buf.clear();
    reader.read_line(&mut buf);
    assert_eq!(buf, "here\r\n");

    Ok(())
}
// b7e82299 ends here
