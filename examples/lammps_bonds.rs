// imports

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*imports][imports:1]]
use std::fs::File;
use text_parser::*;

use nom::*;
// imports:1 ends here

//    2 1 3 1 3 14 0         1.324         1.345         1.300         3.969         0.000         0.193
#[inline]
fn read_bond_order_sum(input: &str) -> IResult<&str, (usize, usize, f64)> {
    // read current atom index and its number of bonds
    let (input, (index, nbonds)) = do_parse!(
        input,
        space0 >>                 // leading spaces
        index: unsigned_digit >>  // column 1
        space1 >>
        digit1 >>                 // column 2
        space1 >>
        nbonds: unsigned_digit >> // column 3
        ((index, nbonds))
    )?;

    // read related bond orders
    let nskip = 2 * nbonds + 1;
    let pskip = count(terminated(not_space, space1), nskip);
    let (input, bosum) = do_parse!(
        input,
        space1 >> pskip >> // ignore preceding items
        bosum: double   >>
        read_until_eol  >>
        (bosum)
    )?;

    Ok((input, (index, nbonds, bosum)))
}

#[test]
fn test_read_bond_order_sum() {
    let line = "   2 1 3 1 3 14 0         1.324         1.345         1.300         3.969         0.000         0.193  \n";

    let (_, (index, nbonds, bosum)) = read_bond_order_sum(line).expect("bond order sum");
    assert_eq!(index, 2);
    assert_eq!(nbonds, 3);
    assert_eq!(bosum, 3.969);
}

// part
// # Timestep 0
// #
// # Number of particles 1822
// #
// # Max number of bonds per atom 16 with coarse bond order cutoff 0.300
// # Particle connection table and bond orders
// # id type nb id_1...id_nb mol bo_1...bo_nb abo nlp q
//  5 1 14 212 248 824 1000 392 648 1320 417 481 597 1381 245 1493 904 0

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*part][part:1]]
use std::collections::HashMap;

named!(read_meta_from_comments<&str, (usize, usize)>, do_parse!(
    tag!("# Timestep")            >> nstep: read_usize >> // # Timestep 0
    read_until_eol                >>                      // #
    tag!("# Number of particles") >> npts: read_usize  >> // # Number of particles 1822
    read_until_eol                >> // #
    read_until_eol                >> // # Max number of bonds per atom 16 with coarse bond order cutoff 0.300
    read_until_eol                >> // # Particle connection table and bond orders
    read_until_eol                >> // # id type nb id_1...id_nb mol bo_1...bo_nb abo nlp q
    ((nstep, npts))
));

#[test]
fn test_read_meta() {
    let txt = "\
# Timestep 0
#
# Number of particles 1822
#
# Max number of bonds per atom 16 with coarse bond order cutoff 0.300
# Particle connection table and bond orders
# id type nb id_1...id_nb mol bo_1...bo_nb abo nlp q
";

    let (_, (nstep, npts)) = read_meta_from_comments(txt).expect("meta");
    assert_eq!(nstep, 0);
    assert_eq!(npts, 1822);
}

fn read_part(s: &str) -> IResult<&str, Vec<(usize, usize, f64)>> {
    let (rest, (nstep, npts)) = read_meta_from_comments(s)?;
    std::dbg!(nstep);
    terminated(count(read_bond_order_sum, npts), tag("# \n"))(rest)
}

/// Calculate average number of bonded particles and bond order sum for each
/// particle in trajectory
pub fn average_bond_orders(fname: &str) -> Result<()> {
    let parser = TextParser::new(1050);
    let fp = File::open(fname).expect("test bonds file");

    let mut map_nbonds = HashMap::new();
    let mut map_bosum = HashMap::new();

    let mut i = 0;
    for m in parser.parse(fp, read_part) {
        i += 1;
        for (index, nbonds, bosum) in m {
            let v = map_nbonds.entry(index).or_insert(0.);
            *v += nbonds as f64;
            let v = map_bosum.entry(index).or_insert(0.);
            *v += bosum;
        }
    }

    // calculate averages
    for (_, v) in map_nbonds.iter_mut() {
        *v /= (i as f64)
    }
    for (_, v) in map_bosum.iter_mut() {
        *v /= (i as f64)
    }

    Ok(())
}
// part:1 ends here

// main

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*main][main:1]]
use guts::cli::*;
use guts::prelude::*;

use std::time;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Cli {
    /// The file to read
    file: String,
}

fn main() -> CliResult {
    let args = Cli::from_args();
    println!("parsing {:}", &args.file);

    let now = time::SystemTime::now();
    average_bond_orders(&args.file)?;
    let delta = now.elapsed()?.as_secs();
    println!("elapsed time = {:} s", delta);

    Ok(())
}
// main:1 ends here
