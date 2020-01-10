// imports

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*imports][imports:1]]
use guts::prelude::*;
use guts::fs::*;
// imports:1 ends here

// base

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*base][base:1]]
/// Return an iterator over a bunch of lines from reader `r`
fn read_by_chunk<R, C>(r: R, mut collect_done: C) -> impl Iterator<Item = String>
where
    R: Read,
    C: FnMut(&str) -> bool,
{
    let mut reader = BufReader::new(r);

    std::iter::from_fn(move || {
        let mut chunk = String::new();
        loop {
            match reader.read_line(&mut chunk) {
                Ok(n) if n == 0 => {
                    break;
                }
                Ok(_) => {
                    if collect_done(&chunk) {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read line: {:?}", e);
                    return None;
                }
            }
        }

        if chunk.is_empty() {
            None
        } else {
            Some(chunk)
        }
    })
}

#[test]
fn test() {
    use std::io::Cursor;

    let x = "1
2
3
4
5";
    let r = Cursor::new(x.as_bytes());
    let mut i = 0;
    for chunk in read_by_chunk(r, |chunk| {
        i += 1;
        if i >= 2 {
            i = 0;
            true
        } else {
            false
        }
    }) {
        dbg!(chunk);
    }
}
// base:1 ends here

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

// parts

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*parts][parts:1]]
impl TextReader {
    /// Split into multiple parts separated by a data label line.
    pub fn split_if<F>(self, label_fn: F) -> DataRecords<F>
    where
        F: Fn(&str) -> bool,
    {
        DataRecords::new(self.reader, label_fn)
    }
}

pub struct DataRecords<F>
where
    F: Fn(&str) -> bool,
{
    label: String,
    lines: std::io::Lines<FileReader>,
    is_data_label: F,
}

impl<F> Iterator for DataRecords<F>
where
    F: Fn(&str) -> bool,
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut data_lines = String::from(&self.label);
        loop {
            match self.lines.next() {
                // label line
                Some(Ok(line)) if (self.is_data_label)(&line) => {
                    // safe data label
                    let head = self.label.to_string();
                    self.label = line;
                    self.label += "\n";
                    // ignore empty record
                    if !data_lines.is_empty() {
                        return Some(data_lines);
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
        if data_lines.is_empty() {
            return None;
        } else {
            let part = data_lines.clone();
            self.label.clear();
            data_lines.clear();
            return Some(part);
        }
    }
}

impl<F> DataRecords<F>
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
// parts:1 ends here

// test

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*test][test:1]]
#[test]
fn test_parser() {
    let f = "./tests/files/lammps-test.dump";
    let parser = TextReader::from_path(f).unwrap();
    let records = parser.split_if(|line| line.starts_with("ITEM: TIMESTEP"));
    assert_eq!(records.count(), 3);

    let f = "./tests/files/multi.xyz";
    let parser = TextReader::from_path(f).unwrap();
    let records = parser.split_if(|line| line.trim().parse::<usize>().is_ok());
    assert_eq!(records.count(), 6);
    // for (i, p) in records.take(3).enumerate() {
    //     println!("{} => {}", i, p);
    // }

    let parser = TextReader::from_path(f).unwrap();
    for chunk in parser.chunks(5) {
        // dbg!(chunk.lines().count());
    }
}
// test:1 ends here
