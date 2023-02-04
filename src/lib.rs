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
//! 
//! // split remaining text into chunks (each chunk has 5 lines)
//! let chunks = reader.chunks(5);
//! 
//! for x in chunks {
//!     // call nom parser to parse each chunk
//!     dbg!(x);
//! }
//! ```

// [[file:../parser.note::cbed1309][cbed1309]]
use gut::prelude::*;

use std::path::Path;
// cbed1309 ends here

// [[file:../parser.note::9b3ecbac][9b3ecbac]]
mod core;
mod grep;
mod reader;
mod view;
// 9b3ecbac ends here

// [[file:../parser.note::838e8dea][838e8dea]]
/// Convert a string to a float.
///
/// This method performs certain checks, that are specific to quantum
/// chemistry output, including avoiding the problem with Ds instead
/// of Es in scientific notation. Another point is converting string
/// signifying numerical problems (*****) to something we can manage
/// (NaN).
pub fn parse_fortran_float(s: &str) -> Option<f64> {
    if s.chars().all(|x| x == '*') {
        std::f64::NAN.into()
    } else {
        s.parse().ok().or_else(|| s.replacen("D", "E", 1).parse().ok())
    }
}

#[test]
fn test_fortran_float() {
    let x = parse_fortran_float("14");
    assert_eq!(x, Some(14.0));

    let x = parse_fortran_float("14.12E4");
    assert_eq!(x, Some(14.12E4));

    let x = parse_fortran_float("14.12D4");
    assert_eq!(x, Some(14.12E4));

    let x = parse_fortran_float("****");
    assert!(x.unwrap().is_nan());
}
// 838e8dea ends here

// [[file:../parser.note::ff35c905][ff35c905]]
pub mod parsers;
pub mod partition;
pub use crate::reader::*;

pub use crate::grep::GrepReader;
pub use crate::view::TextViewer;
// ff35c905 ends here
