// base

// [[file:~/Workspace/Programming/rust-libs/text-parser/text-parser.note::*base][base:1]]
#[macro_use]
extern crate nom;

use std::fs::File;
use textparser::*;
// base:1 ends here

//    2 1 3 1 3 14 0         1.324         1.345         1.300         3.969         0.000         0.193
#[inline]
fn read_bond_order_sum(input: &str) -> nom::IResult<&str, (usize, usize, f64)> {
    // read current atom index and its number of bonds
    let (input, (index, nbonds)) = sp!(input, do_parse!(
        index : unsigned_digit >>
                digit          >>
        nbonds: unsigned_digit >>
        ((index, nbonds))
    ))?;

    // read related bond orders
    let nskip = 2 * nbonds + 1;
    let (input, bosum) = sp!(input, do_parse!(
               count!(sp!(not_space), nskip) >> // ignore preceding items
        bosum: double                        >> read_line >>
        (bosum)
    ))?;

    Ok((input, (index, nbonds, bosum)))
}

#[test]
fn test_read_bond_order_sum() {
    let line = "   2 1 3 1 3 14 0         1.324         1.345         1.300         3.969         0.000         0.193  \n";

    let (_, (index, nbonds, bosum)) = read_bond_order_sum(line).expect("bond order sum");
    assert_eq!(index, 2);
    assert_eq!(nbonds, 14);
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

// [[file:~/Workspace/Programming/rust-libs/text-parser/text-parser.note::*part][part:1]]
use std::collections::HashMap;

named!(read_meta_from_comments<&str, (usize, usize)>, sp!(do_parse!(
    //call!(read_lines_until, "# Timestep") >>
    tag!("# Timestep")            >> nstep: read_usize >>
    read_line >>            // #
    tag!("# Number of particles") >> npts : read_usize >>
    read_line >>            // #
    read_line >>            // # Max number of bonds per atom 16 with coarse bond order cutoff 0.300
    read_line >>            // # Particle connection table and bond orders
    read_line >>            // # id type nb id_1...id_nb mol bo_1...bo_nb abo nlp q
    ((nstep, npts))
)));

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

fn read_part(input: &str) -> nom::IResult<&str, Vec<(usize, usize, f64)>> {
    let (input, values) = do_parse!(input,
        meta  : read_meta_from_comments     >>
        values: many1!(read_bond_order_sum) >> read_line >>
        (values)
    )?;

    Ok((input, values))
}

/// Calculate average number of bonded particles and bond order sum for each
/// particle in trajectory
pub fn average_bond_orders(fname: &str) -> Result<()>{
    let parser = TextParser::default();
    let fp = File::open(fname).expect("test bonds file");

    let mut map_nbonds = HashMap::new();
    let mut map_bosum  = HashMap::new();
    let mut i = 0;
    parser.parse(fp,
                 // parse a single part
                 read_part,
                 // collect all parts
                 |m| {
                     i += 1;
                     //println!("{:#?}", i);
                     for (index, nbonds, bosum) in m {
                         let v = map_nbonds.entry(index).or_insert(0.);
                         *v += nbonds as f64;
                         let v = map_bosum.entry(index).or_insert(0.);
                         *v += bosum;
                     }
                 }
    ).expect("text parser");

    // calculate averages
    for (_, v) in map_nbonds.iter_mut() {
        *v /= (i as f64)
    }
    for (_, v) in map_bosum.iter_mut() {
        *v /= (i as f64)
    }

    // println!("average coordinate number:\n{:#?}", map_nbonds);

    // println!("average bond order sum:\n{:#?}", map_bosum);

    Ok(())
}
// part:1 ends here

// main

// [[file:~/Workspace/Programming/rust-libs/text-parser/text-parser.note::*main][main:1]]
#[macro_use] extern crate quicli;
use quicli::prelude::*;
use ::structopt::StructOpt;
use std::time;

#[derive(Debug, StructOpt)]
struct Cli {
    /// The file to read
    file: String,
    // Quick and easy logging setup you get for free with quicli
    #[structopt(flatten)]
    verbosity: Verbosity,
}

main!(|args: Cli, log_level: verbosity| {
    println!("parsing {:}", &args.file);

    let now = time::SystemTime::now();
    average_bond_orders(&args.file)?;
    let delta= now.elapsed()?.as_secs();
    println!("elapsed time = {:} s", delta);
});
// main:1 ends here
