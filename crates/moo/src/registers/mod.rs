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

//! # Registers
//! This module provides types that represent CPU registers and segment descriptors.

pub mod descriptors_16;
pub mod descriptors_32;
pub mod registers_16;
pub mod registers_32;

use std::fmt::Display;

use crate::types::{chunks::MooChunkType, MooCpuType};

use binrw::binrw;

use crate::registers::{descriptors_16::MooDescriptors16, descriptors_32::MooDescriptors32};
pub use registers_16::{MooRegisters16, MooRegisters16Init, MooRegisters16Printer};
pub use registers_32::{MooRegisters32, MooRegisters32Init, MooRegisters32Printer};

/// An enumeration of all possible CPU registers understood by MOO.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[binrw]
#[brw(little)]
#[br(repr = u8)]
#[bw(repr = u8)]
pub enum MooRegister {
    AX,
    BX,
    CX,
    DX,
    CS,
    SS,
    DS,
    ES,
    SP,
    BP,
    SI,
    DI,
    IP,
    FLAGS,
    CR0,
    CR3,
    EAX,
    EBX,
    ECX,
    EDX,
    ESI,
    EDI,
    EBP,
    ESP,
    FS,
    GS,
    EIP,
    DR6,
    DR7,
    EFLAGS,
}

/// An enumeration of all possible segment registers understood by MOO.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[binrw]
#[brw(little)]
#[br(repr = u8)]
#[bw(repr = u8)]
pub enum MooSegmentRegister {
    /// The code segment register.
    CS,
    /// The stack segment register.
    SS,
    /// The data segment register.
    DS,
    /// The extra segment register.
    ES,
    /// The FS segment register is only available on 32-bit and later x86 CPUs.
    FS,
    /// The GS segment register is only available on 32-bit and later x86 CPUs.
    GS,
}

impl MooRegister {
    pub fn is_32bit(&self) -> bool {
        matches!(
            self,
            MooRegister::EAX
                | MooRegister::EBX
                | MooRegister::ECX
                | MooRegister::EDX
                | MooRegister::ESI
                | MooRegister::EDI
                | MooRegister::EBP
                | MooRegister::ESP
                | MooRegister::EIP
                | MooRegister::EFLAGS
                | MooRegister::CR0
                | MooRegister::CR3
                | MooRegister::DR6
                | MooRegister::DR7
        )
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MooRegisterDiff {
    pub register: MooRegister,
    pub initial:  u32,
    pub r#final:  u32,
}

impl MooRegisterDiff {
    pub fn register(&self) -> MooRegister {
        self.register
    }
}

#[derive(Clone, Debug, PartialEq)]
#[binrw]
#[brw(little)]
pub enum MooDescriptors {
    Sixteen(MooDescriptors16),
    ThirtyTwo(MooDescriptors32),
}

#[derive(Clone, Debug, PartialEq)]
#[binrw]
#[brw(little)]
pub enum MooRegisters {
    Sixteen(MooRegisters16),
    ThirtyTwo(MooRegisters32),
}

#[derive(Clone)]
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

impl From<&MooRegistersInit> for MooRegisters {
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

    pub fn is_valid(&self) -> bool {
        match self {
            MooRegisters::Sixteen(regs) => regs.is_valid(),
            MooRegisters::ThirtyTwo(regs) => regs.is_valid(),
        }
    }

    pub fn flags(&self) -> u32 {
        match self {
            MooRegisters::Sixteen(regs) => regs.flags as u32,
            MooRegisters::ThirtyTwo(regs) => regs.eflags,
        }
    }

    pub fn delta(&self, other: &MooRegisters) -> MooRegisters {
        match (self, other) {
            (MooRegisters::Sixteen(regs1), MooRegisters::Sixteen(regs2)) => MooRegisters::Sixteen(regs1.delta(regs2)),
            (MooRegisters::ThirtyTwo(regs1), MooRegisters::ThirtyTwo(regs2)) => {
                MooRegisters::ThirtyTwo(regs1.delta(regs2))
            }
            _ => panic!("Cannot compare different register types"),
        }
    }

    pub fn sp_linear_real(&self) -> Option<u32> {
        match self {
            MooRegisters::Sixteen(regs) => regs.sp_linear_real(),
            MooRegisters::ThirtyTwo(regs) => regs.sp_linear_real(),
        }
    }

    pub fn csip_linear_real(&self) -> Option<u32> {
        match self {
            MooRegisters::Sixteen(regs) => regs.csip_linear_real(),
            MooRegisters::ThirtyTwo(regs) => regs.csip_linear_real(),
        }
    }
}

pub struct MooRegistersPrinter<'a> {
    pub regs: &'a MooRegisters,
    pub cpu_type: MooCpuType,
    pub diff: Option<&'a MooRegisters>,
    pub indent: u32,
}

impl Display for MooRegistersPrinter<'_> {
    #[rustfmt::skip]
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        match (self.regs, self.diff) {
            (MooRegisters::Sixteen(regs), None) => {
                write!(fmt, "{}", MooRegisters16Printer { regs, cpu_type: self.cpu_type, diff: None, indent: self.indent })
            }
            (MooRegisters::Sixteen(regs), Some(MooRegisters::Sixteen(diff_regs))) => {
                let rehydrated = regs.rehydrate(diff_regs);
                write!(fmt, "{}", MooRegisters16Printer { regs: &rehydrated, cpu_type: self.cpu_type, diff: Some(diff_regs), indent: self.indent })
            }
            (MooRegisters::ThirtyTwo(regs), None) => {
                write!(fmt, "{}", MooRegisters32Printer { regs, cpu_type: self.cpu_type, diff: None, indent: self.indent })
            }
            (MooRegisters::ThirtyTwo(regs), Some(MooRegisters::ThirtyTwo(diff_regs))) => {
                let rehydrated = regs.rehydrate(diff_regs);
                write!(fmt, "{}", MooRegisters32Printer { regs: &rehydrated, cpu_type: self.cpu_type, diff: Some(diff_regs), indent: self.indent })
            }
            _ => Err(std::fmt::Error),
        }
    }
}
