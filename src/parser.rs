// imports

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*imports][imports:1]]
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use crate::common::*;
use crate::*;
// imports:1 ends here

// base

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*base][base:1]]
/// A stream parser for large text file
pub struct TextParser {
    /// The buffer size in number of lines
    buffer_size: usize,
}

impl TextParser {
    /// Construct a text parser with buffer size `n`
    pub fn new(n: usize) -> Self {
        TextParser {
            buffer_size: n,
            ..Default::default()
        }
    }
}

/// General interface for parsing a large text file
impl Default for TextParser {
    fn default() -> Self {
        TextParser { buffer_size: 1000 }
    }
}
// base:1 ends here

// core

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*core][core:1]]
impl TextParser {
    /// Entry point for parsing a text file
    ///
    /// # Parameters
    /// - parser: nom parser
    ///
    /// # Panics
    /// - panics if `file` doesnot exist.
    ///
    pub fn parse_file<R: AsRef<Path>, F, P>(&self, file: R, parser: F) -> impl Iterator<Item = P>
    where
        F: Fn(&str) -> nom::IResult<&str, P>,
    {
        let fp = std::fs::File::open(file.as_ref()).expect("a file to parse");
        self.parse(fp, parser)
    }

    /// Entry point for parsing from text stream
    ///
    /// # Parameters
    /// - parser: nom parser
    ///
    pub fn parse<R: Read, F, P>(&self, r: R, parser: F) -> impl Iterator<Item = P>
    where
        F: Fn(&str) -> nom::IResult<&str, P>,
    {
        let mut reader = BufReader::with_capacity(1024 * 1024 * 100, r);

        let mut i = 0;
        let mut eof = false;
        let mut remained = String::new();
        let nlines = self.buffer_size;
        std::iter::from_fn(move || {
            loop {
                // i += 1;
                // dbg!(i);
                // 1. parse/consume the chunk until we get Incomplete error
                for _ in 0..nlines {
                    match reader.read_line(&mut remained) {
                        Ok(n) if n == 0 => {
                            eof = true;
                            break;
                        }
                        Err(e) => {
                            eprintln!("Failed to read line: {:?}", e);
                            return None;
                        }
                        Ok(_) => {}
                    }
                }
                if eof {
                    remained.push_str(MAGIC_EOF);
                }

                let chunk = &remained;
                match parser(chunk) {
                    // 1.1 success parsed one part
                    Ok((rest, part)) => {
                        // dbg!(rest);
                        // avoid infinite loop
                        debug_assert!(rest.len() < chunk.len());

                        // update the chunk stream with the rest
                        remained = rest.to_owned();

                        // collect the parsed value
                        return Some(part);
                    }

                    // 1.2 the chunk is incomplete.
                    //
                    // `Incomplete` means the nom parser does not have enough
                    // data to decide, so we wait for the next refill and then
                    // retry parsing
                    Err(nom::Err::Incomplete(_)) => {
                        remained = chunk.to_owned();
                        if eof {
                            eprintln!("always incompelete???");
                            return None;
                        }
                    }

                    // 1.3 found parse errors, just ignore it and continue
                    Err(nom::Err::Error(err)) => {
                        if !eof {
                            eprintln!("found parsing error: {:?}", err);
                            eprintln!("the context lines: {}", chunk);
                        }
                        return None;
                    }

                    // 1.4 found serious errors
                    Err(nom::Err::Failure(err)) => {
                        eprintln!("found parser failure: {:?}", err);
                        return None;
                    }
                }
                if eof {
                    eprintln!("done");
                    return None;
                }
            }
        })
    }
}

/// Return an iterator over every n lines from `r`
fn read_chunk<R: Read>(r: R, nlines: usize) -> impl Iterator<Item = String> {
    let mut reader = BufReader::new(r);

    std::iter::from_fn(move || {
        let mut chunk = String::new();
        for _ in 0..nlines {
            match reader.read_line(&mut chunk) {
                Ok(n) if n == 0 => {
                    break;
                }
                Err(e) => {
                    eprintln!("Failed to read line: {:?}", e);
                    return None;
                }
                Ok(_) => {}
            }
        }

        if chunk.is_empty() {
            None
        } else {
            Some(chunk)
        }
    })
}
// core:1 ends here
