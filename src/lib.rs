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

// [[file:../parser.note::*imports][imports:1]]
use gut::prelude::*;
// imports:1 ends here

// [[file:../parser.note::9b3ecbac][9b3ecbac]]
mod core;
mod grep;
mod reader;
mod view;
// 9b3ecbac ends here

// [[file:../parser.note::ff35c905][ff35c905]]
pub mod parsers;
pub mod partition;
pub use crate::reader::*;

pub use crate::grep::GrepReader;
pub use crate::view::TextViewer;
// pub use crate::parser::TextParser;
// ff35c905 ends here
