// imports

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*imports][imports:1]]
use guts::prelude::*;
use guts::fs::*;
// imports:1 ends here

// reader

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*reader][reader:1]]
type FileReader = BufReader<File>;

#[derive(Debug)]
pub struct TextReader {
    reader: FileReader,
}

impl TextReader {
    /// Build a text parser for file from path `p`.
    pub fn from_path<P: AsRef<Path>>(p: P) -> Result<Self> {
        let reader = text_file_reader(p)?;
        let parser = Self { reader };
        Ok(parser)
    }

    /// Returns an iterator over `n` lines at a time.
    pub fn chunks(self, nlines: usize) -> impl Iterator<Item = String> {
        read_chunk(self.reader, nlines)
    }

    /// Returns an iterator over the lines of this reader. Each string returned
    /// will not have a line ending.
    pub fn lines(self) -> std::io::Lines<FileReader> {
        self.reader.lines()
    }
}

fn text_file_reader<P: AsRef<Path>>(p: P) -> Result<FileReader> {
    let f = File::open(p.as_ref())?;
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

// records

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*records][records:1]]
impl TextReader {
    /// Split into multiple records separated by a label line.
    pub fn records<F>(self, label_fn: F) -> Records<F>
    where
        F: Fn(&str) -> bool,
    {
        Records::new(self.reader, label_fn)
    }
}

pub struct Records<F>
where
    F: Fn(&str) -> bool,
{
    label: String,
    lines: std::io::Lines<FileReader>,
    is_data_label: F,
}

impl<F> Iterator for Records<F>
where
    F: Fn(&str) -> bool,
{
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        let mut data_lines = String::new();
        loop {
            match self.lines.next() {
                // label line
                Some(Ok(line)) if (self.is_data_label)(&line) => {
                    let head = self.label.to_string();
                    // save data label
                    self.label = line;
                    // skip the first empty line
                    if !head.is_empty() {
                        return Some((head, data_lines));
                    }
                }
                // normal line
                Some(Ok(line)) => {
                    data_lines += &line;
                    data_lines += "\n";
                }
                // reach EOF
                None => {
                    break;
                }
                Some(Err(e)) => {
                    error!("read line error:\n {}", e);
                    return None;
                }
            }
        }
        // handle final record
        if !data_lines.is_empty() {
            let part = (self.label.clone(), data_lines.clone());
            data_lines.clear();
            return Some(part);
        } else {
            return None;
        }
    }
}

impl<F> Records<F>
where
    F: Fn(&str) -> bool,
{
    fn new(reader: FileReader, f: F) -> Self {
        Self {
            lines: reader.lines(),
            label: String::new(),
            is_data_label: f,
        }
    }
}
// records:1 ends here

// test

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*test][test:1]]
#[test]
fn test_parser() {
    let f = "./tests/files/lammps-test.dump";
    let reader = TextReader::from_path(f).unwrap();
    let records = reader.records(|line| line.starts_with("ITEM: TIMESTEP"));
    assert_eq!(records.count(), 3);

    let f = "./tests/files/multi.xyz";
    let if_data_label = |line: &str| line.trim().parse::<usize>().is_ok();
    let reader = TextReader::from_path(f).unwrap();
    let records = reader.records(if_data_label);
    for (_label, _data) in records.take(5) {
        // dbg!(_label);
        // dbg!(_data);
    }

    let reader = TextReader::from_path(f).unwrap();
    for chunk in reader.chunks(5) {
        // dbg!(chunk.lines().count());
    }
}
// test:1 ends here
