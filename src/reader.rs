// imports

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*imports][imports:1]]
use guts::fs::*;
use guts::prelude::*;

use std::io::prelude::*;
use std::io::Cursor;
// imports:1 ends here

// reader

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*reader][reader:1]]
type FileReader = BufReader<File>;

#[derive(Debug)]
pub struct TextReader<R: BufRead> {
    reader: R,
}

impl TextReader<FileReader> {
    /// Build a text reader for file from path `p`.
    pub fn from_path<P: AsRef<Path>>(p: P) -> Result<Self> {
        let reader = text_file_reader(p)?;
        let parser = Self { reader };
        Ok(parser)
    }
}

impl<'a> TextReader<Cursor<&'a str>> {
    /// Build a text reader for string slice.
    pub fn from_str(s: &'a str) -> Self {
        let r = Cursor::new(s);
        TextReader { reader: r }
    }
}

impl<R: Read> TextReader<BufReader<R>> {
    /// Build a text reader from a struct implementing Read trait.
    pub fn new(r: R) -> Self {
        Self {
            reader: BufReader::new(r),
        }
    }
}

impl<R: BufRead + Seek> TextReader<R> {
    /// Skip reading until finding a matched line. Return the position before
    /// the matched line.
    pub fn seek_line<F>(&mut self, f: F) -> Result<u64>
    where
        F: Fn(&str) -> bool,
    {
        let mut line = String::new();
        let mut m = 0u64;
        loop {
            let n = self.reader.read_line(&mut line)?;
            // EOF
            if n == 0 {
                break;
            } else {
                // reverse the reading of the line
                if f(&line) {
                    // self.reader.seek(std::io::SeekFrom::Start(0));
                    // let mut s = vec![0; m];
                    // self.reader.read_exact(&mut s)?;
                    // return Ok(String::from_utf8(s).unwrap());
                    let _ = self.reader.seek(std::io::SeekFrom::Start(m))?;
                    return Ok(m);
                }
            }
            m += n as u64;
            line.clear();
        }

        Ok(m)
    }

    #[deprecated(note = "Use seek_line method instead.")]
    pub fn skip_until<F>(&mut self, f: F) -> Result<u64>
    where
        F: Fn(&str) -> bool,
    {
        self.seek_line(f)
    }

    /// Returns an iterator over the lines of this reader. Each string returned
    /// will not have a line ending.
    pub fn lines(self) -> impl Iterator<Item = String> {
        // silently ignore UTF-8 error
        self.reader
            .lines()
            .filter_map(|s| if let Ok(line) = s { Some(line) } else { None })
    }

    /// Read all text into string `buf`.
    pub fn read_to_string(&mut self, buf: &mut String) -> Result<usize> {
        let n = self.reader.read_to_string(buf)?;
        Ok(n)
    }
}

fn text_file_reader<P: AsRef<Path>>(p: P) -> Result<FileReader> {
    let p = p.as_ref();
    let f = File::open(p).with_context(|| format!("Failed to open file {:?}", p))?;

    let reader = BufReader::new(f);
    Ok(reader)
}
// reader:1 ends here

// impl/peeking

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*impl/peeking][impl/peeking:1]]
/// An iterator over partitioned lines of an instance of BufRead.
///
/// see also: TextReader.partitions method.
pub struct Partitions<R, P> {
    buf: R,
    partition: P,
    peeked: Option<(String, usize)>,
}

impl<R, P> Partitions<R, P> {
    fn new(buf: R, partition: P) -> Self {
        Self {
            buf,
            partition,
            peeked: None,
        }
    }
}

impl<R: BufRead, P: Partition> Iterator for Partitions<R, P> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chunk = String::new();
        let mut nlist = vec![];
        loop {
            let mut next_line = String::new();
            match self.buf.read_line(&mut next_line) {
                Ok(0) => {
                    if let Some((peeked_line, _peeked_n)) = &self.peeked {
                        chunk += peeked_line;
                        self.peeked = None;
                        return Some(chunk);
                    }
                    break;
                }
                Ok(n) => {
                    // not the first line
                    if let Some((peeked_line, peeked_n)) = &self.peeked {
                        chunk += peeked_line;
                        nlist.push(*peeked_n);
                        let context = ReadContext {
                            buf: &chunk,
                            nlist: &nlist,
                            peeked_line: &next_line,
                        };

                        self.peeked = Some((next_line.clone(), n));
                        if !self.partition.read_next(context) {
                            return Some(chunk);
                        }
                    } else {
                        // update peeked value
                        self.peeked = Some((next_line, n));
                    }
                }
                Err(e) => {
                    // discard any read in buf
                    error!("Read line failure: {:?}", e);
                    error!("current buf: {:?}", chunk);
                    return None;
                }
            }
        }
        // process final iteration
        None
    }
}

/// A helper struct for handling buffered text.
pub struct ReadContext<'a> {
    buf: &'a str,
    nlist: &'a [usize],
    peeked_line: &'a str,
}

impl<'a> ReadContext<'a> {
    /// Return the number of lines that alredy read in.
    #[inline]
    pub fn nlines(&self) -> usize {
        self.nlist.len()
    }

    /// Return the text that already read in.
    pub fn text(&self) -> &str {
        &self.buf
    }

    /// Return current line.
    pub fn this_line(&self) -> &str {
        let n = self.nlines();
        assert!(n > 0);
        let n = self.nlist[n - 1];
        let m = self.buf.len() - n;
        &self.buf[m..]
    }

    /// Return peeked next line.
    pub fn next_line(&self) -> &str {
        self.peeked_line
    }

    pub fn prev_line(&self) -> &str {
        todo!()
    }
}

/// Read next line or not
pub trait Partition {
    /// Instruct the reader to read in the next line or not.
    ///
    /// Always read in next line by default.
    #[inline]
    fn read_next(&self, _context: ReadContext) -> bool {
        true
    }
}

impl<R: BufRead> TextReader<R> {
    /// Returns an iterator over `n` lines at a time.
    pub fn partitions<P: Partition>(self, p: P) -> Partitions<R, P> {
        Partitions::new(self.reader, p)
    }
}
// impl/peeking:1 ends here

// chunks
// Read text in chunk of every n lines.

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*chunks][chunks:1]]
// read in nlines as a partition
pub struct NLines(usize);

impl Partition for NLines {
    fn read_next(&self, context: ReadContext) -> bool {
        context.nlines() < self.0
    }
}

impl<R: BufRead> TextReader<R> {
    /// Returns an iterator over `n` lines at a time.
    pub fn chunks(self, n: usize) -> Partitions<R, NLines> {
        self.partitions(NLines(n))
    }
}
// chunks:1 ends here

// terminated with

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*terminated with][terminated with:1]]
// Terminated with
pub struct Terminated<F>
where
    F: Fn(&str) -> bool,
{
    f: F,
}

impl<F> Partition for Terminated<F>
where
    F: Fn(&str) -> bool,
{
    #[inline]
    fn read_next(&self, context: ReadContext) -> bool {
        !(self.f)(context.this_line())
    }
}

impl<R: BufRead> TextReader<R> {
    /// Returns an iterator over `n` lines at a time.
    pub fn terminated_bunches<F>(self, f: F) -> Partitions<R, Terminated<F>>
    where
        F: Fn(&str) -> bool,
    {
        self.partitions(Terminated { f })
    }
}
// terminated with:1 ends here

// preceded with

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*preceded with][preceded with:1]]
// Preceded with
pub struct Preceded<F>
where
    F: Fn(&str) -> bool,
{
    f: F,
}

impl<F> Partition for Preceded<F>
where
    F: Fn(&str) -> bool,
{
    #[inline]
    fn read_next(&self, context: ReadContext) -> bool {
        !(self.f)(context.next_line())
    }
}

impl<R: BufRead> TextReader<R> {
    /// Returns an iterator over `n` lines at a time.
    pub fn preceded_bunches<F>(self, f: F) -> Partitions<R, Preceded<F>>
    where
        F: Fn(&str) -> bool,
    {
        self.partitions(Preceded { f })
    }
}
// preceded with:1 ends here

// impl

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*impl][impl:1]]
#[deprecated(note = "Plan to be removed")]
impl<R: BufRead> TextReader<R> {
    /// Return an iterator over a bunch of lines preceded by a label line.
    pub fn bunches<F>(self, label_fn: F) -> Bunches<F, R>
    where
        F: Fn(&str) -> bool,
    {
        Bunches::new(self.reader, label_fn)
    }
}

pub struct Bunches<F, R>
where
    F: Fn(&str) -> bool,
    R: BufRead,
{
    lines: std::iter::Peekable<std::io::Lines<R>>,
    is_data_label: F,
}

impl<F, R> Bunches<F, R>
where
    F: Fn(&str) -> bool,
    R: BufRead,
{
    fn new(reader: R, f: F) -> Self {
        Self {
            lines: reader.lines().peekable(),
            is_data_label: f,
        }
    }
}

impl<F, R> Iterator for Bunches<F, R>
where
    F: Fn(&str) -> bool,
    R: BufRead,
{
    type Item = Vec<String>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chunk = vec![];
        loop {
            match self.lines.next() {
                Some(Ok(line)) => {
                    chunk.push(line);
                    // check next line to decide if return
                    match self.lines.peek() {
                        Some(Ok(next_line)) => {
                            if (self.is_data_label)(next_line) {
                                return Some(chunk);
                            }
                        }
                        Some(Err(e)) => {
                            warn!("found reading error: {}", e);
                        }
                        None => {
                            // reach eof
                            return Some(chunk);
                        }
                    }
                }
                Some(Err(e)) => {
                    // ignore
                    warn!("found reading error: {}", e);
                }
                None => {
                    // reach eof
                    break;
                }
            }
        }
        None
    }
}
// impl:1 ends here

// test

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*test][test:1]]
#[test]
fn test_reader() -> Result<()> {
    let f = "./tests/files/lammps-test.dump";
    let reader = TextReader::from_path(f)?;
    let bunches = reader.preceded_bunches(|line| line.starts_with("ITEM: TIMESTEP"));
    assert_eq!(bunches.count(), 3);

    let f = "./tests/files/multi.xyz";
    let if_data_label = |line: &str| line.trim().parse::<usize>().is_ok();
    let reader = TextReader::from_path(f)?;
    let bunches = reader.preceded_bunches(if_data_label);
    assert_eq!(bunches.count(), 6);

    // test chunks
    let reader = TextReader::from_path(f)?;
    assert_eq!(reader.chunks(1).count(), 99);
    let reader = TextReader::from_path(f)?;
    let chunks = reader.chunks(5);
    let nn: Vec<_> = chunks.map(|x| x.lines().count()).collect();
    assert_eq!(nn.len(), 20);
    assert_eq!(nn[0], 5);
    assert_eq!(nn[19], 4);

    // test lines
    let reader = TextReader::from_path(f)?;
    let line = reader.lines().skip(1).next().unwrap();
    assert_eq!(line, " Configuration number :        7");

    // test seeking
    let f = "./tests/files/ch3f.mol2";
    let mut reader = TextReader::from_path(f)?;
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
// test:1 ends here
