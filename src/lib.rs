//! Text Reader for large text file.
//!
//! # Example
//!
//! ```
//! use gchemol_parser::TextReader;
//! 
//! let mut reader = TextReader::from_path("./tests/files/ch3f.mol2").unwrap();
//! 
//! // read a line into `s`
//! let mut s = String::new();
//! reader.read_line(&mut s).unwrap();
//! 
//! // seek a specific line
//! let _ = reader.seek_line(|line| line.starts_with("@<TRIPOS>")).unwrap();
//! 
//! // split remaining text into chunks (each chunk has 5 lines)
//! let chunks = reader.chunks(5);
//! 
//! for x in chunks {
//!     // call nom parser to parse each chunk
//!     dbg!(x);
//! }
//! ```

// [[file:../parser.note::*mods][mods:1]]
mod core;
mod grep;
mod parser;
mod reader;
// mods:1 ends here

// [[file:../parser.note::*exports][exports:1]]
pub mod parsers;
pub mod partition;
pub use crate::reader::*;

pub use crate::grep::GrepReader;
pub use crate::parser::TextParser;
// exports:1 ends here
