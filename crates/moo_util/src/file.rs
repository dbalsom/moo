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

use std::path::Path;

pub fn group_extension_from_path(path: impl AsRef<Path>) -> Option<u8> {
    path.as_ref().file_name().and_then(|os| os.to_str()).and_then(|name| {
        name.split('.').find_map(|part| {
            if part.len() == 1 {
                part.chars().next().and_then(|c| c.to_digit(10)).map(|d| d as u8)
            }
            else {
                None
            }
        })
    })
}

#[cfg(test)]
mod tests {
    use super::group_extension_from_path;
    use std::path::Path;

    #[test]
    fn returns_none_for_no_digit_part() {
        assert_eq!(group_extension_from_path(Path::new("00.MOO")), None);
        assert_eq!(group_extension_from_path(Path::new("00.MOO.gz")), None);
    }

    #[test]
    fn returns_some_for_single_digit_part() {
        assert_eq!(group_extension_from_path(Path::new("D2.1.MOO")), Some(1));
        assert_eq!(group_extension_from_path(Path::new("D3.4.MOO.gz")), Some(4));
    }

    #[test]
    fn ignores_multi_digit_parts() {
        assert_eq!(group_extension_from_path(Path::new("file.12.MOO")), None);
        assert_eq!(group_extension_from_path(Path::new("a.123.b")), None);
    }

    #[test]
    fn returns_first_single_digit_part() {
        // The first single-digit part is "7", even though "3" appears later.
        assert_eq!(group_extension_from_path(Path::new("X.7.3.MOO")), Some(7));
    }

    #[test]
    fn works_with_directory_components() {
        assert_eq!(group_extension_from_path(Path::new("/some/path/D3.4.MOO.gz")), Some(4));
    }
}
