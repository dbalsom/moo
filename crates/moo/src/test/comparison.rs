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
use crate::types::MooRamEntry;

/// An enumeration of possible results when comparing two [MooTest]s.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MooComparison {
    /// The two [MooTest]s are equal.
    Equal,
    /// The two [MooTest]s differ in register values.
    RegisterMismatch,
    /// The two [MooTest]s differ in cycle count, with the differing values provided.
    CycleCountMismatch(usize, usize),
    /// The two [MooTest]s differ in cycle address, with the differing values provided.
    CycleAddressMismatch(u32, u32),
    /// The two [MooTest]s differ in bus state, with the differing values provided.
    CycleBusMismatch(u8, u8),
    /// The two [MooTest]s differ in memory address, with the differing entries provided.
    MemoryAddressMismatch(MooRamEntry, MooRamEntry),
    /// The two [MooTest]s differ in memory values, with the differing entries provided.
    MemoryValueMismatch(MooRamEntry, MooRamEntry),
    /// The two [MooTest]s differ in ALE signal state, with the cycle number and differing values provided.
    ALEMismatch(usize, bool, bool),
}
