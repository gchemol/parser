// imports

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*imports][imports:1]]
use guts::prelude::*;
use guts::fs::*;
// imports:1 ends here

// reader

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*reader][reader:1]]
type FileReader = BufReader<File>;

#[derive(Debug)]
pub struct TextReader<R: BufRead> {
    reader: R,
}

impl TextReader<FileReader> {
    /// Build a text reader for file from path `p`.
    pub fn from_path<P: AsRef<Path>>(p: P) -> Result<Self> {
        let reader = text_file_reader(p)?;
        let parser = Self { reader };
        Ok(parser)
    }
}

impl<R: Read> TextReader<BufReader<R>> {
    /// Build a text reader from a struct implementing Read trait.
    pub fn new(r: R) -> Self {
        Self {
            reader: BufReader::new(r),
        }
    }
}

impl<R: BufRead> TextReader<R> {
    /// Returns an iterator over `n` lines at a time.
    pub fn chunks(self, nlines: usize) -> impl Iterator<Item = String> {
        read_chunk(self.reader, nlines)
    }

    /// Returns an iterator over the lines of this reader. Each string returned
    /// will not have a line ending.
    pub fn lines(self) -> impl Iterator<Item = String> {
        // silently ignore UTF-8 error
        self.reader
            .lines()
            .filter_map(|s| if let Ok(line) = s { Some(line) } else { None })
    }
}

fn text_file_reader<P: AsRef<Path>>(p: P) -> Result<FileReader> {
    let p = p.as_ref();
    let f = File::open(p).with_context(|| format!("Failed to open file {:?}", p))?;

    let reader = BufReader::new(f);
    Ok(reader)
}

/// Return an iterator over every n lines from `r`
fn read_chunk<R: Read>(r: R, nlines: usize) -> impl Iterator<Item = String> {
    let mut reader = BufReader::new(r);

    std::iter::from_fn(move || {
        let mut chunk = String::new();
        for _ in 0..nlines {
            match reader.read_line(&mut chunk) {
                Ok(n) if n == 0 => {
                    break;
                }
                Err(e) => {
                    eprintln!("Failed to read line: {:?}", e);
                    return None;
                }
                Ok(_) => {}
            }
        }

        if chunk.is_empty() {
            None
        } else {
            Some(chunk)
        }
    })
}
// reader:1 ends here

// bunches

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*bunches][bunches:1]]
impl<R: BufRead> TextReader<R> {
    /// Return an iterator over a bunch of lines preceded by a label line.
    pub fn bunches<F>(self, label_fn: F) -> Bunches<F, R>
    where
        F: Fn(&str) -> bool,
    {
        Bunches::new(self.reader, label_fn)
    }
}

pub struct Bunches<F, R>
where
    F: Fn(&str) -> bool,
    R: BufRead,
{
    lines: std::iter::Peekable<std::io::Lines<R>>,
    is_data_label: F,
}

impl<F, R> Bunches<F, R>
where
    F: Fn(&str) -> bool,
    R: BufRead,
{
    fn new(reader: R, f: F) -> Self {
        Self {
            lines: reader.lines().peekable(),
            is_data_label: f,
        }
    }
}

impl<F, R> Iterator for Bunches<F, R>
where
    F: Fn(&str) -> bool,
    R: BufRead,
{
    type Item = Vec<String>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chunk = vec![];
        loop {
            match self.lines.next() {
                Some(Ok(line)) => {
                    chunk.push(line);
                    // check next line to decide if return
                    match self.lines.peek() {
                        Some(Ok(next_line)) => {
                            if (self.is_data_label)(next_line) {
                                return Some(chunk);
                            }
                        }
                        Some(Err(e)) => {
                            warn!("found reading error: {}", e);
                        }
                        None => {
                            // reach eof
                            return Some(chunk);
                        }
                    }
                }
                Some(Err(e)) => {
                    // ignore
                    warn!("found reading error: {}", e);
                }
                None => {
                    // reach eof
                    break;
                }
            }
        }
        None
    }
}
// bunches:1 ends here

// test

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*test][test:1]]
#[test]
fn test_parser() -> Result<()> {
    let f = "./tests/files/lammps-test.dump";
    let reader = TextReader::from_path(f)?;
    let bunches = reader.bunches(|line| line.starts_with("ITEM: TIMESTEP"));
    assert_eq!(bunches.count(), 3);

    let f = "./tests/files/multi.xyz";
    let if_data_label = |line: &str| line.trim().parse::<usize>().is_ok();
    let reader = TextReader::from_path(f)?;
    let bunches = reader.bunches(if_data_label);
    assert_eq!(bunches.count(), 6);

    // test chunks
    let reader = TextReader::from_path(f)?;
    for chunk in reader.chunks(5) {
        // dbg!(chunk.lines().count());
    }

    // test lines
    let reader = TextReader::from_path(f)?;
    let line = reader.lines().skip(1).next().unwrap();
    assert_eq!(line, " Configuration number :        7");

    Ok(())
}
// test:1 ends here
