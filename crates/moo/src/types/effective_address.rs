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

use crate::registers::MooSegmentRegister;

use binrw::binrw;

/// A [MooEffectiveAddress] represents the components of an effective address calculation in the
/// event that a test instruction has a ModR/M (or SIB) byte that specifies a memory address
/// operand.
#[derive(Clone, Debug)]
#[binrw]
#[brw(little)]
pub struct MooEffectiveAddress {
    /// The segment register used as the base for the effective address calculation.
    pub base_segment: MooSegmentRegister,
    /// The selector value of the segment register used as the base for the effective address.
    pub base_selector: u16,
    /// The base address from the segment register used as the base for the effective address.
    /// For real mode, this is typically the segment value shifted left by 4 bits.
    pub base_address: u32,
    /// The limit of the segment used as the base for the effective address. For real mode, this
    /// is 0xFFFF.
    pub base_limit: u32,
    /// The offset added to the base address to compute the effective address.
    pub offset: u32,
    /// The linear address computed from the base address and offset.
    pub linear_address: u32,
    /// The physical address computed from the linear address. In real mode, this is the same as
    /// the linear address.
    pub physical_address: u32,
}

impl MooEffectiveAddress {
    /// Creates a new [MooEffectiveAddress] for a real mode test instruction.
    /// # Arguments
    /// * `base_segment` - The segment register used as the base for the effective address calculation.
    /// * `base_selector` - The selector value of the segment register used as the base for the effective address.
    /// * `base_address` - The base address from the segment register used as the base for the effective address.
    /// * `base_limit` - The limit of the segment used as the base for the effective address.
    /// * `offset` - The offset added to the base address to compute the effective address.
    /// # Returns
    /// A new [MooEffectiveAddress].
    pub fn new_real(
        base_segment: MooSegmentRegister,
        base_selector: u16,
        base_address: u32,
        base_limit: u32,
        offset: u32,
    ) -> Self {
        let linear_address = base_address.wrapping_add(offset);
        Self {
            base_segment,
            base_selector,
            base_address,
            base_limit,
            offset,
            linear_address,
            physical_address: linear_address,
        }
    }
}
