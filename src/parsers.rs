// base

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*base][base:1]]
pub use crate::core::complete::*;
pub use crate::core::*;

/// Read the remaining line. Return a line including eol
pub fn read_line(s: &str) -> IResult<&str, &str> {
    use nom::combinator::recognize;
    use nom::sequence::pair;

    recognize(pair(take_until("\n"), tag("\n")))(s)
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
}

/// Read the remaining line. Return a line excluding eol.
pub fn read_until_eol(s: &str) -> IResult<&str, &str> {
    use nom::character::complete::not_line_ending;
    use nom::character::complete::line_ending;

    nom::sequence::terminated(not_line_ending, line_ending)(s)
}

/// Match line ending preceded with zero or more whitespace chracters
pub fn eol(s: &str) -> IResult<&str, &str> {
    use nom::character::complete::line_ending;

    nom::sequence::terminated(space0, line_ending)(s)
}

/// Anything except whitespace, this parser will not consume "\n" character
pub fn not_space(s: &str) -> IResult<&str, &str> {
    use nom::bytes::complete::is_not;

    is_not(" \t\r\n")(s)
}

/// Match one unsigned integer: 123
pub fn unsigned_digit(s: &str) -> IResult<&str, usize> {
    use nom::combinator::map;

    map(digit1, |s: &str| s.parse().unwrap())(s)
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
    use nom::sequence::tuple;

    let (r, (x, _, y, _, z)) = tuple((double, space1, double, space1, double))(s)?;

    Ok((r, [x, y, z]))
}

/// Take and consuming to `token`.
pub fn jump_to<'a>(token: &'a str) -> impl Fn(&'a str) -> IResult<&str, ()> {
    use nom::combinator::map;
    use nom::sequence::pair;

    map(pair(take_until(token), tag(token)), |_| ())
}

#[test]
fn test_take() {
    let x = "xxbcc aa cc";
    let (r, _) = jump_to("aa")(x).unwrap();
    assert_eq!(r, " cc");
}
// base:1 ends here
