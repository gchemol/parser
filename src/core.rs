// [[file:../parser.note::*imports][imports:1]]

// imports:1 ends here

// [[file:../parser.note::422da9c6][422da9c6]]
pub use nom;
// pub use nom::IResult;

// parse result with verbose error
pub type IResult<I, O> = nom::IResult<I, O, nom::error::VerboseError<I>>;

// add error context
pub use nom::error::context;

// macros
pub use crate::call; //  used in do_parse! macro
pub use crate::do_parse;

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
// 422da9c6 ends here

// [[file:../parser.note::df9c1bf7][df9c1bf7]]
/// macro imported from nom 6
///
/// Used to wrap common expressions and function as macros.
///
/// ```
/// # #[macro_use] extern crate nom;
/// # use nom::IResult;
/// # fn main() {
///   fn take_wrapper(input: &[u8], i: u8) -> IResult<&[u8], &[u8]> { take!(input, i * 10) }
///
///   // will make a parser taking 20 bytes
///   named!(parser, call!(take_wrapper, 2));
/// # }
/// ```
///
#[macro_export]
macro_rules! call (
  ($i:expr, $fun:expr) => ( $fun( $i ) );
  ($i:expr, $fun:expr, $($args:expr),* ) => ( $fun( $i, $($args),* ) );
);

/// macro imported from nom 6
///
/// `do_parse` applies sub parsers in a sequence.
/// It can store intermediary results and make them available
/// for later parsers.
///
/// The input type `I` must implement `nom::InputLength`.
///
/// This combinator will count how much data is consumed by every child parser
/// and take it into account if there is not enough data.
///
/// ```
/// # #[macro_use] extern crate nom;
/// # use nom::{Err,Needed};
/// use nom::number::streaming::be_u8;
///
/// // this parser implements a common pattern in binary formats,
/// // the TAG-LENGTH-VALUE, where you first recognize a specific
/// // byte slice, then the next bytes indicate the length of
/// // the data, then you take that slice and return it
/// //
/// // here, we match the tag 42, take the length in the next byte
/// // and store it in `length`, then use `take!` with `length`
/// // to obtain the subslice that we store in `bytes`, then return
/// // `bytes`
/// // you can use other macro combinators inside do_parse (like the `tag`
/// // one here), or a function (like `be_u8` here), but you cannot use a
/// // module path (like `nom::be_u8`) there, because of limitations in macros
/// named!(tag_length_value,
///   do_parse!(
///     tag!( &[ 42u8 ][..] ) >>
///     length: be_u8         >>
///     bytes:  take!(length) >>
///     (bytes)
///   )
/// );
///
/// # fn main() {
/// let a: Vec<u8>        = vec!(42, 2, 3, 4, 5);
/// let result_a: Vec<u8> = vec!(3, 4);
/// let rest_a: Vec<u8>   = vec!(5);
/// assert_eq!(tag_length_value(&a[..]), Ok((&rest_a[..], &result_a[..])));
///
/// // here, the length is 5, but there are only 3 bytes afterwards (3, 4 and 5),
/// // so the parser will tell you that you need 7 bytes: one for the tag,
/// // one for the length, then 5 bytes
/// let b: Vec<u8>     = vec!(42, 5, 3, 4, 5);
/// assert_eq!(tag_length_value(&b[..]), Err(Err::Incomplete(Needed::new(2))));
/// # }
/// ```
///
/// the result is a tuple, so you can return multiple sub results, like
/// this:
/// `do_parse!(I->IResult<I,A> >> I->IResult<I,B> >> ... I->IResult<I,X> , ( O, P ) ) => I -> IResult<I, (O,P)>`
///
/// ```
/// # #[macro_use] extern crate nom;
/// use nom::number::streaming::be_u8;
/// named!(tag_length_value<(u8, &[u8])>,
///   do_parse!(
///     tag!( &[ 42u8 ][..] ) >>
///     length: be_u8         >>
///     bytes:  take!(length) >>
///     (length, bytes)
///   )
/// );
///
/// # fn main() {
/// # }
/// ```
///
#[macro_export]
macro_rules! do_parse (
  (__impl $i:expr, ( $($rest:expr),* )) => (
    std::result::Result::Ok(($i, ( $($rest),* )))
  );

  (__impl $i:expr, $field:ident : $submac:ident!( $($args:tt)* ) ) => (
    do_parse!(__impl $i, $submac!( $($args)* ))
  );

  (__impl $i:expr, $submac:ident!( $($args:tt)* ) ) => (
    nom_compile_error!("do_parse is missing the return value. A do_parse call must end
      with a return value between parenthesis, as follows:

      do_parse!(
        a: tag!(\"abcd\") >>
        b: tag!(\"efgh\") >>

        ( Value { a: a, b: b } )
    ");
  );

  (__impl $i:expr, $field:ident : $submac:ident!( $($args:tt)* ) ~ $($rest:tt)* ) => (
    nom_compile_error!("do_parse uses >> as separator, not ~");
  );
  (__impl $i:expr, $submac:ident!( $($args:tt)* ) ~ $($rest:tt)* ) => (
    nom_compile_error!("do_parse uses >> as separator, not ~");
  );
  (__impl $i:expr, $field:ident : $e:ident ~ $($rest:tt)*) => (
    do_parse!(__impl $i, $field: call!($e) ~ $($rest)*);
  );
  (__impl $i:expr, $e:ident ~ $($rest:tt)*) => (
    do_parse!(__impl $i, call!($e) ~ $($rest)*);
  );

  (__impl $i:expr, $e:ident >> $($rest:tt)*) => (
    do_parse!(__impl $i, call!($e) >> $($rest)*);
  );
  (__impl $i:expr, $submac:ident!( $($args:tt)* ) >> $($rest:tt)*) => (
    {
      use std::result::Result::*;

      let i_ = $i.clone();
      match $submac!(i_, $($args)*) {
        Err(e) => Err(e),
        Ok((i,_))     => {
          let i_ = i.clone();
          do_parse!(__impl i_, $($rest)*)
        },
      }
    }
  );

  (__impl $i:expr, $field:ident : $e:ident >> $($rest:tt)*) => (
    do_parse!(__impl $i, $field: call!($e) >> $($rest)*);
  );

  (__impl $i:expr, $field:ident : $submac:ident!( $($args:tt)* ) >> $($rest:tt)*) => (
    {
      use std::result::Result::*;

      let i_ = $i.clone();
      match  $submac!(i_, $($args)*) {
        Err(e) => Err(e),
        Ok((i,o))     => {
          let $field = o;
          let i_ = i.clone();
          do_parse!(__impl i_, $($rest)*)
        },
      }
    }
  );

  // ending the chain
  (__impl $i:expr, $e:ident >> ( $($rest:tt)* )) => (
    do_parse!(__impl $i, call!($e) >> ( $($rest)* ));
  );

  (__impl $i:expr, $submac:ident!( $($args:tt)* ) >> ( $($rest:tt)* )) => ({
    use std::result::Result::*;

    match $submac!($i, $($args)*) {
      Err(e) => Err(e),
      Ok((i,_))     => {
        do_parse!(__finalize i, $($rest)*)
      },
    }
  });

  (__impl $i:expr, $field:ident : $e:ident >> ( $($rest:tt)* )) => (
    do_parse!(__impl $i, $field: call!($e) >> ( $($rest)* ) );
  );

  (__impl $i:expr, $field:ident : $submac:ident!( $($args:tt)* ) >> ( $($rest:tt)* )) => ({
    use std::result::Result::*;

    match $submac!($i, $($args)*) {
      Err(e) => Err(e),
      Ok((i,o))     => {
        let $field = o;
        do_parse!(__finalize i, $($rest)*)
      },
    }
  });

  (__finalize $i:expr, ( $o: expr )) => ({
    use std::result::Result::Ok;
    Ok(($i, $o))
  });

  (__finalize $i:expr, ( $($rest:tt)* )) => ({
    use std::result::Result::Ok;
    Ok(($i, ( $($rest)* )))
  });

  ($i:expr, $($rest:tt)*) => (
    {
      do_parse!(__impl $i, $($rest)*)
    }
  );
  ($submac:ident!( $($args:tt)* ) >> $($rest:tt)* ) => (
    nom_compile_error!("if you are using do_parse outside of a named! macro, you must
        pass the input data as first argument, like this:

        let res = do_parse!(input,
          a: tag!(\"abcd\") >>
          b: tag!(\"efgh\") >>
          ( Value { a: a, b: b } )
        );");
  );
  ($e:ident! >> $($rest:tt)* ) => (
    do_parse!( call!($e) >> $($rest)*);
  );
);
// df9c1bf7 ends here

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
