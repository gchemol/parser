// base

// [[file:~/Workspace/Programming/rust-libs/text-parser/text-parser.note::*base][base:1]]
use crate::combinators::*;

use std::io::{Read, BufRead, BufReader};

use nom;
use quicli::prelude::*;

type Result<T> = ::std::result::Result<T, Error>;
// base:1 ends here

// parse

// [[file:~/Workspace/Programming/rust-libs/text-parser/text-parser.note::*parse][parse:1]]
use std::str;

/// A stream parser for large text file
pub struct TextParser {
    /// The buffer size
    buffer_size: usize,
}

impl TextParser {
    /// Construct a text parser with buffer size `n`
    pub fn new(n: usize) -> Self {
        TextParser { buffer_size: n }
    }
}

/// General interface for parsing a large text file
impl Default for TextParser {
    fn default() -> Self {
        TextParser {
            buffer_size: 1 * 1024 * 1024 * 1024,
        }
    }
}

impl TextParser {
    /// Entry point for parsing a text file
    ///
    /// # Parameters
    /// - parser: nom parser
    /// - collector: a closure to collect parsed results
    pub fn parse<R: Read, F, C, P: Sized>(&self, f: R, parser: F, mut collector: C) -> Result<()>
    where
        F: Fn(&str) -> nom::IResult<&str, P>,
        C: FnMut(P),
    {
        // a. prepare data
        // let mut reader = BufReader::new(f);
        let mut reader = BufReader::with_capacity(self.buffer_size, f);
        let mut chunk = String::new();

        // b. process the read/parse loop
        // indicate if we finish reading
        let mut eof = false;
        'out: loop {
            // 0. fill chunk
            // if ! eof {
            //     for _ in 0..self.buffer_size {
            //         // reach EOF
            //         if reader.read_line(&mut chunk)? == 0 {
            //             eof = true;
            //             // a workaround for nom 4.0 changes: append a magic_eof line to make
            //             // stream `complete`
            //             chunk.push_str(MAGIC_EOF);
            //             break;
            //         }
            //     }
            // }

            // we can't have two `&mut` references to `stdin`, so use a block
            // to end the borrow early.
            let length = {
                let buffer = reader.fill_buf()?;
                let length = buffer.len();
                if length != 0 {
                    // fill chunk with new data
                    chunk.push_str(&str::from_utf8(&buffer)?);
                } else {
                    eof = true;
                    chunk.push_str(MAGIC_EOF);
                }

                length
            };
            // ensure the bytes we worked with aren't returned again later
            reader.consume(length);

            // 1. parse/consume the chunk until we get Incomplete error
            // remained: the unprocessed lines by parser
            let remained = String::new();
            let mut input = chunk.as_str();
            loop {
                match parser(input) {
                    // 1.1 success parsed one part
                    Ok((rest, part)) => {
                        // avoid infinite loop
                        debug_assert!(rest.len() < input.len());
                        // update the stream with the rest
                        input = rest;
                        // collect the parsed value
                        collector(part);
                        //println!("parse ok");
                    }

                    // 1.2 the chunk is incomplete.
                    // `Incomplete` means the nom parser does not have enough data to decide,
                    // so we wait for the next refill and then retry parsing
                    Err(nom::Err::Incomplete(_)) => {
                        // the chunk is unstained, so just break the parsing loop
                        //println!("parse incomplete");
                        break;
                    }

                    // 1.3 found parse errors, just ignore it and continue
                    Err(nom::Err::Error(err)) => {
                        if !eof {
                            eprintln!("found parsing error: {:?}", err);
                            eprintln!("the context lines: {}", input);
                        }
                        //break 'out;
                        break;
                    }

                    // 1.4 found serious errors
                    Err(nom::Err::Failure(err)) => {
                        bail!("encount hard failure: {:?}", err);
                    }

                    // 1.5 alerting nom changes
                    _ => {
                        bail!("found unrecovered nom state!");
                    }
                }
            }

            // all done, get out the loop
            if eof {
                if input.len() != 0 {
                    if input.trim() != MAGIC_EOF.trim() {
                        info!("remained data:\n {:}", input);
                    }
                }
                break;
            } else {
                // update chunk with remained data
                chunk = String::from(input);
            };
        }
        info!("parsing done.");

        // c. finish the job
        Ok(())
    }
}
// parse:1 ends here
