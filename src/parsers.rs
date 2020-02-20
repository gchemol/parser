// base

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*base][base:1]]
pub use crate::core::complete::*;
pub use crate::core::*;

/// Read a new line including eol (\n) or consume the rest if there is no eol
/// char.
pub fn read_line(s: &str) -> IResult<&str, &str> {
    use nom::combinator::recognize;

    // if there is no newline if `s`, take the whole str
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

/// Match one unsigned integer: 123
pub fn unsigned_digit(s: &str) -> IResult<&str, usize> {
    map(digit1, |s: &str| s.parse().unwrap())(s)
}

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
    let (r, (x, _, y, _, z)) = tuple((double, space1, double, space1, double))(s)?;

    Ok((r, [x, y, z]))
}

/// Take and consuming to `token`.
pub fn jump_to<'a>(token: &'a str) -> impl Fn(&'a str) -> IResult<&str, ()> {
    map(pair(take_until(token), tag(token)), |_| ())
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
pub fn take_s<'a>(n: usize) -> impl Fn(&'a str) -> IResult<&'a str, &'a str> {
    take(n)
}
// base:1 ends here
