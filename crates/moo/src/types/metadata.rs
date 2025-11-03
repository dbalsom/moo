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

use crate::types::{MooCpuMode, MooCpuType};
use binrw::binrw;

/// A [MooFileMetadata] struct represents the metadata header for a `MOO` test file.
#[derive(Clone, Debug, Default)]
#[binrw]
#[brw(little)]
pub struct MooFileMetadata {
    /// The major version of the MOO test file collection this file belongs to.
    pub set_version_major: u8,
    /// The minor version of the MOO test file collection this file belongs to.
    pub set_version_minor: u8,
    /// The CPU type the tests in this file are designed for. This enum can be more specific than
    /// the CPU architecture string found in a [MooFileHeader](crate::types::chunks::MooFileHeader).
    pub cpu_type: MooCpuType,
    /// The opcode of the instruction being tested in this file.
    /// This is stored as a u32 to accommodate multibyte opcodes, but is typically no longer than
    /// 16 bits.
    pub opcode: u32,
    /// The ASCII-encoded mnemonic string of the instruction being tested in this file, padded
    /// with spaces.
    pub mnemonic: [u8; 8],
    /// The number of tests contained in this file.
    pub test_ct: u32,
    /// The seed value used for generating the tests in this file.
    pub file_seed: u64,
    /// The CPU mode of the tests in the test file, stored as a byte value.
    pub cpu_mode: MooCpuMode,
    /// The group extension of the instruction being tested in this file, if applicable.
    pub extension: u8,
    pub reserved: [u8; 2],
}

impl MooFileMetadata {
    /// Create a new [MooFileMetadata] with the specified parameters.
    /// # Arguments
    /// * `set_version_major` - The major version of the MOO test file collection this file belongs to.
    /// * `set_version_minor` - The minor version of the MOO test file collection this file belongs to.
    /// * `cpu_type` - The CPU type the tests in this file are designed for.
    /// * `opcode` - The opcode of the instruction being tested in this file.
    /// # Returns
    /// A new [MooFileMetadata] instance with the specified parameters and default values for
    /// other fields.
    pub fn new(
        set_version_major: u8,
        set_version_minor: u8,
        cpu_type: MooCpuType,
        opcode: u32,
        extension: Option<u8>,
    ) -> Self {
        Self {
            set_version_major,
            set_version_minor,
            cpu_type,
            opcode,
            extension: extension.unwrap_or(0xFF),
            mnemonic: [' ' as u8; 8],
            ..Default::default()
        }
    }

    /// Builder-style method to set the test count of the [MooFileMetadata].
    /// # Arguments
    /// * `test_count` - The number of tests contained in this file.
    pub fn with_test_count(mut self, test_count: u32) -> Self {
        self.test_ct = test_count;
        self
    }

    /// Builder-style method to set the file seed of the [MooFileMetadata].
    /// # Arguments
    /// * `file_seed` - The seed value used for generating the tests in this file
    pub fn with_file_seed(mut self, file_seed: u64) -> Self {
        self.file_seed = file_seed;
        self
    }
    /// Builder-style method to set the CPU mode of the [MooFileMetadata].
    /// # Arguments
    /// * `cpu_mode` - The CPU mode of the tests in the test file.
    pub fn with_cpu_mode(mut self, cpu_mode: MooCpuMode) -> Self {
        self.cpu_mode = cpu_mode;
        self
    }
    /// Builder-style method to set the mnemonic string of the [MooFileMetadata].
    /// # Arguments
    /// * `mnemonic` - The ASCII-encoded mnemonic string of the instruction being tested in this file.
    pub fn with_mnemonic(mut self, mnemonic: String) -> Self {
        for c in self.mnemonic.iter_mut() {
            *c = ' ' as u8;
        }
        let mnemonic = mnemonic.into_bytes();
        let mnemonic_len = std::cmp::min(mnemonic.len(), 8);
        self.mnemonic[0..mnemonic_len].copy_from_slice(&mnemonic.as_slice()[0..mnemonic_len]);
        self
    }

    /// Get the mnemonic string of the [MooFileMetadata].
    pub fn mnemonic(&self) -> String {
        String::from_utf8_lossy(&self.mnemonic).trim().to_string()
    }

    pub fn group_extension(&self) -> Option<u8> {
        if self.extension == 0xFF {
            None
        }
        else {
            Some(self.extension)
        }
    }

    pub fn set_group_extension(&mut self, extension: Option<u8>) {
        self.extension = extension.unwrap_or(0xFF);
    }
}

/// A [MooTestGenMetadata] struct represents the test generation metadata for a `MOO` test file.
/// This chunk and struct are considered for internal use only by a `MOO` test generator / validator.
/// It is subject to change at any time.
#[derive(Clone, Debug)]
#[binrw]
#[brw(little)]
pub struct MooTestGenMetadata {
    /// The per-test seed value used for generating this test.
    pub seed:   u64,
    /// The number of generations (attempts) it took to create this test.
    pub gen_ct: u16,
}
