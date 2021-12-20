// [[file:../../parser.note::*imports][imports:1]]
use super::*;
// imports:1 ends here

// [[file:../../parser.note::aba05bc2][aba05bc2]]
use ::grep::regex::{RegexMatcher, RegexMatcherBuilder};

// Line oriented matches span no more than one line. The given pattern should
// not contain a literal \n.
pub fn make_matcher(pat: &str) -> Result<RegexMatcher> {
    let matcher = RegexMatcher::new_line_matcher(&pat)?;
    Ok(matcher)
}

// Build a new matcher from a plain alternation of literals, substantially
// faster than by joining the patterns with a | and calling build.
pub fn build_matcher_for_literals<B: AsRef<str>>(literals: &[B]) -> Result<RegexMatcher> {
    let matcher = RegexMatcherBuilder::new()
        .line_terminator(Some(b'\n'))
        // allow ^ matches the beginning of lines and $ matches the end of lines
        .multi_line(true)
        .build_literals(literals)?;

    Ok(matcher)
}
// aba05bc2 ends here

// [[file:../../parser.note::f1d2704d][f1d2704d]]
use ::grep::searcher::{Sink, SinkError, SinkMatch};

/// The closure accepts two parameters: the absolute position of matched line
/// and a UTF-8 string containing the matched data. The closure returns a
/// `std::io::Result<bool>`. If the `bool` is `false`, then the search stops
/// immediately. Otherwise, searching continues.
#[derive(Clone, Debug)]
pub struct PartSink<F>(pub F)
where
    F: FnMut(u64, &str) -> std::io::Result<bool>;

impl<F> Sink for PartSink<F>
where
    F: FnMut(u64, &str) -> std::io::Result<bool>,
{
    type Error = std::io::Error;

    fn matched(&mut self, _searcher: &Searcher, mat: &SinkMatch<'_>) -> std::io::Result<bool> {
        let matched_line = std::str::from_utf8(mat.bytes()).map_err(|e| Self::Error::error_message(e))?;
        // the absolute byte offset of the start of this match relative to the
        // very beginning of the input.
        let matched_line_position = mat.absolute_byte_offset();
        (self.0)(matched_line_position, &matched_line)
    }
}
// f1d2704d ends here

// [[file:../../parser.note::ca7a00d2][ca7a00d2]]
use ::grep::searcher::{BinaryDetection, Searcher, SearcherBuilder};

/// Do not count line number
pub fn make_searcher() -> Searcher {
    SearcherBuilder::new()
        .line_number(false)
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .build()
}
// ca7a00d2 ends here
