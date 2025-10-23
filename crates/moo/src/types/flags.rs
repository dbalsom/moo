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

/// [MooCpuFlag] represents the individual bits contained within an x86 CPU's FLAGS or EFLAGS
/// register.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum MooCpuFlag {
    /// Carry Flag
    CF = 0,
    /// Reserved bit, always 1 on all x86 CPUs
    Reserved0 = 1,
    /// Parity Flag
    PF = 2,
    // Reserved bit, always 0
    Reserved1 = 3,
    /// Auxiliary Carry Flag
    AF = 4,
    /// Reserved bit, always 1
    Reserved2 = 5,
    /// Zero Flag
    ZF = 6,
    /// Sign Flag
    SF = 7,
    /// Trap Flag
    TF = 8,
    /// Interrupt Enable Flag
    IF = 9,
    /// Direction Flag
    DF = 10,
    /// Overflow Flag
    OF = 11,
    /// Bit 0 of I/O Privilege Level
    IOPL0 = 12,
    /// Bit 1 of I/O Privilege Level
    IOPL1 = 13,
    /// Nested Task Flag
    NT = 14,
    /// Reserved
    Reserved3 = 15,
    /// Resume Flag
    RF = 16,
    /// Virtual-8086 Mode flag
    VM = 17,
}

impl MooCpuFlag {
    /// Convert a u8 bit index into a [MooCpuFlag] or return `None` if the index is out of range.
    pub fn from_bit(bit: u8) -> Option<Self> {
        match bit {
            0 => Some(MooCpuFlag::CF),
            1 => Some(MooCpuFlag::Reserved0),
            2 => Some(MooCpuFlag::PF),
            3 => Some(MooCpuFlag::Reserved1),
            4 => Some(MooCpuFlag::AF),
            5 => Some(MooCpuFlag::Reserved2),
            6 => Some(MooCpuFlag::ZF),
            7 => Some(MooCpuFlag::SF),
            8 => Some(MooCpuFlag::TF),
            9 => Some(MooCpuFlag::IF),
            10 => Some(MooCpuFlag::DF),
            11 => Some(MooCpuFlag::OF),
            12 => Some(MooCpuFlag::IOPL0),
            13 => Some(MooCpuFlag::IOPL1),
            14 => Some(MooCpuFlag::NT),
            15 => Some(MooCpuFlag::Reserved3),
            16 => Some(MooCpuFlag::RF),
            17 => Some(MooCpuFlag::VM),
            _ => None,
        }
    }
}

/// A representation of the difference between two flag registers.
#[derive(Clone, Default, Debug)]
pub struct MooCpuFlagsDiff {
    /// Flags that were modified and set in the final flag state.
    pub set: Vec<MooCpuFlag>,
    /// Flags that were modified and cleared in the final flag state.
    pub cleared: Vec<MooCpuFlag>,
    /// Flags that were unmodified and remain set in the final flag state.
    pub unmodified_set: Vec<MooCpuFlag>,
    /// Flags that were unmodified and remain cleared in the final flag state.
    pub unmodified_cleared: Vec<MooCpuFlag>,
}
