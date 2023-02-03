// [[file:../../parser.note::88a60571][88a60571]]
use super::*;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct GrepJsonOut {
    r#type: String,
    data: Data,
}

#[derive(Deserialize, Debug)]
struct Data {
    absolute_offset: u64,
}

/// Mark positions with `pattern` using external ripgrep command.
///
/// # Parameters
/// * max_count: exits search if max_count matches reached.
pub fn mark_matched_positions_with_ripgrep(pattern: &str, path: &Path, max_count: Option<usize>) -> Result<Vec<u64>> {
    use gut::cli::duct::cmd;

    let json_out = if let Some(m) = max_count {
        cmd!("rg", "--no-line-number", "--max-count", m.to_string(), "--json", pattern, path).read()?
    } else {
        cmd!("rg", "--no-line-number", "--json", pattern, path).read()?
    };

    let mut marked_positions = vec![];
    for line in json_out.lines() {
        if let Some(d) = serde_json::from_str::<GrepJsonOut>(line).ok() {
            marked_positions.push(d.data.absolute_offset);
        }
    }

    Ok(marked_positions)
}

#[test]
fn test_json() {
    let marked = mark_matched_positions_with_ripgrep("^ITEM: NUMBER OF ATOMS", "./tests/files/lammps-test.dump".as_ref(), None).unwrap();
    assert_eq!(marked.len(), 3);
}
// 88a60571 ends here
