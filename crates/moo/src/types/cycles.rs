use binrw::binrw;
use std::fmt::Display;
use crate::types::{MooBusState, MooCpuType, MooCpuWidth, MooDataWidth, MooTState};

#[derive(Copy, Clone, Debug, Default)]
#[binrw]
#[brw(little)]
pub struct MooCycleState {
    pub pins0: u8,
    pub address_bus: u32,
    pub segment: u8,
    pub memory_status: u8,
    pub io_status: u8,
    pub pins1: u8,
    pub data_bus: u16,
    pub bus_state: u8,
    pub t_state: u8,
    pub queue_op: u8,
    pub queue_byte: u8,
}

impl MooCycleState {
    pub const PIN_ALE: u8 = 0b0000_0001;
    pub const PIN_BHE: u8 = 0b0000_0010;
    pub const PIN_READY: u8 = 0b0000_0100;
    pub const PIN_LOCK: u8 = 0b0000_1000;

    pub const MRDC_BIT: u8 = 0b0000_0100;
    pub const AMWC_BIT: u8 = 0b0000_0010;
    pub const MWTC_BIT: u8 = 0b0000_0001;

    pub const IORC_BIT: u8 =  0b0000_0100;
    pub const AIOWC_BIT: u8 = 0b0000_0010;
    pub const IOWC_BIT: u8 = 0b0000_0001;

    #[inline]
    pub fn bhe(&self) -> bool {
        self.pins0 & MooCycleState::PIN_BHE == 0
    }
    #[inline]
    pub fn ale(&self) -> bool {
        self.pins0 & MooCycleState::PIN_ALE != 0
    }
    #[inline]
    pub fn t_state(&self) -> MooTState {
        MooTState::try_from(self.t_state & 0x07).unwrap_or(MooTState::Ti)
    }
    #[inline]
    pub fn is_reading_mem(&self) -> bool {
        (self.memory_status & Self::MRDC_BIT) != 0
    }
    #[inline]
    pub fn is_writing_mem(&self) -> bool {
        (self.memory_status & Self::MWTC_BIT) != 0
    }
    #[inline]
    pub fn is_reading_io(&self) -> bool {
        (self.io_status & Self::IORC_BIT) != 0
    }
    #[inline]
    pub fn is_writing_io(&self) -> bool {
        (self.io_status & Self::IOWC_BIT) != 0
    }
    #[inline]
    pub fn is_reading(&self) -> bool {
        self.is_reading_mem() || self.is_reading_io()
    }
    #[inline]
    pub fn is_writing(&self) -> bool {
        self.is_writing_mem() || self.is_writing_io()
    }
}

pub struct MooCycleStatePrinter {
    pub cpu_type: MooCpuType,
    pub address_latch: u32,
    pub state: MooCycleState,
}

impl MooCycleStatePrinter {
    pub fn data_width(&self) -> MooDataWidth {
        let cpu_width = MooCpuWidth::from(self.cpu_type);
        match cpu_width {
            MooCpuWidth::Eight => MooDataWidth::EightLow,
            MooCpuWidth::Sixteen => {
                if (self.address_latch & 1 != 0)
                    && (self.state.pins0 & MooCycleState::PIN_ALE == 0)
                {
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
        } else {
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

        let intr_chr = '.';
        let inta_chr = '.';

        let bus_state = self.cpu_type.decode_status(self.state.bus_state);
        let bus_raw = self.cpu_type.raw_status(self.state.bus_state);
        let bus_str = bus_state.to_string();

        let t_state = self.state.t_state.try_into().unwrap_or(MooTState::Ti);
        let t_string = self.cpu_type.tstate_to_string(t_state);

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

        write!(
            f,
            "{ale_str:02}{addr_latch:0bus_chr_width$X}:{addr_bus:0bus_chr_width$X}:{data_bus:0data_chr_width$X} \
            {seg_str:02} M:{rs_chr}{aws_chr}{ws_chr} I:{ior_chr}{aiow_chr}{iow_chr} \
            P:{intr_chr}{inta_chr}{bhe_chr} {bus_str:04}[{bus_raw:01}] {t_str:02} {xfer_str:06}",
            ale_str = ale_str,
            addr_latch = self.address_latch,
            addr_bus = self.state.address_bus,
            data_bus = self.state.data_bus,
            bus_chr_width = bus_chr_width,
            seg_str = seg_str,
            rs_chr = rs_chr,
            aws_chr = aws_chr,
            ws_chr = ws_chr,
            ior_chr = ior_chr,
            aiow_chr = aiow_chr,
            iow_chr = iow_chr,
            intr_chr = intr_chr,
            inta_chr = inta_chr,
            bhe_chr = bhe_chr,
            bus_str = bus_str,
            bus_raw = bus_raw,
            t_str = t_string,
            xfer_str = xfer_str,
            // q_op_chr = q_op_chr,
            // q_str = self.queue.to_string(),
            // width = self.queue.size() * 2,
            // q_read_str = q_read_str,
        )
    }
}
