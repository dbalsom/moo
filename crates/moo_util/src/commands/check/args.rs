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

use crate::args::{hash_parser, in_path_parser, index_parser, out_path_parser};

use bpaf::{construct, Parser};

#[derive(Clone, Debug)]
pub(crate) struct CheckParams {
    pub(crate) in_path: PathBuf,
    pub(crate) out_path: Option<PathBuf>,
    pub(crate) hash: Option<String>,
    pub(crate) index: Option<usize>,
    pub(crate) fix: bool,
    pub(crate) compress: bool,
}

pub(crate) fn check_parser() -> impl Parser<CheckParams> {
    let in_path = in_path_parser();
    let out_path = out_path_parser().optional();
    let hash = hash_parser().optional();
    let index = index_parser().optional();
    let fix = bpaf::long("fix")
        .help("Automatically fix any detected issues where possible")
        .switch();
    let compress = bpaf::long("compress")
        .help("Compress the output file if --fix is specified")
        .switch();

    construct!(CheckParams {
        in_path,
        out_path,
        hash,
        index,
        fix,
        compress,
    })
    .guard(
        |p| {
            if p.fix {
                p.out_path.is_some()
            }
            else {
                true
            }
        },
        "--output is required if --fix is specified",
    )
}
