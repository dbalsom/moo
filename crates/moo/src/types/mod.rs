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

pub mod chunks;
pub mod cycles;
pub mod effective_address;
pub mod errors;
pub mod flags;
pub mod metadata;
pub mod ram;
pub mod registers;
pub mod registers_16;
pub mod registers_32;
pub mod test;
pub mod test_state;

use binrw::binrw;
pub use cycles::*;
pub use metadata::*;
pub use ram::*;
pub use registers::*;
pub use registers_16::*;
pub use registers_32::*;
use std::fmt::Display;
pub use test::*;
pub use test_state::*;

/// [MooCpuType] represents the type of CPU used to produce a test case.
#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[binrw]
#[br(repr(u8))]
#[bw(repr(u8))]
pub enum MooCpuType {
    #[default]
    Intel8088,
    Intel8086,
    NecV20,
    NecV30,
    Intel80188,
    Intel80186,
    Intel80286,
    Intel80386Ex,
}

#[derive(Copy, Clone, Debug, Default)]
pub enum MooStateType {
    #[default]
    Initial,
    Final,
}

/// [MooCpuWidth] represents the native bus size of a CPU.
#[derive(Copy, Clone, Debug, Default)]
pub enum MooCpuWidth {
    #[default]
    Eight,
    Sixteen,
}

impl From<MooCpuType> for MooCpuWidth {
    fn from(cpu_type: MooCpuType) -> Self {
        match cpu_type {
            MooCpuType::Intel8088 | MooCpuType::NecV20 | MooCpuType::Intel80188 => MooCpuWidth::Eight,
            _ => MooCpuWidth::Sixteen,
        }
    }
}

/// [MooDataWidth] represents the current width of the data bus.
#[derive(Copy, Clone, Debug, Default)]
pub enum MooDataWidth {
    #[default]
    Invalid,
    /// The entire data bus is being driven.
    Sixteen,
    /// The low half of the data bus is being driven, A0 is even.
    EightLow,
    /// The low half of the data bus is being driven, A0 is odd.
    EightHigh,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MooBusState {
    INTA = 0, // IRQ Acknowledge
    IOR = 1,  // IO Read
    IOW = 2,  // IO Write
    HALT = 3, // Halt
    CODE = 4, // Code
    MEMR = 5, // Memory Read
    MEMW = 6, // Memory Write
    PASV = 7, // Passive
}

impl Display for MooBusState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use MooBusState::*;
        match self {
            INTA => write!(f, "INTA"),
            IOR => write!(f, "IOR "),
            IOW => write!(f, "IOW "),
            HALT => write!(f, "HALT"),
            CODE => write!(f, "CODE"),
            MEMR => write!(f, "MEMR"),
            MEMW => write!(f, "MEMW"),
            PASV => write!(f, "PASV"),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MooIvtOrder {
    ReadFirst,
    PushFirst,
}

impl From<MooCpuType> for MooIvtOrder {
    fn from(cpu_type: MooCpuType) -> Self {
        match cpu_type {
            MooCpuType::Intel80286 => MooIvtOrder::PushFirst,
            _ => MooIvtOrder::ReadFirst,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum MooTState {
    Ti,
    T1,
    T2,
    T3,
    T4,
    Tw,
}

impl TryFrom<u8> for MooTState {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MooTState::Ti),
            1 => Ok(MooTState::T1),
            2 => Ok(MooTState::T2),
            3 => Ok(MooTState::T3),
            4 => Ok(MooTState::T4),
            5 => Ok(MooTState::Tw),
            _ => Err(format!("Invalid bus cycle value: {}", value)),
        }
    }
}

impl MooCpuType {
    pub fn bus_chr_width(&self) -> usize {
        use MooCpuType::*;
        match self {
            Intel80286 => 6,
            Intel80386Ex => 6,
            _ => 5,
        }
    }

    pub fn data_chr_width(&self) -> usize {
        use MooCpuType::*;
        match self {
            Intel8088 | NecV20 => 2,
            _ => 4,
        }
    }

    /// Convert a string representation of a CPU type to a [MooCpuType].
    pub fn from_str(str: &str) -> Result<MooCpuType, String> {
        if (str == "286 ") || (str == "C286") {
            Ok(MooCpuType::Intel80286)
        }
        else if str == "386E" {
            Ok(MooCpuType::Intel80386Ex)
        }
        else if str == "8088" {
            Ok(MooCpuType::Intel8088)
        }
        else if str == "8086" {
            Ok(MooCpuType::Intel8086)
        }
        else if str == "188 " {
            Ok(MooCpuType::Intel80188)
        }
        else if str == "186 " {
            Ok(MooCpuType::Intel80186)
        }
        else if str == "V20 " {
            Ok(MooCpuType::NecV20)
        }
        else if str == "V30 " {
            Ok(MooCpuType::NecV30)
        }
        else {
            Err(format!("Unknown CPU type: {:?}", str))
        }
    }

    pub fn to_str(&self) -> &str {
        use MooCpuType::*;
        match self {
            Intel80286 => "286 ",
            Intel80386Ex => "386E",
            Intel8088 => "8088",
            Intel8086 => "8086",
            Intel80188 => "188 ",
            Intel80186 => "186 ",
            NecV20 => "V20 ",
            NecV30 => "V30 ",
        }
    }

    pub fn tstate_to_string(&self, state: MooTState) -> String {
        use MooTState::*;
        match self {
            MooCpuType::Intel80286 => match state {
                Ti => "Ti".to_string(),
                T1 => "Ts".to_string(),
                T2 => "Tc".to_string(),
                Tw => "Tw".to_string(),
                _ => "T?".to_string(),
            },
            _ => match state {
                Ti => "Ti".to_string(),
                T1 => "T1".to_string(),
                T2 => "T2".to_string(),
                T3 => "T3".to_string(),
                T4 => "T4".to_string(),
                Tw => "Tw".to_string(),
            },
        }
    }

    pub fn decode_status(&self, status_byte: u8) -> MooBusState {
        use MooBusState::*;
        use MooCpuType::*;
        match self {
            Intel80286 => match status_byte & 0x0F {
                0b0000 => INTA,
                0b0001 => PASV, // Reserved
                0b0010 => PASV, // Reserved
                0b0011 => PASV, // None
                0b0100 => HALT,
                0b0101 => MEMR,
                0b0110 => MEMW,
                0b0111 => PASV, // None
                0b1000 => PASV, // Reserved
                0b1001 => IOR,
                0b1010 => IOW,
                0b1011 => PASV, // None
                0b1100 => PASV, // Reserved
                0b1101 => CODE,
                0b1110 => PASV, // Reserved
                0b1111 => PASV, // None
                _ => PASV,      // Default to passive state
            },
            Intel80386Ex => match status_byte & 0x07 {
                0 => INTA, // IRQ Acknowledge
                1 => PASV, // Passive
                2 => IOR,  // IO Read
                3 => IOW,  // IO Write
                4 => CODE, // Code fetch
                5 => HALT, // Halt
                6 => MEMR, // Memory Read
                _ => MEMW, // Memory Write
            },
            _ => {
                match status_byte & 0x07 {
                    0 => INTA, // IRQ Acknowledge
                    1 => IOR,  // IO Read
                    2 => IOW,  // IO Write
                    3 => HALT, // Halt
                    4 => CODE, // Code fetch
                    5 => MEMR, // Memory Read
                    6 => MEMW, // Memory Write
                    _ => PASV, // Passive state
                }
            }
        }
    }

    pub fn raw_status(&self, status_byte: u8) -> u8 {
        match self {
            MooCpuType::Intel80286 => status_byte & 0x0F,
            _ => status_byte & 0x07,
        }
    }

    /// Return the numeric bit width of the CPU data bus (8 or 16).
    pub fn bus_bitness(&self) -> u32 {
        if self.has_16bit_bus() {
            16
        }
        else {
            8
        }
    }

    /// Return the numeric bit width of the CPU registers (16 or 32).
    pub fn reg_bitness(&self) -> u32 {
        if self.has_32bit_regs() {
            32
        }
        else {
            16
        }
    }

    /// Return true if the CPU has 32-bit registers.
    pub fn has_32bit_regs(&self) -> bool {
        matches!(self, MooCpuType::Intel80386Ex)
    }

    /// Return true if the CPU has a native 16-bit data bus.
    pub fn has_16bit_bus(&self) -> bool {
        matches!(
            self,
            MooCpuType::Intel8086
                | MooCpuType::Intel80186
                | MooCpuType::Intel80286
                | MooCpuType::NecV30
                | MooCpuType::Intel80386Ex
        )
    }

    /// Return true if the CPU has a native 8-bit data bus.
    pub fn has_8bit_bus(&self) -> bool {
        matches!(
            self,
            MooCpuType::Intel8088 | MooCpuType::Intel80188 | MooCpuType::NecV20
        )
    }

    /// Return true if the CPU is an Intel CPU (or authorized 2nd source).
    pub fn is_intel(&self) -> bool {
        matches!(
            self,
            MooCpuType::Intel8088
                | MooCpuType::Intel8086
                | MooCpuType::Intel80188
                | MooCpuType::Intel80186
                | MooCpuType::Intel80286
                | MooCpuType::Intel80386Ex
        )
    }

    /// Return true if the CPU is an NEC V20 or V30.
    pub fn is_nec(&self) -> bool {
        matches!(self, MooCpuType::NecV20 | MooCpuType::NecV30)
    }
}

#[derive(Clone, Debug)]
#[binrw]
#[brw(little)]
pub struct MooDate {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub millisecond: u16,
}

#[derive(Clone, Debug)]
#[binrw]
#[brw(little)]
pub struct MooException {
    pub exception_num: u8,
    pub flag_address:  u32,
}
