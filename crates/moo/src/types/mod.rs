pub mod chunks;
pub mod cycles;
pub mod errors;
pub mod metadata;
pub mod ram;
pub mod registers;
pub mod state;
pub mod test;

use binrw::binrw;
pub use cycles::*;
pub use metadata::*;
pub use ram::*;
pub use registers::*;
pub use state::*;
pub use test::*;

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

impl MooCpuType {
    pub fn bitness(&self) -> u32 {
        if self.is_16bit() {
            16
        } else {
            8
        }
    }

    pub fn is_16bit(&self) -> bool {
        matches!(
            self,
            MooCpuType::Intel8086
                | MooCpuType::Intel80186
                | MooCpuType::Intel80286
                | MooCpuType::NecV30
                | MooCpuType::Intel80386Ex
        )
    }

    pub fn is_8bit(&self) -> bool {
        matches!(
            self,
            MooCpuType::Intel8088 | MooCpuType::Intel80188 | MooCpuType::NecV20
        )
    }

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
    pub flag_address: u32,
}
