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
    io,
    io::Read,
    path::{Path, PathBuf},
};

#[cfg(feature = "gzip")]
use flate2::read::GzDecoder;
use once_cell::sync::Lazy;
use rayon::prelude::{IntoParallelIterator, IntoParallelRefIterator};
use regex::Regex;

pub static MOO_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\.moo(\.gz)?$").expect("valid regex"));

/// Collect files and read them one-by-one into an internal buffer.
///
/// Behavior:
/// - If `path` is a file, that single file is included (no regex check).
/// - If `path` is a directory, files in that directory (non-recursive)
///   whose *file names* match `pattern` are included.
/// - Files are sorted by file name (UTF-8) for deterministic iteration.
#[derive(Debug)]
pub struct WorkingSet {
    files:    Vec<PathBuf>,
    /// Index of the next file to read.
    next_idx: usize,
    /// Internal read buffer reused on each `read_next()`.
    buffer:   Vec<u8>,
}

impl WorkingSet {
    pub fn from_path<P: AsRef<Path>>(path: P, limit: Option<usize>) -> io::Result<Self> {
        WorkingSet::from_path_regex(path, None, limit)
    }

    /// Build a working set from a path and a regex.
    /// - Returns Ok even if no files match (the set will be empty).
    pub fn from_path_regex<P: AsRef<Path>>(path: P, pattern: Option<&Regex>, limit: Option<usize>) -> io::Result<Self> {
        let path = path.as_ref();

        let mut files = Vec::new();

        if path.is_file() {
            files.push(path.to_path_buf());
        }
        else if path.is_dir() {
            log::debug!("Supplied path is directory. Iterating through files...");
            let path_iter = path.read_dir()?;
            for entry in path_iter.take(limit.unwrap_or(usize::MAX)) {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => continue, // skip unreadable entries
                };
                let p = entry.path();

                if p.is_file() {
                    if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                        let working_pattern = pattern.unwrap_or(&*MOO_REGEX);
                        if working_pattern.is_match(name) {
                            log::debug!("Found MOO file: {}", p.display());
                            files.push(p);
                        }
                        else {
                            log::warn!("Ignoring file with unknown pattern: {}", p.display());
                        }
                    }
                }
            }
            // deterministic ordering by file name (fallback: full path)
            files.sort_by(|a, b| {
                let an = a.file_name().and_then(|s| s.to_str()).unwrap_or_default();
                let bn = b.file_name().and_then(|s| s.to_str()).unwrap_or_default();
                an.cmp(bn).then_with(|| a.cmp(b))
            });
        }

        Ok(Self {
            files,
            next_idx: 0,
            buffer: Vec::new(),
        })
    }

    /// Total number of files.
    pub fn total(&self) -> usize {
        self.files.len()
    }

    /// True if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Borrow the collected files.
    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }

    /// Consume and return the collected files.
    pub fn into_files(self) -> Vec<PathBuf> {
        self.files
    }

    /// Iterator over `&Path` (borrowing).
    pub fn iter(&self) -> impl Iterator<Item = &Path> {
        self.files.iter().map(|p| p.as_path())
    }

    /// Into-iterator over `PathBuf` (consuming).
    pub fn into_iter(self) -> impl Iterator<Item = PathBuf> {
        self.files.into_iter()
    }

    pub fn par_iter(&self) -> rayon::slice::Iter<'_, PathBuf> {
        use rayon::prelude::*;
        self.files.par_iter()
    }

    pub fn into_par_iter(self) -> rayon::vec::IntoIter<PathBuf> {
        use rayon::prelude::*;
        self.files.into_par_iter()
    }
}

impl<'a> IntoIterator for &'a WorkingSet {
    type Item = &'a Path;
    type IntoIter = std::iter::Map<std::slice::Iter<'a, PathBuf>, fn(&PathBuf) -> &Path>;

    fn into_iter(self) -> Self::IntoIter {
        fn map_ref(p: &PathBuf) -> &Path {
            p.as_path()
        }
        self.files.iter().map(map_ref as fn(&PathBuf) -> &Path)
    }
}

impl IntoIterator for WorkingSet {
    type Item = PathBuf;
    type IntoIter = std::vec::IntoIter<PathBuf>;

    fn into_iter(self) -> Self::IntoIter {
        self.files.into_iter()
    }
}

// Allow `ws.into_par_iter()` directly.
impl IntoParallelIterator for WorkingSet {
    type Iter = rayon::vec::IntoIter<PathBuf>;
    type Item = PathBuf;

    fn into_par_iter(self) -> Self::Iter {
        self.files.into_par_iter()
    }
}

// Allow `(&ws).into_par_iter()` (parallel over &PathBuf).
impl<'a> IntoParallelIterator for &'a WorkingSet {
    type Iter = rayon::slice::Iter<'a, PathBuf>;
    type Item = &'a PathBuf;

    fn into_par_iter(self) -> Self::Iter {
        self.files.par_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn empty_dir_ok() -> io::Result<()> {
        let tmp = tempfile::tempdir()?;
        let re = Regex::new(r"^.*\.json$").unwrap();
        let ws = WorkingSet::from_path_regex(tmp.path(), Some(&re), None)?;
        assert_eq!(ws.total(), 0);
        Ok(())
    }
}
