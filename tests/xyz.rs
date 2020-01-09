// xyz.rs
// :PROPERTIES:
// :header-args: :tangle tests/xyz.rs
// :END:

// [[file:~/Workspace/Programming/gchemol-rs/parser/parser.note::*xyz.rs][xyz.rs:1]]
use guts::prelude::*;
use text_parser::old::*;
use text_parser::*;

/// A minimal representation for chemical atom.
#[derive(Debug)]
struct Atom {
    symbol: String,
    position: [f64; 3],
}

impl Atom {
    fn new(sym: &str, position: [f64; 3]) -> Self {
        Self {
            symbol: sym.into(),
            position,
        }
    }
}

/// Create Atom object from xyz line
///
/// # Example
///
/// C -11.4286  1.7645  0.0000
named!(read_atom_xyz<&str, Atom>,
    do_parse!(
                  space0    >>                   // ignore optional preceeding space
        sym     : alpha1    >> space1         >> // element symbol, e.g. "Fe"
        position: xyz_array >> read_until_eol >> // ignore the remaining characters
        (
            Atom::new(sym, position)
        )
    )
);

#[test]
fn test_parser_read_atom() {
    let (_, x) = read_atom_xyz("C -11.4286 -1.3155  0.0000 \n").unwrap();
    assert_eq!("C", x.symbol);
    let (_, x) = read_atom_xyz(" C -11.4286 -1.3155  0.0000 \n").unwrap();
    assert_eq!("C", x.symbol);
    assert_eq!(0.0, x.position[2]);
}

/// read meta data in xyz format
///
/// # Example
///
/// 16
/// this is comment line
named!(read_meta<&str, (usize, &str)>,
    do_parse!(
        natoms : read_usize     >>
        title  : read_until_eol >>
        (
            (natoms, title)
        )
    )
);

#[test]
fn test_read_meta() {
    let txt = "          16
 Configuration number :        7
";

    let (_, x) = read_meta(txt).unwrap();
    assert_eq!(x.0, 16);
    assert_eq!(x.1, " Configuration number :        7");
}

/// Create a list of atoms from many lines in xyz format
///
/// # Example
///
/// 16
/// comment line
/// C -11.4286  1.7645  0.0000
/// C -10.0949  0.9945  0.0000
/// C -10.0949 -0.5455  0.0000
///
fn read_xyz_stream(input: &str) -> IResult<&str, Vec<Atom>> {
    let (rest, (natoms, _)) = read_meta(input)?;
    many_m_n(natoms, natoms, read_atom_xyz)(rest)
}

#[test]
fn test_parser_read_xyz() {
    let txt = "          16
 Configuration number :        7
   N   1.38635  -0.29197   0.01352
   N  -1.38633   0.29227   0.00681
   C   0.91882   0.97077  -0.01878
   C  -0.44889   1.25897  -0.02208
   C  -0.91881  -0.97095   0.00730
   C   0.44886  -1.25914   0.01058
   H   1.66107   1.76596  -0.02576
   H  -0.80712   2.28604  -0.03176
   H   0.80714  -2.28611   0.02735
   H  -1.66109  -1.76602   0.02139
   O   4.17450  -0.57938  -0.37886
   H   3.20186  -0.81182  -0.38259
   H   4.56688  -0.80173   0.51370
   O  -3.77040   0.96374  -1.49419
   H  -3.35189   1.74003  -1.96565
   H  -3.08717   0.51682  -0.91667
";
    let (_, x) = read_xyz_stream(txt).unwrap();
    assert_eq!(x.len(), 16);
}

#[test]
fn test_text_parser() -> Result<()> {
    let fname = "tests/files/multi.xyz";
    let parser = TextParser::default();
    let parts: Vec<_> = parser.parse_file(fname, read_xyz_stream).collect();

    assert_eq!(parts.len(), 6);

    Ok(())
}
// xyz.rs:1 ends here
