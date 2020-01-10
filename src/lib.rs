// mods

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*mods][mods:1]]
mod core;
mod parser_;
mod reader;

pub(crate) mod common {
    pub use guts::prelude::*;
}

#[cfg(feature = "adhoc")]
pub mod parsers;
mod old;
// mods:1 ends here

// re-exports

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*re-exports][re-exports:1]]
pub use crate::core::*;

pub use crate::parser_::TextParser;

#[cfg(feature = "adhoc")]
pub use crate::reader::*;
// re-exports:1 ends here
