// mods

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*mods][mods:1]]
mod core;
mod reader;
mod parser;
// mods:1 ends here

// exports

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*exports][exports:1]]
pub mod parsers;
pub use crate::reader::*;

pub use crate::parser::TextParser;
// exports:1 ends here
