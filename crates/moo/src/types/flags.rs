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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum MooCpuFlag {
    CF = 0,         // Carry Flag
    Reserved0 = 1,  // Reserved
    PF = 2,         // Parity Flag
    Reserved1 = 3,  // Reserved
    AF = 4,         // Auxiliary Carry Flag
    Reserved2 = 5,  // Reserved
    ZF = 6,         // Zero Flag
    SF = 7,         // Sign Flag
    TF = 8,         // Trap Flag
    IF = 9,         // Interrupt Enable Flag
    DF = 10,        // Direction Flag
    OF = 11,        // Overflow Flag
    IOPL0 = 12,     // I/O Privilege Level (2 bits)
    IOPL1 = 13,     // I/O Privilege Level (2 bits)
    NT = 14,        // Nested Task
    Reserved3 = 15, // Reserved
    RF = 16,        // Resume Flag
    VM = 17,        // Virtual-8086 Mode
}

impl MooCpuFlag {
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

#[derive(Clone, Default, Debug)]
pub struct MooCpuFlagsDiff {
    pub set_flags: Vec<MooCpuFlag>,
    pub cleared_flags: Vec<MooCpuFlag>,
}
