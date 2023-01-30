// [[file:../parser.note::*docs][docs:1]]
//! Split large text stream into multiple parts.
//!
//! # Example
//!
//! ```
//! use gchemol_parser::TextReader;
//! use gchemol_parser::partition::*;
//! 
//! let txt = "part1> line 1
//! part1> line 2 tail
//! part2> line 3
//! part2> line 5 tail
//! part3> line 8 tail ";
//! 
//! // instruct TextReader how to split the text stream into multiple parts.
//! struct PartX;
//! impl ReadPart for PartX {
//!     fn read_next(&self, context: ReadContext) -> ReadAction {
//!         let n = context.number_of_lines();
//!         // check the last line
//!         if context.line(n).ends_with("tail\n") {
//!             // make a new part terminated with this line
//!             ReadAction::Done(n)
//!         } else {
//!             // continue to read next line until find another tail line
//!             ReadAction::Need(1)
//!         }
//!     }
//! }
//! 
//! let reader = TextReader::from_str(txt);
//! let parts = reader.partitions(PartX);
//! assert_eq!(parts.count(), 3);
//! ```
// docs:1 ends here

// [[file:../parser.note::*imports][imports:1]]
use std::io::prelude::*;

use crate::reader::TextReader;
use gut::prelude::*;
// imports:1 ends here

// [[file:../parser.note::de2a5565][de2a5565]]
/// A helper struct for handling buffered text.
pub struct ReadContext<'a> {
    /// Buffered text.
    chunk: &'a str,
    /// byte size for each line of text.
    nlist: &'a [usize],
}

impl<'a> ReadContext<'a> {
    pub(crate) fn new(buf: &'a str, nlist: &'a [usize]) -> Self {
        Self { chunk: buf, nlist }
    }

    /// Return the number of lines that already read in.
    #[inline]
    pub fn number_of_lines(&self) -> usize {
        self.nlist.len()
    }

    /// Return the line numbered as `n` (1-based)
    ///
    /// # Panic
    ///
    /// * Panics if index `n` out of bounds.
    pub fn line(&self, n: usize) -> &str {
        assert_ne!(n, 0, "invalid line number");
        let n = n - 1;
        let ns = self.nlist[0..n].iter().sum();
        let nb = self.nlist[n];
        &self.chunk[ns..ns + nb]
    }

    /// Return buffered text.
    pub fn text(&self) -> &str {
        &self.chunk
    }
}

#[test]
fn test_read_context() {
    let txt = "line1\nLine2\nline 3\n";
    let nlist: Vec<_> = txt.lines().map(|l| l.len() + 1).collect();
    let context = ReadContext::new(txt, &nlist);

    assert_eq!(context.number_of_lines(), 3);
    assert_eq!(context.line(1), "line1\n");
    assert_eq!(context.line(2), "Line2\n");
    assert_eq!(context.line(3), "line 3\n");
}

/// Read text stream at line basis
pub enum ReadAction {
    /// Need next n lines to decide
    Need(usize),
    /// Read part done, return the first n lines as a part
    Done(usize),
    /// Error description
    Error(String),
}

/// Instruct the reader how to read a part of text by inspecting `ReadContext`
pub trait ReadPart {
    /// How to read next lines?
    fn read_next<'a>(&self, context: ReadContext<'a>) -> ReadAction {
        ReadAction::Need(1)
    }

    /// Read `n` lines at each time.
    fn n_stride(&self) -> usize {
        1
    }
}
// de2a5565 ends here

// [[file:../parser.note::11167470][11167470]]
/// An iterator over part of text stream.
pub struct Partitions<R, P>
where
    R: BufRead,
{
    reader: TextReader<R>,
    part: P,
    buf: String,
    nlist: Vec<usize>,
}

impl<R: BufRead, P> Partitions<R, P> {
    fn new(reader: TextReader<R>, part: P) -> Self {
        Self {
            reader,
            part,
            buf: String::new(),
            nlist: vec![],
        }
    }

    /// Read in `n` lines into `buf`. Return the number of bytes read in total.
    fn read_n_lines(&mut self, n: usize) -> Option<usize> {
        assert_ne!(n, 0);
        for _ in 0..n {
            let m = self.reader.read_line(&mut self.buf)?;
            self.nlist.push(m);
        }
        return Some(self.nlist.iter().sum());
    }

    /// build ReadContext for client.
    fn context(&self) -> ReadContext {
        ReadContext::new(&self.buf, &self.nlist)
    }
}

impl<R: BufRead, P: ReadPart> Iterator for Partitions<R, P> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        // read next n lines
        let mut m = self.part.n_stride();
        loop {
            // let _ = self.read_n_lines(m)?;
            // process the last part when reaching EOF
            if self.read_n_lines(m).is_none() {
                if self.buf.is_empty() {
                    break None;
                } else {
                    break Some(self.buf.drain(..).collect());
                }
            }
            // read in enough number of lines?
            match self.part.read_next(self.context()) {
                ReadAction::Need(n) => m = n,
                ReadAction::Done(n) => {
                    // take the first `n` lines as a part
                    let ns = self.nlist.drain(0..n).sum();
                    break Some(self.buf.drain(0..ns).collect());
                }
                ReadAction::Error(s) => {
                    error!("partition failure: {}", s);
                    break None;
                }
                _ => unreachable!(),
            }
        }
    }
}

impl<R: BufRead> TextReader<R> {
    /// Returns an iterator over part of text, using a generic text partioner
    /// `p`.
    // #[deprecated(note = "please use GrepReader instead")]
    pub fn partitions<P>(self, p: P) -> Partitions<R, P>
    where
        P: ReadPart,
    {
        Partitions::new(self, p)
    }
}
// 11167470 ends here

// [[file:../parser.note::9f67096e][9f67096e]]
/// Read in `n` lines at each time.
pub struct Chunks(usize);

impl ReadPart for Chunks {
    fn read_next(&self, context: ReadContext) -> ReadAction {
        let n = context.number_of_lines();
        if n == self.0 {
            ReadAction::Done(n)
        } else {
            unreachable!()
        }
    }

    fn n_stride(&self) -> usize {
        self.0
    }
}

impl<R: BufRead> TextReader<R> {
    /// Returns an iterator over each part of text in `n` lines.
    // #[deprecated(note = "please use GrepReader instead")]
    pub fn chunks(self, n: usize) -> Partitions<R, Chunks> {
        Partitions::new(self, Chunks(n))
    }
}
// 9f67096e ends here

// [[file:../parser.note::f96ed947][f96ed947]]
/// Terminated with a tail line
pub struct Terminated<F>(pub F);

impl<F> ReadPart for Terminated<F>
where
    F: Fn(&str) -> bool,
{
    #[inline]
    fn read_next(&self, context: ReadContext) -> ReadAction {
        let n = context.number_of_lines();
        let last_line = context.line(n);
        if (self.0)(last_line) {
            ReadAction::Done(n)
        } else {
            ReadAction::Need(1)
        }
    }
}

impl<R: BufRead> TextReader<R> {
    /// Returns an iterator over a part of text terminated with a tail line.
    // #[deprecated(note = "please use GrepReader instead")]
    pub fn partitions_terminated<F>(self, f: F) -> Partitions<R, Terminated<F>>
    where
        F: Fn(&str) -> bool,
    {
        Partitions::new(self, Terminated(f))
    }
}

#[test]
fn test_terminated() -> Result<()> {
    let txt = ": part1> line 1
: part1> line 2 tail
: part2> line 3
: part2> line 4
: part2> line 5 tail
: part3> line 6
: part3> line 7
: part3> line 8 tail ";

    let reader = TextReader::from_str(txt);
    let parts: Vec<_> = reader.partitions_terminated(|line| line.ends_with("tail\n")).collect();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0].lines().count(), 2);
    assert_eq!(parts[1].lines().count(), 3);
    assert_eq!(parts[2].lines().count(), 3);

    Ok(())
}
// f96ed947 ends here

// [[file:../parser.note::2b9d1c8d][2b9d1c8d]]
/// Preceded with a head line
pub struct Preceded<F>(pub F)
where
    F: Fn(&str) -> bool;

impl<F> ReadPart for Preceded<F>
where
    F: Fn(&str) -> bool,
{
    #[inline]
    fn read_next(&self, context: ReadContext) -> ReadAction {
        let n = context.number_of_lines();
        // n > 1: need at least two lines to decide
        if n > 1 && (self.0)(context.line(n)) {
            ReadAction::Done(n - 1)
        } else {
            ReadAction::Need(1)
        }
    }
}

impl<R: BufRead> TextReader<R> {
    /// Returns an iterator over a part of text preceded with a head line.
    // #[deprecated(note = "please use GrepReader instead")]
    pub fn partitions_preceded<F>(self, f: F) -> Partitions<R, Preceded<F>>
    where
        F: Fn(&str) -> bool,
    {
        Partitions::new(self, Preceded(f))
    }
}

#[test]
fn test_preceded() -> Result<()> {
    let txt = ": part1> line 1 head
: part1> line 2
: part2> line 3 head
: part2> line 4
: part2> line 5
: part3> line 6 head
: part3> line 7
: part3> line 8 ";

    let reader = TextReader::from_str(txt);
    let parts: Vec<_> = reader.partitions_preceded(|line| line.ends_with("head\n")).collect();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0].lines().count(), 2);
    assert_eq!(parts[1].lines().count(), 3);
    assert_eq!(parts[2].lines().count(), 3);

    Ok(())
}
// 2b9d1c8d ends here

// [[file:../parser.note::1970f69f][1970f69f]]
#[cfg(test)]
mod test {
    use super::*;

    struct XyzFile;
    impl ReadPart for XyzFile {
        fn read_next(&self, context: ReadContext) -> ReadAction {
            let n = context.number_of_lines();
            // the first line contains the number of atoms in this part
            if let Ok(natoms) = context.line(1).trim().parse::<usize>() {
                if n >= natoms + 2 {
                    ReadAction::Done(n)
                } else {
                    ReadAction::Need(natoms + 2 - n)
                }
            } else {
                ReadAction::Error("invalid xyz format".into())
            }
        }
    }

    #[test]
    fn test_adhoc() -> Result<()> {
        let f = "./tests/files/lammps-test.dump";
        let reader = TextReader::try_from_path(f.as_ref())?;

        // preceded parts
        let parts = reader.partitions_preceded(|line| line.starts_with("ITEM: TIMESTEP"));
        assert_eq!(parts.count(), 3, "preceded");

        // terminated parts
        let f = "./tests/files/multi.xyz";
        let reader = TextReader::try_from_path(f.as_ref())?;
        let parts: Vec<_> = reader.partitions(XyzFile).collect();
        assert_eq!(parts.len(), 6);
        assert_eq!(parts[0].lines().count(), 18);
        assert_eq!(parts[1].lines().count(), 12);
        assert_eq!(parts[2].lines().count(), 18);
        assert_eq!(parts[3].lines().count(), 18);
        assert_eq!(parts[4].lines().count(), 18);
        assert_eq!(parts[5].lines().count(), 15);

        // read chunks in constant number of lines
        let f = "./tests/files/multi.xyz";
        let reader = TextReader::try_from_path(f.as_ref())?;
        assert_eq!(reader.chunks(1).count(), 99, "chunks");
        let reader = TextReader::try_from_path(f.as_ref())?;
        let chunks = reader.chunks(5);
        let nn: Vec<_> = chunks.map(|x| x.lines().count()).collect();
        assert_eq!(nn.len(), 20, "chunks");
        assert_eq!(nn[0], 5, "chunks");
        assert_eq!(nn[19], 4, "chunks");

        Ok(())
    }

    // test default impl
    #[test]
    fn test_read_part_default() -> Result<()> {
        struct OnePart;
        impl ReadPart for OnePart {}

        let f = "./tests/files/multi.xyz";
        let reader = TextReader::try_from_path(f.as_ref())?;
        let parts = reader.partitions(OnePart);
        assert_eq!(parts.count(), 1);

        Ok(())
    }
}
// 1970f69f ends here
