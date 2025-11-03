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

use binrw::binrw;

/// [MooRamEntries] is a collection of [MooRamEntry] items representing memory addresses and their
/// corresponding byte values. It maps to a `MOO` `RAM ` chunk.
#[derive(Clone, Debug, Default)]
#[binrw]
#[brw(little)]
pub struct MooRamEntries {
    pub entry_count: u32,
    #[br(count = entry_count)]
    pub entries: Vec<MooRamEntry>,
}

impl From<&[MooRamEntry]> for MooRamEntries {
    fn from(entries: &[MooRamEntry]) -> Self {
        Self {
            entry_count: entries.len() as u32,
            entries: entries.to_vec(),
        }
    }
}

impl MooRamEntries {
    /// Returns the number of entries in the [MooRamEntries] chunk.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns a slice of all [MooRamEntry] items in the [MooRamEntries] chunk.
    pub fn entries(&self) -> &[MooRamEntry] {
        &self.entries
    }

    /// Searches for a sequence of bytes within the [MooRamEntries] and returns the starting index
    /// if found, otherwise returns `None`.
    pub fn find(&self, bytes: &[u8]) -> Option<usize> {
        let values = self.entries.iter().map(|entry| entry.value).collect::<Vec<u8>>();

        let index = values
            .windows(bytes.len())
            .position(|window| window == bytes)
            .unwrap_or(usize::MAX);

        if index == usize::MAX {
            None
        }
        else {
            Some(index)
        }
    }

    /// Retrieves a vector of consecutive byte values starting from the specified index in the
    /// [MooRamEntries]. Consecutive bytes are defined as those with sequential addresses.
    pub fn get_consecutive_bytes(&self, start_index: usize) -> Vec<u8> {
        let mut out = Vec::new();
        let entries = &self.entries;

        // Guard for out-of-range start index
        if start_index >= entries.len() {
            return out;
        }

        // Start from the initial entry
        let mut idx = start_index;
        let mut prev_addr = entries[idx].address;
        out.push(entries[idx].value);
        idx += 1;

        // Collect subsequent bytes while addresses increment by 1
        while idx < entries.len() {
            let e = &entries[idx];
            if e.address != prev_addr.wrapping_add(1) {
                break;
            }
            out.push(e.value);
            prev_addr = e.address;
            idx += 1;
        }

        out
    }
}

/// A [MooRamEntry] represents a single memory address and its corresponding byte value.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[binrw]
#[brw(little)]
pub struct MooRamEntry {
    /// The memory address of the entry. Not all bits may be valid, depending on the CPU architecture.
    pub address: u32,
    /// The byte value stored at the memory address.
    pub value:   u8,
}
