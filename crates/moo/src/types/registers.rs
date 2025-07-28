use binrw::binrw;
use crate::types::{MooCpuType, MooRegisters16, MooRegisters16Init, MooRegisters32, MooRegisters32Init};
use crate::types::chunks::MooChunkType;

#[derive (Clone, PartialEq)]
#[binrw]
#[brw(little)]
pub enum MooRegisters {
    Sixteen(MooRegisters16),
    ThirtyTwo(MooRegisters32),
}

#[derive (Clone)]
pub enum MooRegistersInit {
    Sixteen(MooRegisters16Init),
    ThirtyTwo(MooRegisters32Init),
}

impl From<&MooRegisters> for MooChunkType {
    fn from(regs: &MooRegisters) -> Self {
        match regs {
            MooRegisters::Sixteen(_) => MooChunkType::Registers16,
            MooRegisters::ThirtyTwo(_) => MooChunkType::Registers32,
        }
    }
}

impl From<MooRegistersInit> for MooRegisters {
    fn from(init: MooRegistersInit) -> Self {
        MooRegisters::from(&init)
    }
}

impl From<(&MooRegistersInit, &MooRegistersInit)> for MooRegisters {
    fn from((init1, init2): (&MooRegistersInit, &MooRegistersInit)) -> Self {
        match (init1, init2) {
            (MooRegistersInit::Sixteen(regs1), MooRegistersInit::Sixteen(regs2)) => {
                MooRegisters::Sixteen(MooRegisters16::from((regs1, regs2)))
            }
            (MooRegistersInit::ThirtyTwo(regs1), MooRegistersInit::ThirtyTwo(regs2)) => {
                MooRegisters::ThirtyTwo(MooRegisters32::from((regs1, regs2)))
            }
            _ => panic!("Cannot combine different register types"),
        }
    }
}

impl From<&MooRegistersInit > for MooRegisters {
    fn from(init: &MooRegistersInit) -> Self {
        match init {
            MooRegistersInit::Sixteen(regs) => MooRegisters::Sixteen(MooRegisters16::from(regs)),
            MooRegistersInit::ThirtyTwo(regs) => MooRegisters::ThirtyTwo(MooRegisters32::from(regs)),
        }
    }
}

impl Default for MooRegisters {
    fn default() -> Self {
        MooRegisters::Sixteen(MooRegisters16::default())
    }
}

impl MooRegisters {
    pub fn default_opt(cpu_type: MooCpuType) -> Self {
        match cpu_type {
            MooCpuType::Intel80386Ex => MooRegisters::ThirtyTwo(MooRegisters32::default()),
            _ => MooRegisters::Sixteen(MooRegisters16::default()),
        }
    }
}