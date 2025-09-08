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

use crate::types::MooCpuType;
use binrw::binrw;

#[derive(Debug, Default)]
#[binrw]
#[brw(little)]
pub struct MooFileMetadata {
    pub set_version_major: u8,
    pub set_version_minor: u8,
    pub cpu_type: MooCpuType,
    pub opcode: u32,
    pub mnemonic: [u8; 8],
    pub test_ct: u32,
    pub file_seed: u64,
    pub flag_mask: u32,
}

impl MooFileMetadata {
    pub fn new(set_version_major: u8, set_version_minor: u8, cpu_type: MooCpuType, opcode: u32) -> Self {
        Self {
            set_version_major,
            set_version_minor,
            cpu_type,
            opcode,
            mnemonic: [' ' as u8; 8],
            ..Default::default()
        }
    }

    pub fn with_test_count(mut self, test_count: u32) -> Self {
        self.test_ct = test_count;
        self
    }
    pub fn with_file_seed(mut self, file_seed: u64) -> Self {
        self.file_seed = file_seed;
        self
    }
    pub fn with_flag_mask(mut self, flag_mask: u32) -> Self {
        self.flag_mask = flag_mask;
        self
    }
    pub fn with_mnemonic(mut self, mnemonic: String) -> Self {
        for c in self.mnemonic.iter_mut() {
            *c = ' ' as u8;
        }
        let mnemonic = mnemonic.into_bytes();
        let mnemonic_len = std::cmp::min(mnemonic.len(), 8);
        self.mnemonic[0..mnemonic_len].copy_from_slice(&mnemonic.as_slice()[0..mnemonic_len]);
        self
    }

    pub fn mnemonic(&self) -> String {
        String::from_utf8_lossy(&self.mnemonic).trim().to_string()
    }
}

#[derive(Clone, Debug)]
#[binrw]
#[brw(little)]
pub struct MooTestGenMetadata {
    pub seed:   u64,
    pub gen_ct: u16,
}
