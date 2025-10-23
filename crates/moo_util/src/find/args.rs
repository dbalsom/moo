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

use crate::args::{hash_parser, in_path_parser};
use bpaf::{construct, Parser};

#[derive(Clone, Debug)]
pub(crate) struct FindParams {
    pub(crate) in_path: PathBuf,
    pub(crate) hash:    Option<String>,
}

pub(crate) fn find_parser() -> impl Parser<FindParams> {
    //let path = positional::<String>("PATH").help("Path to the file to dump");

    let in_path = in_path_parser();

    let hash = hash_parser().optional();

    construct!(FindParams { in_path, hash }).guard(|p| p.hash.is_some(), "--hash must be provided")
}
