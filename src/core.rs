// [[file:../parser.note::*imports][imports:1]]

// imports:1 ends here

// [[file:../parser.note::*base][base:1]]
pub use nom;
// pub use nom::IResult;

// parse result with verbose error
pub type IResult<I, O> = nom::IResult<I, O, nom::error::VerboseError<I>>;

// add error context
pub use nom::error::context;

// macros
pub use nom::do_parse;

// branch
pub use nom::branch::alt;

// multi
pub use nom::multi::count;
pub use nom::multi::many_m_n;
pub use nom::multi::{many0, many1, many_till};
pub use nom::multi::{separated_list0, separated_list1};

pub use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};

// combinator
pub use nom::combinator::{map, map_opt, map_res};
pub use nom::combinator::{not, opt, peek};

// Some important traits
//
// we can call finish() method, and then propagate the errors normally with nice
// error messages
pub use nom::Finish;
// base:1 ends here

// [[file:../parser.note::*trace error][trace error:1]]
use gut::prelude::*;

/// Show nice parse trace on Error.
pub trait TraceNomError<I, O> {
    fn nom_trace_err(self) -> Result<(I, O)>;
}

impl<I: std::fmt::Display, O> TraceNomError<I, O> for IResult<I, O> {
    fn nom_trace_err(self) -> Result<(I, O)> {
        use nom::Finish;

        let r = self.finish().map_err(|e| format_err!("nom parsing failure:\n{}", e))?;
        Ok(r)
    }
}
// trace error:1 ends here

// [[file:../parser.note::*complete or streaming][complete or streaming:1]]
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
