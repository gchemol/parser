// [[file:../parser.note::*docs][docs:1]]
//! Selected and extra winnow parser combinators
// docs:1 ends here

// [[file:../parser.note::273abf0b][273abf0b]]
use crate::common::*;

use winnow::error::ParserError;
use winnow::error::StrContext;
use winnow::stream::Stream;
// 273abf0b ends here

// [[file:../parser.note::0512156a][0512156a]]
pub use winnow::ascii::{alpha0, alpha1, digit0, digit1, line_ending, space0, space1};
pub use winnow::combinator::seq;
pub use winnow::combinator::{delimited, preceded, repeat, separated, terminated};
pub use winnow::prelude::*;
pub use winnow::Parser;
// 0512156a ends here

// [[file:../parser.note::fb1326ab][fb1326ab]]
/// Create context labe
pub fn label(s: &'static str) -> StrContext {
    StrContext::Label(s)
}

/// Convert winnow error to anyhow Error
pub fn parse_error(e: winnow::error::ParseError<&str, winnow::error::ContextError>, input: &str) -> Error {
    anyhow!("found parse error:\n{:}\ninput={input:?}", e.to_string())
}

/// Anything except whitespace, this parser will not consume "\n" character
pub fn not_space<'a>(input: &mut &'a str) -> PResult<&'a str> {
    winnow::token::take_till(1.., |c| " \t\r\n".contains(c))
        .context(label("not_space"))
        .parse_next(input)
}

/// Read a new line including eol (\n) or consume the rest if there is no eol
/// char.
pub fn read_line<'a>(s: &mut &'a str) -> PResult<&'a str> {
    use winnow::ascii::till_line_ending;
    use winnow::combinator::opt;

    // use winnow::combinator::recognize;
    // if there is no newline in `s`, take the whole str
    let o = (till_line_ending, opt(line_ending)).recognize().parse_next(s)?;
    Ok(o)
}

/// Take the rest line. The line ending is not included.
pub fn rest_line<'a>(input: &mut &'a str) -> PResult<&'a str> {
    use winnow::ascii::{line_ending, till_line_ending};
    terminated(till_line_ending, line_ending).context(label("rest line")).parse_next(input)
}

/// Take and consuming to `literal`.
pub fn jump_to<'a>(literal: &str) -> impl FnMut(&mut &str) -> PResult<()> + '_ {
    use winnow::token::take_until;
    move |input: &mut &str| {
        let _: (&str, &str) = (take_until(0.., literal), literal).context(label("jump_to")).parse_next(input)?;
        Ok(())
    }
}

/// Take until found `literal`. The `literal` will not be consumed.
pub fn jump_until<'a>(literal: &str) -> impl FnMut(&mut &str) -> PResult<()> + '_ {
    use winnow::token::take_until;
    move |input: &mut &str| {
        let _: &str = take_until(0.., literal).context(label("jump_until")).parse_next(input)?;
        Ok(())
    }
}

/// A combinator that takes a parser `inner` and produces a parser
/// that also consumes both leading and trailing whitespace, returning
/// the output of `inner`.
pub fn ws<'a, ParseInner, Output, Error>(inner: ParseInner) -> impl Parser<&'a str, Output, Error>
where
    ParseInner: Parser<&'a str, Output, Error>,
    Error: ParserError<&'a str>,
{
    delimited(space0, inner, space0)
}
// fb1326ab ends here

// [[file:../parser.note::3d14b516][3d14b516]]
/// Match one unsigned integer: 123
pub fn unsigned_integer<'a>(input: &mut &'a str) -> PResult<usize> {
    use winnow::ascii::digit1;
    digit1.try_map(|x: &str| x.parse()).context(label("usize")).parse_next(input)
}

/// Match one signed integer: -123 or +123
pub fn signed_integer(s: &mut &str) -> PResult<isize> {
    use winnow::ascii::digit1;
    use winnow::combinator::alt;
    use winnow::combinator::opt;

    let sign = opt(alt(("-", "+")));
    (sign, digit1).recognize().try_map(|x: &str| x.parse::<isize>()).parse_next(s)
}

/// Parse a line containing an unsigned integer number.
pub fn read_usize(s: &mut &str) -> PResult<usize> {
    use winnow::ascii::{line_ending, space0};

    // allow white spaces
    let p = delimited(space0, unsigned_integer, space0);
    terminated(p, line_ending).parse_next(s)
}

/// Parse a line containing many unsigned numbers
pub fn read_usize_many(s: &mut &str) -> PResult<Vec<usize>> {
    use winnow::ascii::{line_ending, space0, space1};
    use winnow::combinator::separated;

    let x = seq! {
        _: space0,
        separated(1.., unsigned_integer, space1),
        _: space0,
        _: line_ending,
    }
    .parse_next(s)?;
    Ok(x.0)
}

pub use self::signed_integer as signed_digit;
pub use self::unsigned_integer as unsigned_digit;
// 3d14b516 ends here

// [[file:../parser.note::4ef79da3][4ef79da3]]
/// Parse a f64 float number
pub fn double(input: &mut &str) -> PResult<f64> {
    use winnow::ascii::float;
    float(input)
}

/// Consume three float numbers separated by one or more spaces. Return xyz array.
pub fn xyz_array(s: &mut &str) -> PResult<[f64; 3]> {
    use winnow::ascii::space1;
    let x = seq! {double, _: space1, double, _: space1, double}.parse_next(s)?;
    Ok([x.0, x.1, x.2])
}

/// Parse a line containing a float number possibly surrounded by spaces
pub fn read_double(s: &mut &str) -> PResult<f64> {
    use winnow::ascii::{line_ending, space0};

    // allow white spaces
    let p = delimited(space0, double, space0);
    terminated(p, line_ending).parse_next(s)
}

/// Parse a line containing many float numbers
pub fn read_double_many(s: &mut &str) -> PResult<Vec<f64>> {
    use winnow::ascii::{line_ending, space0, space1};
    use winnow::combinator::separated;

    let x = seq! {
        _: space0,
        separated(1.., double, space1),
        _: space0,
        _: line_ending,
    }
    .parse_next(s)?;
    Ok(x.0)
}
// 4ef79da3 ends here

// [[file:../parser.note::838e8dea][838e8dea]]
/// Convert a string to a float.
///
/// This method performs certain checks, that are specific to quantum
/// chemistry output, including avoiding the problem with Ds instead
/// of Es in scientific notation. Another point is converting string
/// signifying numerical problems (*****) to something we can manage
/// (NaN).
pub fn parse_float(s: &str) -> Option<f64> {
    if s.chars().all(|x| x == '*') {
        std::f64::NAN.into()
    } else {
        s.parse().ok().or_else(|| s.replacen("D", "E", 1).parse().ok())
    }
}

#[test]
fn test_fortran_float() {
    let x = parse_float("14");
    assert_eq!(x, Some(14.0));

    let x = parse_float("14.12E4");
    assert_eq!(x, Some(14.12E4));

    let x = parse_float("14.12D4");
    assert_eq!(x, Some(14.12E4));

    let x = parse_float("****");
    assert!(x.unwrap().is_nan());
}
// 838e8dea ends here

// [[file:../parser.note::10e5dba2][10e5dba2]]
#[test]
fn test_ws() -> PResult<()> {
    use winnow::ascii::{digit1, line_ending, space0};

    let s = " 123 ";
    let (_, x) = ws(digit1).parse_peek(s)?;
    assert_eq!(x, "123");

    let s = "123 ";
    let (_, x) = ws(digit1).parse_peek(s)?;
    assert_eq!(x, "123");

    let s = "123\n";
    let (_, x) = ws(digit1).parse_peek(s)?;
    assert_eq!(x, "123");

    Ok(())
}

#[test]
fn test_jump_to() {
    let x = "xxbcc aa cc";
    let (r, _) = jump_to("aa").parse_peek(x).unwrap();
    assert_eq!(r, " cc");
}

#[test]
fn test_read_line() {
    let txt = "first line\nsecond line\r\nthird line\n";
    let (rest, line) = read_line.parse_peek(txt).unwrap();
    assert_eq!(line, "first line\n");
    let (rest, line) = read_line.parse_peek(rest).unwrap();
    assert_eq!(line, "second line\r\n");
    let (rest, line) = read_line.parse_peek(rest).unwrap();
    assert_eq!(line, "third line\n");
    assert_eq!(rest, "");

    // when there is no newline
    let txt = "no newline at the end";
    let (rest, line) = read_line.parse_peek(txt).unwrap();
    assert_eq!(line, txt);
    assert_eq!(rest, "");

    let txt = "no";
    let (_, line) = not_space.parse_peek(txt).unwrap();
    assert_eq!(line, "no");

    let txt = "no ";
    let (_, line) = not_space.parse_peek(txt).unwrap();
    assert_eq!(line, "no");

    let txt = "no-a\n";
    let (_, line) = not_space.parse_peek(txt).unwrap();
    assert_eq!(line, "no-a");

    let txt = "no+b\t";
    let (_, line) = not_space.parse_peek(txt).unwrap();
    assert_eq!(line, "no+b");

    let txt = " no-a\n";
    let x = not_space.parse_peek(txt);
    assert!(x.is_err());
}

#[test]
fn test_read_many() {
    let (_, ns) = read_usize_many.parse_peek("11 2 3 4 5\r\n\n").expect("usize parser");
    assert_eq!(5, ns.len());
    let _ = read_usize_many.parse_peek(" 11 2 3 4 5 \n").expect("usize parser");
    let _ = read_usize_many.parse_peek("11 2 3 4 5 \r\n").expect("usize parser");

    let line = " 1.2  3.4 -5.7 0.2 \n";
    let (_, fs) = read_double_many.parse_peek(line).expect("f64 parser");
    assert_eq!(4, fs.len());
}

#[test]
fn test_signed_digit() {
    let (_, x) = signed_digit.parse_peek("-123").expect("signed digit, minus");
    assert_eq!(x, -123);

    let (_, x) = signed_digit.parse_peek("123").expect("signed digit, normal");
    assert_eq!(x, 123);

    let (_, x) = signed_digit.parse_peek("+123").expect("signed digit, plus");
    assert_eq!(x, 123);

    let s = "12x";
    let (r, n) = unsigned_digit.parse_peek(s).unwrap();
    assert_eq!(n, 12);
    assert_eq!(r, "x");

    let (r, n) = read_usize.parse_peek(" 12 \n").unwrap();
    assert_eq!(n, 12);
    assert_eq!(r, "");
}
// 10e5dba2 ends here
