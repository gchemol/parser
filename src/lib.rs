#![deny(missing_docs)]
//! Text Reader for large text file.
//!
//! # Example
//!
//! ```
//! use gchemol_parser::TextReader;
//! 
//! let mut reader = TextReader::try_from_path("./tests/files/ch3f.mol2".as_ref()).unwrap();
//! 
//! // read a line into `s`
//! let mut s = String::new();
//! reader.read_line(&mut s).unwrap();
//! 
//! // seek a specific line
//! let _ = reader.seek_line(|line| line.starts_with("@<TRIPOS>")).unwrap();
//! ```

// [[file:../parser.note::cbed1309][cbed1309]]
use gut::prelude::*;

use std::path::Path;
// cbed1309 ends here

// [[file:../parser.note::9b3ecbac][9b3ecbac]]
mod grep;
mod reader;
mod view;

mod common {
    pub use gut::prelude::*;
}
// 9b3ecbac ends here

// [[file:../parser.note::ff35c905][ff35c905]]
pub mod parsers;
// pub mod partition;
pub use crate::reader::*;

pub use crate::grep::GrepReader;
pub use crate::view::TextViewer;
// ff35c905 ends here
