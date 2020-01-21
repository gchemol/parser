// imports

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*imports][imports:1]]

// imports:1 ends here

// base

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*base][base:1]]
pub use nom;
pub use nom::IResult;

// macros
pub use nom::do_parse;

// branch
pub use nom::branch::alt;

// multi
pub use nom::multi::count;
pub use nom::multi::many_m_n;
pub use nom::multi::{many0, many1, many_till};
pub use nom::multi::{separated_list, separated_nonempty_list};

pub use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};

// combinator
pub use nom::combinator::{map, map_opt, map_res};
pub use nom::combinator::{not, opt, peek};
// base:1 ends here

// complete or streaming

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*complete or streaming][complete or streaming:1]]
macro_rules! nom_use {
    ($input:ident) => {
        pub use nom::bytes::$input::{is_a, is_not};
        pub use nom::bytes::$input::{tag, tag_no_case};
        pub use nom::bytes::$input::{take, take_until};
        pub use nom::character::$input::one_of;
        pub use nom::character::$input::{alpha0, alpha1};
        pub use nom::character::$input::{alphanumeric0, alphanumeric1};
        pub use nom::character::$input::{digit0, digit1};
        pub use nom::character::$input::{multispace0, multispace1};
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
