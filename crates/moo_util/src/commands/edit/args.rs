/*
    MOO-rs Copyright 2025 Daniel Balsom
    https://github.com/dbalsom/moo

    Permission is hereby granted, free of charge, to any person obtaining a
    copy of this software and associated documentation files (the “Software”),
    to deal in the Software without restriction, including without limitation
    the rights to use, copy, modify, merge, publish, distribute, sublicense,
    and/or sell copies of the Software, and to permit persons to whom the
    Software is furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in
    all copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
    FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
    DEALINGS IN THE SOFTWARE.
*/
use std::path::PathBuf;

use crate::args::{hash_parser, in_path_parser, in_schema_parser, out_path_parser};
use bpaf::{construct, Parser};

#[derive(Clone, Debug)]
pub(crate) struct EditParams {
    pub(crate) in_path: PathBuf,
    pub(crate) out_path: PathBuf,
    pub(crate) schema_path: Option<PathBuf>,
    pub(crate) hash: Option<String>,
    pub(crate) index: Option<usize>,
    pub(crate) add_global_mask: bool,
    pub(crate) compress: bool,
    pub(crate) trim: bool,
    pub(crate) set_major_version: Option<u8>,
    pub(crate) set_minor_version: Option<u8>,
    pub(crate) set_metadata_major_version: Option<u8>,
    pub(crate) set_metadata_minor_version: Option<u8>,
}

pub(crate) fn edit_parser() -> impl Parser<EditParams> {
    let in_path = in_path_parser();
    let out_path = out_path_parser();
    let schema_path = in_schema_parser().optional();
    let hash = hash_parser().optional();
    let index = bpaf::long("index")
        .help("Index of the test to edit")
        .argument("INDEX")
        .optional();

    let add_global_mask = bpaf::long("add-global-mask")
        .help("Add the global register mask from a schema to the tests")
        .switch();

    let compress = bpaf::long("compress").help("Compress the output file(s)").switch();
    let trim = bpaf::long("trim")
        .help("Trim test files to count specified in schema")
        .switch();

    let set_major_version = bpaf::long("set-major-version")
        .help("Set the major version of the test file")
        .argument::<u8>("MAJOR_VERSION")
        .optional();

    let set_minor_version = bpaf::long("set-minor-version")
        .help("Set the minor version of the test file")
        .argument::<u8>("MINOR_VERSION")
        .optional();

    let set_metadata_major_version = bpaf::long("set-metadata-major-version")
        .help("Set the major version of the test metadata")
        .argument::<u8>("METADATA_MAJOR_VERSION")
        .optional();

    let set_metadata_minor_version = bpaf::long("set-metadata-minor-version")
        .help("Set the minor version of the test metadata")
        .argument::<u8>("METADATA_MINOR_VERSION")
        .optional();

    construct!(EditParams {
        in_path,
        out_path,
        schema_path,
        hash,
        index,
        add_global_mask,
        compress,
        trim,
        set_major_version,
        set_minor_version,
        set_metadata_major_version,
        set_metadata_minor_version,
    })
    .guard(
        |p| {
            if p.add_global_mask {
                p.schema_path.is_some()
            }
            else {
                true
            }
        },
        "--schema must also be provided with the --add-global-mask option.",
    )
    .guard(
        |p| {
            if p.trim {
                p.schema_path.is_some()
            }
            else {
                true
            }
        },
        "--schema must also be provided with the --trim option.",
    )
}
