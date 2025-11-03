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

use crate::types::{MooBusState, MooCpuDataBusWidth, MooCpuType, MooDataWidth, MooTState};
use binrw::binrw;
use std::fmt::Display;

/// A [MooCycleState] represents the state of the CPU during a single clock cycle, capturing the
/// address and data buses, memory and I/O status, bus state, and the state of various CPU pins.
///
/// This struct corresponds to the payload of a `CYCL` chunk in a `MOO` test file.
#[derive(Copy, Clone, Debug, Default)]
#[binrw]
#[brw(little)]
pub struct MooCycleState {
    /// The main pin status bitfield for this cycle.
    /// See the PIN_* constants for bit definitions.
    pub pins0: u8,
    pub address_bus: u32,
    /// The raw segment status bits for this cycle. Only valid if the CPU architecture uses segment
    /// status pins.
    pub segment: u8,
    /// The memory RW status bitfield for this cycle.
    pub memory_status: u8,
    /// The I/O RW status bitfield for this cycle.
    pub io_status: u8,
    pub pins1: u8,
    /// The contents of the data bus during this cycle. For CPUs with an 8-bit data bus, only the
    /// lower 8 bits are valid. For CPUs with a 16-bit data bus, the upper, lower, or both bytes
    /// may be valid depending on the value of A0 and the BHE pin.
    pub data_bus: u16,
    /// The raw bus state byte for this cycle. This value is decoded based on the CPU type to
    /// determine the actual [MooBusState].
    pub bus_state: u8,
    /// The raw T-state value for this cycle. This value is decoded to determine the actual
    /// [MooTState].
    pub t_state: u8,
    /// The instruction queue operation for this cycle. Only valid if a CPU architecture has a
    /// queue status lines.
    pub queue_op: u8,
    /// The byte read from the queue during this cycle, if the queue operation indicates a read
    /// from the queue. Otherwise, this value is undefined.
    pub queue_byte: u8,
}

impl MooCycleState {
    /// A constant mask for the ALE (Address Latch Enable) pin in the pins0 field.
    pub const PIN_ALE: u8 = 0b0000_0001;
    /// A constant mask for the BHE (Bus High Enable) pin in the pins0 field.
    pub const PIN_BHE: u8 = 0b0000_0010;
    /// A constant mask for the READY pin in the pins0 field.
    pub const PIN_READY: u8 = 0b0000_0100;
    /// A constant mask for the LOCK pin in the pins0 field.
    pub const PIN_LOCK: u8 = 0b0000_1000;

    /// A constant mask for the MRDC (Memory Read) bit in the memory_status field.
    pub const MRDC_BIT: u8 = 0b0000_0100;
    /// A constant mask for the AMWC (Advanced Memory Write) bit in the memory_status field.
    pub const AMWC_BIT: u8 = 0b0000_0010;
    /// A constant mask for the MWTC (Memory Write) bit in the memory_status field.
    pub const MWTC_BIT: u8 = 0b0000_0001;

    /// A constant mask for the IORC (I/O Read) bit in the io_status field.
    pub const IORC_BIT: u8 = 0b0000_0100;
    /// A constant mask for the AIOWC (Advanced I/O Write) bit in the io_status field.
    pub const AIOWC_BIT: u8 = 0b0000_0010;
    /// A constant mask for the IOWC (I/O Write) bit in the io_status field.
    pub const IOWC_BIT: u8 = 0b0000_0001;

    /// Returns true if the BHE (Bus High Enable) pin is active (low).
    #[inline]
    pub fn bhe(&self) -> bool {
        self.pins0 & MooCycleState::PIN_BHE == 0
    }
    /// Returns true if the ALE (Address Latch Enable) pin is active (high).
    /// On architectures that use an active-low ADS signal, ADS is translated to ALE for consistency.
    #[inline]
    pub fn ale(&self) -> bool {
        self.pins0 & MooCycleState::PIN_ALE != 0
    }
    /// Returns the current T-state of the CPU during this cycle.
    #[inline]
    pub fn t_state(&self) -> MooTState {
        MooTState::try_from(self.t_state & 0x07).unwrap_or(MooTState::Ti)
    }
    /// Returns true if the CPU is reading from memory during this cycle.
    #[inline]
    pub fn is_reading_mem(&self) -> bool {
        (self.memory_status & Self::MRDC_BIT) != 0
    }
    /// Returns true if the CPU is writing to memory during this cycle.
    #[inline]
    pub fn is_writing_mem(&self) -> bool {
        (self.memory_status & Self::MWTC_BIT) != 0
    }
    /// Returns true if the CPU is reading from I/O during this cycle.
    #[inline]
    pub fn is_reading_io(&self) -> bool {
        (self.io_status & Self::IORC_BIT) != 0
    }
    /// Returns true if the CPU is writing to I/O during this cycle.
    #[inline]
    pub fn is_writing_io(&self) -> bool {
        (self.io_status & Self::IOWC_BIT) != 0
    }
    #[inline]
    /// Returns true if the CPU is reading from either memory or I/O during this cycle.
    pub fn is_reading(&self) -> bool {
        self.is_reading_mem() || self.is_reading_io()
    }
    /// Returns true if the CPU is writing to either memory or I/O during this cycle.
    #[inline]
    pub fn is_writing(&self) -> bool {
        self.is_writing_mem() || self.is_writing_io()
    }
    /// Returns true if the CPU is performing a code fetch from memory during this cycle.
    #[inline]
    pub fn is_code_fetch(&self, cpu_type: MooCpuType) -> bool {
        self.is_reading_mem() && (self.bus_state(cpu_type) == MooBusState::CODE)
    }
    /// Returns the decoded [MooBusState] for this cycle, based on the provided [MooCpuType]
    #[inline]
    pub fn bus_state(&self, cpu_type: MooCpuType) -> MooBusState {
        cpu_type.decode_status(self.bus_state)
    }
}

/// A helper struct for implementing [Display] for [MooCycleState].
/// This struct provides necessary context for interpreting each cycle state, providing a cpu type,
/// cycle number and address latch value.
pub struct MooCycleStatePrinter {
    /// The CPU type for interpreting the cycle state as a [MooCpuType].
    pub cpu_type: MooCpuType,
    /// The address latch value to use for this cycle. This should be set when ALE is active.
    pub address_latch: u32,
    /// The [MooCycleState] to display.
    pub state: MooCycleState,
    /// Whether to show the cycle number in the output.
    pub show_cycle_num: bool,
    /// The cycle number to display if [show_cycle_num] is true.
    pub cycle_num: usize,
}

impl MooCycleStatePrinter {
    pub fn data_width(&self) -> MooDataWidth {
        let cpu_width = MooCpuDataBusWidth::from(self.cpu_type);
        match cpu_width {
            MooCpuDataBusWidth::Eight => MooDataWidth::EightLow,
            MooCpuDataBusWidth::Sixteen => {
                if ((self.address_latch & 1) != 0) && (self.state.pins0 & MooCycleState::PIN_BHE == 0) {
                    MooDataWidth::EightHigh
                }
                else if self.state.pins0 & MooCycleState::PIN_BHE == 0 {
                    MooDataWidth::Sixteen
                }
                else {
                    MooDataWidth::EightLow
                }
            }
        }
    }

    pub fn data_bus_str(&self) -> String {
        match self.data_width() {
            MooDataWidth::Invalid => "----".to_string(),
            MooDataWidth::Sixteen => format!("{:04X}", self.state.data_bus),
            MooDataWidth::EightLow => format!("{:>4}", format!("{:02X}", self.state.data_bus as u8)),
            MooDataWidth::EightHigh => format!("{:<4}", format!("{:02X}", (self.state.data_bus >> 8) as u8)),
        }
    }
}

impl Display for MooCycleStatePrinter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ale_str = if self.state.pins0 & MooCycleState::PIN_ALE != 0 {
            "A:"
        }
        else {
            "  "
        };

        let mut seg_str = "  ".to_string();

        let rs_chr = match self.state.memory_status & MooCycleState::MRDC_BIT != 0 {
            true => "R",
            false => ".",
        };
        let aws_chr = match self.state.memory_status & MooCycleState::AMWC_BIT != 0 {
            true => 'A',
            false => '.',
        };
        let ws_chr = match self.state.memory_status & MooCycleState::MWTC_BIT != 0 {
            true => "W",
            false => ".",
        };
        let ior_chr = match self.state.io_status & MooCycleState::IORC_BIT != 0 {
            true => 'R',
            false => '.',
        };
        let aiow_chr = match self.state.io_status & MooCycleState::AIOWC_BIT != 0 {
            true => 'A',
            false => '.',
        };
        let iow_chr = match self.state.io_status & MooCycleState::IOWC_BIT != 0 {
            true => 'W',
            false => '.',
        };

        let bhe_chr = match self.state.bhe() {
            true => 'B',
            false => '.',
        };

        let ready_chr = match self.state.pins0 & MooCycleState::PIN_READY != 0 {
            true => 'R',
            false => '.',
        };

        // LOCK is consistently active-low across all x86 CPUs.
        let lock_chr = match self.state.pins0 & MooCycleState::PIN_LOCK == 0 {
            true => 'L',
            false => '.',
        };

        let intr_chr = '.';
        let inta_chr = '.';

        let bus_state = self.cpu_type.decode_status(self.state.bus_state);
        let bus_raw = self.cpu_type.raw_status(self.state.bus_state);
        let bus_str = bus_state.to_string();

        let t_state = self.state.t_state.try_into().unwrap_or(MooTState::Ti);
        let t_str = self.cpu_type.tstate_to_string(t_state);

        let mut xfer_str = "        ".to_string();

        let bus_active = match self.cpu_type {
            MooCpuType::Intel80386Ex => {
                // The 386 can write on t1
                if self.state.is_writing() {
                    true
                }
                else {
                    // The 386 can read after T1
                    t_state != MooTState::T1
                }
            }
            MooCpuType::Intel80286 => {
                // The 286 can read/write after T1
                t_state != MooTState::T1
            }
            _ => {
                // Older CPUs can only read/write in PASV state
                bus_state == MooBusState::PASV
            }
        };

        if bus_active {
            let value = self.data_bus_str();
            if self.state.is_reading() {
                xfer_str = format!("r-> {}", value);
            }
            else if self.state.is_writing() {
                xfer_str = format!("<-w {}", value);
            }
        }

        let bus_chr_width = self.cpu_type.bus_chr_width();
        let data_chr_width = self.cpu_type.data_chr_width();

        let bus_str = format!("{bus_str:04}[{bus_raw:01}]");

        let cycle_num_str = if self.show_cycle_num {
            format!("{:04} ", self.cycle_num)
        }
        else {
            "".to_string()
        };

        write!(
            f,
            "{cycle_num_str}{ale_str:02}{addr_latch:0bus_chr_width$X}:{addr_bus:0bus_chr_width$X}:{data_bus:0data_chr_width$X} \
            {xfer_str:06} \
            {seg_str:02} \
            M:{rs_chr}{aws_chr}{ws_chr} \
            I:{ior_chr}{aiow_chr}{iow_chr} \
            P:{intr_chr}{inta_chr}{lock_chr}{ready_chr}{bhe_chr} \
            {bus_str:08} {t_str:02}",
            addr_latch = self.address_latch,
            addr_bus = self.state.address_bus,
            data_bus = self.state.data_bus,
            // q_op_chr = q_op_chr,
            // q_str = self.queue.to_string(),
            // width = self.queue.size() * 2,
            // q_read_str = q_read_str,
        )
    }
}
