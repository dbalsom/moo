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

//! The `moo-rs` crate is a library for working with **MOO** (Machine-Opcode Operation) test files
//! that typically encode [Single Step Tests](https://github.com/singleStepTests/) (SST)s intended
//! for testing CPU emulators.
//!
//! It provides functionality to read, parse, and manipulate MOO test files that contain
//! initial and final CPU states including the state of registers and memory.
//!
//! ## Features
#![doc = document_features::document_features!()]
//! # Using moo-rs
//!
//! The main data structure provided by this crate is the [MooTestFile](prelude::MooTestFile) struct
//! that represents a **MOO** test file.
//!
//! ## Reading a MOO file:
//!
//! The most common way to read a **MOO** file is to use the [MooTestFile::read](prelude::MooTestFile::read) method.
//! This method takes any type that implements the [Read](std::io::Read) and [Seek](std::io::Seek) traits.
//! [Seek](std::io::Seek) can be provided by wrapping a slice in a [std::io::Cursor].
//! For files, you will likely want to wrap the file in a [std::io::BufReader] for speed, or just read
//! the entire file into memory first.
//!
//! Once a **MOO** file is read, you can access the test cases via the [MooTestFile::tests](prelude::MooTestFile::tests)
//! method which returns a slice of [MooTest](prelude::MooTest) instances.

#![doc = include_str!("../../../doc/moo_format_v1.md")]

/// The maximum major version number of the MOO file format supported by this crate
pub const MOO_MAJOR_VERSION: u8 = 1;
/// The maximum minor version number of the MOO file format supported by this crate
pub const MOO_MINOR_VERSION: u8 = 1;

pub mod prelude;
pub mod registers;
mod test;
pub mod test_file;
pub mod types;
