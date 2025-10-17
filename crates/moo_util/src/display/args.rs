use crate::args::{hash_parser, in_file_parser};
use bpaf::{construct, Parser};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub(crate) struct DisplayParams {
    pub(crate) in_file: PathBuf,
    pub(crate) hash:    Option<String>,
    pub(crate) index:   Option<usize>,
}

pub(crate) fn display_parser() -> impl Parser<DisplayParams> {
    //let path = positional::<String>("PATH").help("Path to the file to dump");

    let in_file = in_file_parser();

    let hash = hash_parser().optional();
    let index = bpaf::long("index")
        .help("Index of the entry to display (for multi-entry files)")
        .argument("INDEX")
        .optional();

    construct!(DisplayParams { in_file, hash, index }).guard(
        |p| p.hash.is_some() || p.index.is_some(),
        "Either --hash or --index must be provided",
    )
}
