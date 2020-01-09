// imports

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*imports][imports:1]]

// imports:1 ends here

// base

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*base][base:1]]
pub use nom;
pub use nom::multi::{many0, many1};
pub use nom::IResult;

// macros
pub use nom::do_parse;

// branch
pub use nom::branch::alt;
// base:1 ends here

// complete or streaming

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*complete or streaming][complete or streaming:1]]
macro_rules! nom_use {
    ($input:ident) => {
        pub use nom::bytes::$input::tag;
        pub use nom::bytes::$input::take_until;
        pub use nom::character::$input::{alpha0, alpha1};
        pub use nom::character::$input::{alphanumeric0, alphanumeric1};
        pub use nom::character::$input::{digit0, digit1};
        pub use nom::character::$input::{space0, space1};
        pub use nom::number::$input::double;
    };
}

pub mod complete {
    nom_use!(complete);
}

pub mod streaming {
    nom_use!(streaming);
}
// complete or streaming:1 ends here
