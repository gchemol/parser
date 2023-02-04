// [[file:../parser.note::*docs][docs:1]]
//! Selected nom parser combinators (complete version, no streaming)
// docs:1 ends here

// [[file:../parser.note::653c73de][653c73de]]
pub use crate::core::complete::*;
pub use crate::core::*;

/// Read a new line including eol (\n) or consume the rest if there is no eol
/// char.
pub fn read_line(s: &str) -> IResult<&str, &str> {
    use nom::combinator::recognize;

    // if there is no newline in `s`, take the whole str
    let (rest, line_opt) = opt(recognize(pair(take_until("\n"), tag("\n"))))(s)?;
    match line_opt {
        None => nom::combinator::rest(rest),
        Some(line) => Ok((rest, line)),
    }
}

#[test]
fn test_read_line() {
    let txt = "first line\nsecond line\r\nthird line\n";
    let (rest, line) = read_line(txt).unwrap();
    assert_eq!(line, "first line\n");
    let (rest, line) = read_line(rest).unwrap();
    assert_eq!(line, "second line\r\n");
    let (rest, line) = read_line(rest).unwrap();
    assert_eq!(line, "third line\n");
    assert_eq!(rest, "");

    // when there is no newline
    let txt = "no newline at the end";
    let (rest, line) = read_line(txt).unwrap();
    assert_eq!(line, txt);
    assert_eq!(rest, "");
}

/// Read the remaining line. Return a line excluding eol.
pub fn read_until_eol(s: &str) -> IResult<&str, &str> {
    use nom::character::complete::line_ending;
    use nom::character::complete::not_line_ending;

    nom::sequence::terminated(not_line_ending, line_ending)(s)
}

/// Match line ending preceded with zero or more whitespace chracters
pub fn eol(s: &str) -> IResult<&str, &str> {
    use nom::character::complete::line_ending;

    nom::sequence::terminated(space0, line_ending)(s)
}

/// Anything except whitespace, this parser will not consume "\n" character
pub fn not_space(s: &str) -> IResult<&str, &str> {
    is_not(" \t\r\n")(s)
}

/// Take and consuming to `token`.
pub fn jump_to<'a>(token: &'a str) -> impl FnMut(&'a str) -> IResult<&str, ()> {
    context("jump to", map(pair(take_until(token), tag(token)), |_| ()))
}

#[test]
fn test_take() {
    let x = "xxbcc aa cc";
    let (r, _) = jump_to("aa")(x).unwrap();
    assert_eq!(r, " cc");
}

/// Return and consume n elements from input string slice.
///
/// Why not use take directly: for avoiding compiling error when using in
/// do_parse macro.
pub fn take_s<'a>(n: usize) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str> {
    take(n)
}

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing whitespace, returning the output of `inner`.
pub fn ws<'a, F: 'a, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: Fn(&'a str) -> IResult<&'a str, O>,
{
    let p = delimited(space0, inner, space0);
    context("white space", p)
}
// 653c73de ends here

// [[file:../parser.note::06980c7a][06980c7a]]
/// Match one unsigned integer: 123
pub fn unsigned_digit(s: &str) -> IResult<&str, usize> {
    map_res(digit1, |s: &str| s.parse())(s)
}

/// Match one unsigned integer: -123 or +123
pub fn signed_digit(s: &str) -> IResult<&str, isize> {
    use nom::combinator::recognize;
    let sign = opt(alt((tag("-"), tag("+"))));
    map_res(recognize(pair(sign, digit1)), |x: &str| x.parse::<isize>())(s)
}

#[test]
fn test_signed_digit() {
    let (_, x) = signed_digit("-123").expect("signed digit, minus");
    assert_eq!(x, -123);

    let (_, x) = signed_digit("123").expect("signed digit, normal");
    assert_eq!(x, 123);

    let (_, x) = signed_digit("+123").expect("signed digit, plus");
    assert_eq!(x, 123);
}

/// Parse a line containing an unsigned integer number.
pub fn read_usize(s: &str) -> IResult<&str, usize> {
    use nom::character::complete::line_ending;

    // allow white spaces
    let p = nom::sequence::delimited(space0, unsigned_digit, space0);
    nom::sequence::terminated(p, line_ending)(s)
}

#[test]
fn test_numbers() {
    let s = "12x";
    let (r, n) = unsigned_digit(s).unwrap();
    assert_eq!(n, 12);
    assert_eq!(r, "x");

    let (r, n) = read_usize(" 12 \n").unwrap();
    assert_eq!(n, 12);
    assert_eq!(r, "");
}

/// Consume three float numbers separated by one or more spaces. Return xyz array.
pub fn xyz_array(s: &str) -> IResult<&str, [f64; 3]> {
    let p = tuple((double, space1, double, space1, double));
    let (r, (x, _, y, _, z)) = context("xyz array", p)(s)?;

    Ok((r, [x, y, z]))
}

/// Parse a line containing many unsigned integers
pub fn read_usize_many(s: &str) -> IResult<&str, Vec<usize>> {
    use nom::character::complete::line_ending;

    nom::sequence::terminated(
        nom::sequence::delimited(space0, nom::multi::separated_list1(space1, unsigned_digit), space0),
        line_ending,
    )(s)
}

/// Parse a line containing a float number
pub fn read_double(s: &str) -> IResult<&str, f64> {
    use nom::character::complete::line_ending;

    // allow white spaces
    let p = nom::sequence::delimited(space0, double, space0);
    nom::sequence::terminated(p, line_ending)(s)
}

/// Parse a line containing many float numbers
pub fn read_double_many(s: &str) -> IResult<&str, Vec<f64>> {
    use nom::character::complete::line_ending;

    nom::sequence::terminated(
        nom::sequence::delimited(space0, nom::multi::separated_list1(space1, double), space0),
        line_ending,
    )(s)
}

#[test]
fn test_read_numbers() {
    let (_, ns) = read_usize_many("11 2 3 4 5\r\n\n").expect("usize parser");
    assert_eq!(5, ns.len());
    let _ = read_usize_many(" 11 2 3 4 5 \n").expect("usize parser");
    let _ = read_usize_many("11 2 3 4 5 \r\n").expect("usize parser");

    let line = " 1.2  3.4 -5.7 0.2 \n";
    let (_, fs) = read_double_many(line).expect("f64 parser");
    assert_eq!(4, fs.len());
}
// 06980c7a ends here

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
