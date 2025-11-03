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

use crate::{
    args::GlobalOptions,
    commands::check::args::CheckParams,
    enums::CheckErrorDetail,
    functions::check::check_test,
    working_set::WorkingSet,
};
use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    fs,
    io::Cursor,
    path::PathBuf,
};

use crate::functions::check::check_metadata;
use anyhow::Error;
use moo::prelude::*;
use rayon::prelude::*;

#[derive(Debug, Default)]
struct CheckStats {
    files_checked: usize,
    tests_checked: usize,
    files_with_errors: usize,
    errors_found: usize,
    read_errors: usize,
    test_errors: HashMap<PathBuf, Vec<CheckErrorDetail>>,
}

impl CheckStats {
    fn combine(mut self, other: CheckStats) -> CheckStats {
        self.files_checked += other.files_checked;
        self.tests_checked += other.tests_checked;
        self.files_with_errors += other.files_with_errors;
        self.read_errors += other.read_errors;
        self.errors_found += other.errors_found;
        // Merge test errors
        for (pb, v_other) in other.test_errors {
            self.test_errors
                .entry(pb)
                .and_modify(|v_self| {
                    // append other's errors into existing vector (moves items, no clones)
                    v_self.extend(v_other.clone());
                })
                .or_insert(v_other); // no existing entry: just insert whole detail
        }
        self
    }
}

pub fn run(_global: &GlobalOptions, params: &CheckParams) -> Result<(), Error> {
    let working_set = WorkingSet::from_path(&params.in_path, None)?;

    if working_set.is_empty() {
        return Err(Error::msg("No files selected"));
    }

    let check_stats = working_set
        .par_iter()
        .map(|path| {
            let mut s = CheckStats {
                files_checked: 1,
                ..Default::default()
            };

            match fs::read(path) {
                Ok(data) => {
                    let mut reader = Cursor::new(data);
                    match MooTestFile::read(&mut reader) {
                        Ok(mut moo) => {
                            let metadata = match moo.metadata_mut() {
                                Some(md) => {
                                    let md_errors = check_metadata(md, path, params.fix);
                                    if !md_errors.is_empty() {
                                        s.read_errors += 1;
                                        s.files_with_errors = 1;
                                        s.test_errors
                                            .entry(path.clone())
                                            .or_default()
                                            .push(CheckErrorDetail::FileError(md_errors));
                                    }

                                    md.clone()
                                }
                                None => {
                                    log::warn!("MOO file {} is missing metadata chunk", path.display());
                                    s.read_errors += 1;
                                    s.files_with_errors = 1;
                                    return s;
                                }
                            };

                            for (ti, test) in moo.tests_mut().iter_mut().enumerate() {
                                match check_test(ti, test, &metadata, params) {
                                    Ok(Some(detail)) => {
                                        // Record error
                                        s.errors_found += 1; // counting failing tests
                                        s.files_with_errors = 1;
                                        s.test_errors.entry(path.clone()).or_default().push(detail);
                                    }
                                    Ok(None) => {
                                        // No error
                                    }
                                    Err(_) => {
                                        // Ignore test check errors for now
                                    }
                                }
                            }

                            s.tests_checked = moo.test_ct();

                            // Write fixed file if needed
                            let tests_fixed = s
                                .test_errors
                                .values()
                                .flat_map(|v| v.iter())
                                .map(|d| d.errors().iter().filter(|e| e.fixed).count())
                                .sum::<usize>();

                            if params.fix && tests_fixed > 0 {
                                let out_path = get_fixed_path(path, params);
                                let mut out_file = fs::File::create(out_path).unwrap();

                                // Set compression flag
                                moo.set_compressed(params.compress);

                                match moo.write(&mut out_file, true) {
                                    Ok(_) => {
                                        log::info!("Wrote fixed file for {}", path.display());
                                    }
                                    Err(e) => {
                                        log::error!("Error writing fixed file for {}: {}", path.display(), e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!("Parse error in {}: {}", path.display(), e);
                            s.read_errors += 1;
                            s.files_with_errors = 1;
                        }
                    }
                }
                Err(e) => {
                    log::warn!("I/O error reading {}: {}", path.display(), e);
                    s.read_errors += 1;
                    s.files_with_errors = 1;
                }
            }

            s
        })
        .reduce(CheckStats::default, CheckStats::combine);

    // Sort and print errors
    let mut sorted_errors: Vec<(&PathBuf, &Vec<CheckErrorDetail>)> = check_stats.test_errors.iter().collect();

    sorted_errors.sort_by(|(id_a, _), (id_b, _)| {
        let path_cmp = id_a.cmp(&id_b);
        if path_cmp == std::cmp::Ordering::Equal {
            id_a.cmp(&id_b)
        }
        else {
            path_cmp
        }
    });

    for (test_path, details) in sorted_errors {
        println!("Errors in file {}:", test_path.display());
        for err in details {
            match err {
                CheckErrorDetail::FileError(errors) => {
                    println!("  File-level errors:");
                    for e in errors {
                        println!("    - {}", e.e_type);
                    }
                }
                CheckErrorDetail::TestError { index, hash, errors } => {
                    println!("  Test {} | {}:", index, hash);
                    for err in errors {
                        println!("    - {}", err.e_type);
                        if err.fixed {
                            println!("    - (successfully fixed)");
                        }
                    }
                }
            }
        }
    }

    // Get total error count
    let total_errors = check_stats.test_errors.values().map(|v| v.len()).sum::<usize>();
    let total_fixed = check_stats
        .test_errors
        .values()
        .flat_map(|v| v.iter())
        .map(|d| d.errors().iter().filter(|e| e.fixed).count())
        .sum::<usize>();

    // report summary
    println!(
        "Checked {} files containing {} tests:",
        check_stats.files_checked, check_stats.tests_checked,
    );

    println!(
        "  {}/{} files contained 1 or more error(s)",
        check_stats.files_with_errors, check_stats.files_checked
    );

    println!(
        "  {} total errors found. {} tests with errors, {} file read errors.",
        total_errors,
        check_stats.test_errors.len(), // number of failing tests recorded
        check_stats.read_errors
    );

    println!("  {}/{} errors reported fixed.", total_fixed, total_errors);

    Ok(())
}

pub fn get_fixed_path(original: &PathBuf, params: &CheckParams) -> PathBuf {
    //let parent = original.parent().unwrap();
    let filename = original.file_stem().unwrap();
    let extension = original.extension().unwrap_or_else(|| OsStr::new("MOO"));

    if extension == "gz" && !params.compress {
        // Special case: original file is .MOO.gz, but we are not compressing output
        let filename = OsStr::new(filename);
        let filename = PathBuf::from(filename);
        let filename = filename.file_stem().unwrap();
        return params
            .out_path
            .as_ref()
            .unwrap()
            .join(join_filename_ext(filename, OsStr::new("MOO")));
    }

    let out_path = params.out_path.as_ref().unwrap().clone();
    out_path.join(join_filename_ext(filename, extension))
}

fn join_filename_ext(filename: &OsStr, extension: &OsStr) -> OsString {
    let mut result = OsString::from(filename);
    result.push(".");
    result.push(extension);
    result
}
