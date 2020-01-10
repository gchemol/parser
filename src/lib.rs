// mods

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*mods][mods:1]]
mod core;
mod reader;

#[cfg(feature = "adhoc")]
pub mod parsers;
// mods:1 ends here

// re-exports

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*re-exports][re-exports:1]]
pub use crate::core::*;

#[cfg(feature = "adhoc")]
pub use crate::reader::*;
// re-exports:1 ends here
