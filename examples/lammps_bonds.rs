// [[file:../parser.note::*imports][imports:1]]
use std::fs::File;

use gchemol_parser::*;
use gchemol_parser::parsers::*;
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
    let mut pskip = count(terminated(not_space, space1), nskip);
    let (input, bosum) = do_parse!(
        input,
        space1 >> pskip >> // ignore preceding items
        bosum: double   >>
        read_line       >>
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

// [[file:../parser.note::*parts][parts:1]]
use std::collections::HashMap;

fn read_meta_from_comments(s: &str) -> IResult<&str, (usize, usize)> {
    let tag_timestep = tag("# Timestep");
    let tag_nparticles = tag("# Number of particles");
    do_parse!(
        s,
        tag_timestep            >> nstep: read_usize >> // # Timestep 0
        read_line               >>                      // #
        tag_nparticles          >> npts: read_usize  >> // # Number of particles 1822
        read_line               >> // #
        read_line               >> // # Max number of bonds per atom 16 with coarse bond order cutoff 0.300
        read_line               >> // # Particle connection table and bond orders
        read_line               >> // # id type nb id_1...id_nb mol bo_1...bo_nb abo nlp q
        ((nstep, npts))
    )
}

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
    terminated(count(read_bond_order_sum, npts), tag("# \n"))(rest)
}
// parts:1 ends here

// [[file:../parser.note::2894a3cd][2894a3cd]]
use gut::fs::*;
fn average_bond_orders(fname: &str) -> Result<()> {
    // read the first 8 lines, determine the number of atoms in each frame
    let r = TextReader::try_from_path(fname.as_ref())?;
    let chunk = r.chunks(8).next().unwrap();
    let (_, (_, n)) = read_meta_from_comments(&chunk).unwrap();
    dbg!(n);

    // parse data for each frame
    let mut map_nbonds = HashMap::new();
    let mut map_bosum = HashMap::new();
    let mut i = 0;
    let r = TextReader::try_from_path(fname.as_ref())?;
    for chunk in r.chunks(n + 7 + 1) {
        i += 1;
        let (_, m) = read_part(&chunk).unwrap();
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
// 2894a3cd ends here

// [[file:../parser.note::*main][main:1]]
use gut::cli::*;
use gut::prelude::*;

use std::time;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Cli {
    /// The file to read
    file: String,
}

fn main() -> Result<()> {
    let args = Cli::from_args();
    setup_logger();

    println!("parsing {:}", &args.file);
    let now = time::SystemTime::now();
    average_bond_orders(&args.file)?;
    let delta = now.elapsed()?.as_secs();
    println!("elapsed time = {:} s", delta);

    Ok(())
}
// main:1 ends here
