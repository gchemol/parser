// lib.rs
// :PROPERTIES:
// :header-args: :tangle src/lib.rs
// :END:

// [[file:~/Workspace/Programming/rust-libs/text-parser/text-parser.note::*lib.rs][lib.rs:1]]
#[macro_use] extern crate nom;

#[macro_use]
mod nom_parser;
mod parser;

pub use self::nom_parser::*;
pub use self::parser::*;
// lib.rs:1 ends here
