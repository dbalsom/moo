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
pub mod comparison;
pub mod cycles;
pub mod effective_address;
pub mod errors;
pub mod flags;
pub mod metadata;
pub mod ram;

use std::fmt::Display;

pub use comparison::*;
pub use cycles::*;
pub use metadata::*;
pub use ram::*;

pub use test::{moo_test::MooTest, test_state::MooTestState};

use crate::test;
use binrw::binrw;

/// [MooCpuType] represents the type of CPU used to produce a particular collection of [MooTestFile](crate::prelude::MooTestFile).
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
    Harris80C286,
    Intel80386Ex,
    Intel80286,
}

#[derive(Copy, Clone, Debug)]
/// [MooCpuFamily] represents the family of CPU types, when a more specific type is not required.
pub enum MooCpuFamily {
    Intel8086,
    NecV30,
    Intel80186,
    Intel80286,
    Intel80386,
}

impl From<MooCpuType> for MooCpuFamily {
    fn from(cpu_type: MooCpuType) -> Self {
        match cpu_type {
            MooCpuType::Intel8088 | MooCpuType::Intel8086 => MooCpuFamily::Intel8086,
            MooCpuType::NecV20 | MooCpuType::NecV30 => MooCpuFamily::NecV30,
            MooCpuType::Intel80188 | MooCpuType::Intel80186 => MooCpuFamily::Intel80186,
            MooCpuType::Intel80286 | MooCpuType::Harris80C286 => MooCpuFamily::Intel80286,
            MooCpuType::Intel80386Ex => MooCpuFamily::Intel80386,
        }
    }
}

/// [MooCpuMode] represents the operating mode of the CPU used to produce a particular [MooTestFile](crate::prelude::MooTestFile).
/// This affects how certain instructions behave, especially on 80286 and later CPUs.
#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[binrw]
#[br(repr(u8))]
#[bw(repr(u8))]
pub enum MooCpuMode {
    #[default]
    RealMode,
    ProtectedMode,
    Virtual8086Mode,
    UnrealMode,
}

/// The [MooStateType] enum represents whether a [MooTestState] is the initial or final state in a test.
#[derive(Copy, Clone, Debug, Default)]
pub enum MooStateType {
    #[default]
    /// The initial CPU state before a test is executed.
    Initial,
    /// The final CPU state after a test has been executed.
    Final,
}

/// [MooCpuDataBusWidth] represents the native bus size of a CPU.
#[derive(Copy, Clone, Debug, Default)]
pub enum MooCpuDataBusWidth {
    #[default]
    /// An 8-bit data bus.
    Eight,
    /// A 16-bit data bus.
    Sixteen,
}

impl From<MooCpuType> for MooCpuDataBusWidth {
    /// Convert a [MooCpuType] to its corresponding [MooCpuDataBusWidth].
    fn from(cpu_type: MooCpuType) -> Self {
        match cpu_type {
            MooCpuType::Intel8088 | MooCpuType::NecV20 | MooCpuType::Intel80188 => MooCpuDataBusWidth::Eight,
            _ => MooCpuDataBusWidth::Sixteen,
        }
    }
}

/// [MooDataWidth] represents the active width of a data bus.
/// On 16-bit buses, this can be the full 16-bits, or either 8-bit halves (high or low).
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
    /// Interrupt Acknowledge
    INTA = 0,
    /// IO Read
    IOR = 1,
    /// IO Write
    IOW = 2,
    /// Halt
    HALT = 3,
    /// Code Fetch
    CODE = 4,
    // Memory Read
    MEMR = 5,
    // Memory Write
    MEMW = 6,
    // Passive state (no activity)
    PASV = 7,
}

/// Display implementation for MooBusState.
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

/// [MooIvtOrder] represents the order of operations performed by a CPU when an interrupt table
/// vector is accessed.
#[derive(Copy, Clone, Debug)]
pub enum MooIvtOrder {
    /// The CPU reads the vector from memory before pushing the current IP/CS to the stack.
    ReadFirst,
    /// The CPU pushes the current IP/CS to the stack before reading the vector from memory.
    PushFirst,
}

impl From<MooCpuType> for MooIvtOrder {
    /// Convert a [MooCpuType] to its corresponding [MooIvtOrder].
    fn from(cpu_type: MooCpuType) -> Self {
        match cpu_type {
            MooCpuType::Intel80286 => MooIvtOrder::PushFirst,
            _ => MooIvtOrder::ReadFirst,
        }
    }
}

/// [MooTState] represents the T-state of the CPU.
#[derive(Copy, Clone, PartialEq)]
pub enum MooTState {
    /// Idle T-state, when a bus cycle is not in progress.
    Ti,
    /// First T-state of a bus cycle. ALE or ADS is asserted on this cycle.
    T1,
    /// Second T-state of a bus cycle.
    T2,
    /// Third T-state of a bus cycle (for 8086/88, 80186/88, V20/30).
    T3,
    /// Fourth T-state of a bus cycle (for 8086/88, 80186/88, V20/30).
    T4,
    /// Wait state. May occur between T3 and T4.
    Tw,
}

/// Try to convert a raw u8 value to a [MooTState].
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
    /// Returns the number of characters to use when displaying this CPU's address bus in cycle logs.
    pub fn bus_chr_width(&self) -> usize {
        use MooCpuType::*;
        match self {
            Harris80C286 | Intel80286 => 6,
            Intel80386Ex => 6,
            _ => 5,
        }
    }

    /// Returns the number of characters to use when displaying this CPU's data bus in cycle logs.
    pub fn data_chr_width(&self) -> usize {
        use MooCpuType::*;
        match self {
            Intel8088 | NecV20 => 2,
            _ => 4,
        }
    }

    /// Convert a string representation of a CPU type to a [MooCpuType].
    pub fn from_str(str: &str) -> Result<MooCpuType, String> {
        match str {
            "286 " => Ok(MooCpuType::Intel80286),
            "C286" => Ok(MooCpuType::Harris80C286),
            "386E" => Ok(MooCpuType::Intel80386Ex),
            "8088" => Ok(MooCpuType::Intel8088),
            "8086" => Ok(MooCpuType::Intel8086),
            "188 " => Ok(MooCpuType::Intel80188),
            "186 " => Ok(MooCpuType::Intel80186),
            "V20 " => Ok(MooCpuType::NecV20),
            "V30 " => Ok(MooCpuType::NecV30),
            _ => Err(format!("Unknown CPU type: {:?}", str)),
        }
    }

    /// Convert a [MooCpuType] to its static string representation.
    pub fn to_str(&self) -> &str {
        use MooCpuType::*;
        match self {
            Intel80286 => "286 ",
            Harris80C286 => "C286",
            Intel80386Ex => "386E",
            Intel8088 => "8088",
            Intel8086 => "8086",
            Intel80188 => "188 ",
            Intel80186 => "186 ",
            NecV20 => "V20 ",
            NecV30 => "V30 ",
        }
    }

    /// Convert a [MooTState] to its string representation for this CPU type.
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

    /// Decode a raw CPU bus status byte into a [MooBusState] for this CPU type.
    pub fn decode_status(&self, status_byte: u8) -> MooBusState {
        use MooBusState::*;
        use MooCpuFamily::*;
        let family = MooCpuFamily::from(*self);
        match family {
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
            Intel80386 => match status_byte & 0x07 {
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

    /// Return the masked raw bus status byte for this CPU type.
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
                | MooCpuType::Harris80C286
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
                | MooCpuType::Harris80C286
                | MooCpuType::Intel80386Ex
        )
    }

    /// Return true if the CPU is an NEC V20 or V30.
    pub fn is_nec(&self) -> bool {
        matches!(self, MooCpuType::NecV20 | MooCpuType::NecV30)
    }
}

// #[derive(Clone, Debug)]
// #[binrw]
// #[brw(little)]
// pub struct MooDate {
//     pub year: u16,
//     pub month: u8,
//     pub day: u8,
//     pub hour: u8,
//     pub minute: u8,
//     pub second: u8,
//     pub millisecond: u16,
// }

/// A [MooException] represents the `EXCP` chunk in a MOO file and contains information about the
/// exception that a test execution may have triggered.
#[derive(Clone, Debug)]
#[binrw]
#[brw(little)]
pub struct MooException {
    /// The exception number that was triggered.
    pub exception_num: u8,
    /// The address of the flag register pushed to the stack by the exception handler.
    pub flag_address:  u32,
}

/// A [MooSegmentSize] represents the native size of a segment.
/// This is only relevant for the 80386 family, as earlier CPUs only support 16-bit segments.
#[derive(Clone, Debug)]
pub enum MooSegmentSize {
    Sixteen,
    ThirtyTwo,
}

impl MooSegmentSize {
    /// Return the effective operand size of an instruction for this segment size, given whether an
    /// operand size override prefix is present.
    pub fn operand_size(&self, has_operand_override: bool) -> MooOperandSize {
        match (self, has_operand_override) {
            (MooSegmentSize::Sixteen, false) => MooOperandSize::Sixteen,
            (MooSegmentSize::Sixteen, true) => MooOperandSize::ThirtyTwo,
            (MooSegmentSize::ThirtyTwo, false) => MooOperandSize::ThirtyTwo,
            (MooSegmentSize::ThirtyTwo, true) => MooOperandSize::Sixteen,
        }
    }
    /// Return the effective address size of an instruction for this segment size, given whether an
    /// address size override prefix is present.
    pub fn address_size(&self, has_address_override: bool) -> MooAddressSize {
        match (self, has_address_override) {
            (MooSegmentSize::Sixteen, false) => MooAddressSize::Sixteen,
            (MooSegmentSize::Sixteen, true) => MooAddressSize::ThirtyTwo,
            (MooSegmentSize::ThirtyTwo, false) => MooAddressSize::ThirtyTwo,
            (MooSegmentSize::ThirtyTwo, true) => MooAddressSize::Sixteen,
        }
    }
}

/// An enum representing the operand size of an instruction.
/// Only applicable to 80386 and later CPUs.
#[derive(Clone, Debug, Default)]
pub enum MooOperandSize {
    #[default]
    Sixteen,
    ThirtyTwo,
}

/// An enum representing the address size of an instruction.
/// Only applicable to 80386 and later CPUs.
#[derive(Clone, Debug, Default)]
pub enum MooAddressSize {
    #[default]
    Sixteen,
    ThirtyTwo,
}
