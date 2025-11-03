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
    collections::HashMap,
    ffi::{OsStr, OsString},
    fs,
    io::Cursor,
    path::PathBuf,
};

use crate::{
    args::GlobalOptions,
    commands::edit::args::EditParams,
    enums::EditErrorDetail,
    functions::{add_masks::add_global_mask, trim::trim_test},
    schema_db::{EditSchemaRecord, SchemaDb},
    working_set::WorkingSet,
};
use anyhow::Error;
use moo::{prelude::MooTestFile, types::MooCpuType};
use rayon::iter::ParallelIterator;

#[derive(Debug, Default)]
struct EditStats {
    files_edited: usize,
    tests_edited: usize,
    files_with_errors: usize,
    read_errors: usize,
    test_errors: HashMap<PathBuf, Vec<EditErrorDetail>>,
}

impl EditStats {
    fn combine(mut self, other: EditStats) -> EditStats {
        self.files_edited += other.files_edited;
        self.tests_edited += other.tests_edited;
        self.files_with_errors += other.files_with_errors;
        self.read_errors += other.read_errors;
        // Merge edit errors
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

pub fn run(_global: &GlobalOptions, params: &EditParams) -> Result<(), Error> {
    let working_set = WorkingSet::from_path(&params.in_path, None)?;

    if working_set.is_empty() {
        return Err(Error::msg("No files selected"));
    }

    let mut load_schema = false;
    if params.add_global_mask || params.trim {
        load_schema = true;
    }

    let schema_db = if load_schema {
        // Load schema csv file
        let schema: SchemaDb<EditSchemaRecord> =
            SchemaDb::from_file(MooCpuType::Intel80386Ex, &params.schema_path.as_ref().unwrap())?;
        Some(schema)
    }
    else {
        None
    };

    let edit_stats = working_set
        .par_iter()
        .map(|path| {
            let mut s = EditStats {
                files_edited: 0,
                ..Default::default()
            };

            match fs::read(path) {
                Ok(data) => {
                    let mut reader = Cursor::new(data);
                    match MooTestFile::read(&mut reader) {
                        Ok(mut moo) => {
                            let metadata = match moo.metadata() {
                                Some(md) => md.clone(),
                                None => {
                                    log::warn!("MOO file {} is missing metadata chunk", path.display());
                                    s.read_errors += 1;
                                    s.files_with_errors = 1;
                                    return s;
                                }
                            };

                            // Do per-file edits here
                            if let Some(major_version) = params.set_major_version {
                                moo.set_version(Some(major_version), None);
                                s.files_edited = 1;
                            }
                            if let Some(minor_version) = params.set_minor_version {
                                moo.set_version(None, Some(minor_version));
                                s.files_edited = 1;
                            }

                            if params.add_global_mask {
                                match add_global_mask(&mut moo, &metadata, schema_db.as_ref().unwrap(), params) {
                                    Ok(edited) => {
                                        if edited {
                                            log::info!("Added global mask to file {}", path.display());
                                            s.files_edited = 1;
                                        }
                                    }
                                    Err(_) => {
                                        // TODO: handle error
                                    }
                                }
                            }

                            if params.trim {
                                match trim_test(&mut moo, &metadata, schema_db.as_ref().unwrap(), params) {
                                    Ok(edited) => {
                                        if edited {
                                            s.files_edited = 1;
                                        }
                                    }
                                    Err(_) => {
                                        // TODO: handle error
                                    }
                                }
                            }

                            for (ti, test) in moo.tests_mut().iter_mut().enumerate() {
                                // Do per-test edits here
                            }

                            // Write edited file if needed

                            if s.files_edited > 0 || s.tests_edited > 0 {
                                let out_path = get_edited_path(path, params);
                                let mut out_file = fs::File::create(out_path).unwrap();

                                // Set compression flag
                                moo.set_compressed(params.compress);

                                match moo.write(&mut out_file, true) {
                                    Ok(_) => {
                                        log::info!("Wrote edited file for {}", path.display());
                                    }
                                    Err(e) => {
                                        log::error!("Error writing edited file for {}: {}", path.display(), e);
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
        .reduce(EditStats::default, EditStats::combine);

    Ok(())
}

pub fn get_edited_path(original: &PathBuf, params: &EditParams) -> PathBuf {
    //let parent = original.parent().unwrap();
    let filename = original.file_stem().unwrap();
    let extension = original.extension().unwrap_or_else(|| OsStr::new("MOO"));

    if extension == "gz" && !params.compress {
        // Special case: original file is .MOO.gz, but we are not compressing output
        let filename = OsStr::new(filename);
        let filename = PathBuf::from(filename);
        let filename = filename.file_stem().unwrap();
        return params.out_path.join(join_filename_ext(filename, OsStr::new("MOO")));
    }

    let out_path = params.out_path.clone();
    out_path.join(join_filename_ext(filename, extension))
}

fn join_filename_ext(filename: &OsStr, extension: &OsStr) -> OsString {
    let mut result = OsString::from(filename);
    result.push(".");
    result.push(extension);
    result
}
