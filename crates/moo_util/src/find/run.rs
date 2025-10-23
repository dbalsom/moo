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

use std::{
    fs,
    io::{BufReader, Cursor},
    path::PathBuf,
};

use crate::{args::GlobalOptions, find::args::FindParams, working_set::WorkingSet};
use anyhow::Error;
use moo::prelude::*;
use rayon::prelude::*;

#[derive(Debug)]
pub struct FindMatch {
    file:  PathBuf,
    index: usize,
}

#[derive(Debug, Default)]
struct SearchStats {
    searched: usize,
    errors:   usize,
    found:    Option<FindMatch>,
}

impl SearchStats {
    fn combine(mut self, other: SearchStats) -> SearchStats {
        self.searched += other.searched;
        self.errors += other.errors;
        // keep the first found match if any
        if self.found.is_none() {
            self.found = other.found;
        }
        self
    }
}

pub fn run(global: &GlobalOptions, params: &FindParams) -> Result<(), Error> {
    let working_set = WorkingSet::from_path(&params.in_path, None)?;

    if working_set.is_empty() {
        return Err(Error::msg("No files selected"));
    }

    let stats: SearchStats = working_set
        .par_iter()
        .map(|path| {
            let mut s = SearchStats {
                searched: 1,
                ..Default::default()
            };

            match fs::read(path) {
                Ok(data) => {
                    let mut reader = Cursor::new(data);
                    match MooTestFile::read(&mut reader) {
                        Ok(moo) => {
                            if let Some(hash) = &params.hash {
                                for (t_idx, test) in moo.tests().iter().enumerate() {
                                    if test.hash_string() == *hash {
                                        s.found = Some(FindMatch {
                                            file:  PathBuf::from(path),
                                            index: t_idx,
                                        });
                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!("Parse error in {}: {}", path.display(), e);
                            s.errors += 1;
                        }
                    }
                }
                Err(e) => {
                    log::warn!("I/O error reading {}: {}", path.display(), e);
                    s.errors += 1;
                }
            }

            s
        })
        .reduce(SearchStats::default, SearchStats::combine);

    // report summary
    match stats.found {
        Some(m) => {
            println!(
                "Found in {} at index {} (searched {} files, {} read errors)",
                m.file.display(),
                m.index,
                stats.searched,
                stats.errors
            );
        }
        None => {
            println!("No match in {} files ({} read errors)", stats.searched, stats.errors);
        }
    }

    Ok(())
}
