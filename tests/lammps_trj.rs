// imports

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*imports][imports:1]]
use std::collections::HashMap;

use gchemol_parser::parsers::*;
use gut::prelude::*;
// imports:1 ends here

// meta

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*meta][meta:1]]
#[derive(Debug)]
struct FrameData {
    timestep: usize,
    natoms: usize,
}

fn read_meta_data(s: &str) -> IResult<&str, FrameData> {
    let tag_timestep = tag("ITEM: TIMESTEP");
    let tag_natoms = tag("ITEM: NUMBER OF ATOMS");
    do_parse!(
        s,
                  tag_timestep >> eol >>
        timestep: read_usize  >> // current timestep in this frame
                  tag_natoms >> eol >>
        natoms  : read_usize  >> // number of atoms
        (
            FrameData {
                timestep, natoms
            }
        )
    )
}

#[test]
fn test_read_meta_data() {
    let txt = "ITEM: TIMESTEP
 0
ITEM: NUMBER OF ATOMS
537
ITEM: BOX BOUNDS pp pp pp
-200.487 200.487
-200.487 200.487
-200.487 200.487
";
    let (_, x) = read_meta_data(txt).unwrap();
    assert_eq!(x.timestep, 0);
    assert_eq!(x.natoms, 537);
}
// meta:1 ends here

// box
// To be extended.

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*box][box:1]]
#[derive(Debug)]
struct BoxData {
    t: String,
    a: String,
    b: String,
    c: String,
}

fn read_box_data(s: &str) -> IResult<&str, BoxData> {
    let tag_box_bounds = tag("ITEM: BOX BOUNDS");
    do_parse!(
        s,
        tag_box_bounds >>
        t: read_until_eol >> // pp pp pp
        a: read_until_eol >> // -200.487 200.487
        b: read_until_eol >> // -200.487 200.487
        c: read_until_eol >> // -200.487 200.487
        (
            BoxData {
                t: t.into(), a: a.into(), b: b.into(), c: c.into()
            }
        )
    )
}

#[test]
fn test_read_box_data() {
    let txt = "ITEM: BOX BOUNDS pp pp pp
-200.487 200.487
-200.487 200.487
-200.487 200.487
";
    let (_, x) = read_box_data(txt).unwrap();
}
// box:1 ends here

// atoms
// Note: data in atom id column is not always sorted

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*atoms][atoms:1]]
// ITEM: ATOMS id type x y z c_eng c_cn c_cnt c_cna
fn read_atom_header(s: &str) -> IResult<&str, &str> {
    let tag_item = tag("ITEM: ATOMS");
    do_parse!(
        s,
        tag_item >>
        header: read_until_eol >> // the line including column headers
        (header)
    )
}

#[test]
fn test_read_atom_header() {
    let txt = "ITEM: ATOMS id type x y z c_eng c_cn c_cnt c_cna
1 1 0.542919 3.02575 -9.81464 -3.53666 9 9.42606 5
";
    let (_, x) = read_atom_header(txt).unwrap();
    assert_eq!(x, " id type x y z c_eng c_cn c_cnt c_cna");
}

#[derive(Debug)]
struct Atom {
    symbol: usize,
    x: f64,
    y: f64,
    z: f64,
}

fn read_atoms(input: &str, natoms: usize) -> IResult<&str, HashMap<usize, Atom>> {
    let (rest, header_line) = read_atom_header(input)?;
    let (rest, atom_lines) = many_m_n(natoms, natoms, read_until_eol)(rest)?;

    // collect column headers
    let headers: Vec<_> = header_line.trim().split_whitespace().collect();

    // parse rows
    let nheaders = headers.len();
    // required labels
    let labels = vec!["type", "id", "x", "y", "z"];
    let mapping: HashMap<_, _> = labels
        .iter()
        .map(|k| {
            let i = headers
                .iter()
                .position(|s| s == k)
                .expect("missing header lable");
            (*k, i)
        })
        .collect();

    // parse atom properties
    let atoms: HashMap<_, _> = atom_lines
        .into_iter()
        .map(|line| {
            let items: Vec<_> = line.trim().split_whitespace().collect();
            assert_eq!(items.len(), nheaders);
            let sym: usize = {
                let i = mapping["type"];
                items[i].parse().expect("invalid type data")
            };
            let id: usize = {
                let i = mapping["id"];
                items[i].parse().expect("invalid id data")
            };
            let x: f64 = {
                let i = mapping["x"];
                items[i].parse().expect("invalid x data")
            };
            let y: f64 = {
                let i = mapping["y"];
                items[i].parse().expect("invalid y data")
            };
            let z: f64 = {
                let i = mapping["z"];
                items[i].parse().expect("invalid z data")
            };

            let atom = Atom {
                symbol: sym.into(),
                x,
                y,
                z,
            };
            (id, atom)
        })
        .collect();

    Ok((rest, atoms))
}

#[test]
fn test_read_atoms() {
    let txt = "ITEM: ATOMS id type x y z c_eng c_cn c_cnt c_cna
1 1 0.542919 3.02575 -9.81464 -3.53666 9 9.42606 5
2 1 -0.566175 4.44199 -8.2352 -3.78452 11 5.16628 5
3 1 0.274667 2.58508 -6.92683 -4.05727 12 4.9903 5
4 1 -1.0285 4.21504 -5.29927 -4.12135 10 4.87744 5
5 1 -0.209709 2.20696 -4.20742 -3.99243 10 0.410664 3
6 1 -1.26547 3.85937 -2.59029 -4.01771 10 0.241619 5
7 1 -0.638752 1.99003 -1.2366 -4.03166 11 0.141509 3
8 1 -1.75827 3.6228 0.296721 -4.14419 13 0.28179 5
9 1 -0.927052 1.51729 1.59026 -3.98979 11 0.889157 5
10 1 -2.18531 3.375 2.91709 -4.08537 12 4.52081 5
11 1 -1.33645 1.21535 4.42074 -4.04849 12 0.307786 3
";

    let (_, x) = read_atoms(txt, 5).unwrap();
    assert_eq!(x.len(), 5);
    assert_eq!(x[&5].x, -0.209709);
}
// atoms:1 ends here

// main

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*main][main:1]]
fn read_lammps_dump(input: &str) -> IResult<&str, HashMap<usize, Atom>> {
    let (rest, frame_data) = read_meta_data(input)?;
    let (rest, box_data) = read_box_data(rest)?;
    read_atoms(rest, frame_data.natoms)
}

#[test]
fn test_parser() -> Result<()> {
    use gchemol_parser::TextReader;
    let fname = "tests/files/lammps-test.dump";
    let reader = TextReader::from_path(fname)?;
    let frames: Vec<_> = reader
        .preceded_bunches(|line| line.starts_with("ITEM: TIMESTEP"))
        .map(|data| {
            let (_, part) = read_lammps_dump(&data).unwrap();
            part
        })
        .collect();
    assert_eq!(frames.len(), 3);

    Ok(())
}
// main:1 ends here
