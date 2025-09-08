
use std::fmt::Display;

use binrw::binrw;
use crate::types::{MooCpuType};

#[derive(Clone)]
#[binrw]
#[brw(little)]
pub struct MooDescriptor32 {
    pub access: u32,
    pub base: u32,
    pub limit: u32,
}

impl Display for MooDescriptor32 {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            fmt,
            "Access:{:08X} Base:{:08X} Limit:{:08X}",
            self.access, self.base, self.limit,
        )
    }
}
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

#[derive(Clone)]
#[binrw]
#[brw(little)]
pub struct MooDescriptors32 {

}

#[derive(Clone)]
pub struct MooRegisters32Init {
    pub cr0: u32,
    pub cr3: u32,
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
    pub esi: u32,
    pub edi: u32,
    pub ebp: u32,
    pub esp: u32,
    pub cs: u32,
    pub ds: u32,
    pub es: u32,
    pub fs: u32,
    pub gs: u32,
    pub ss: u32,
    pub eip: u32,
    pub dr6: u32,
    pub dr7: u32,
    pub eflags: u32,
}

#[derive(Copy, Clone)]
#[binrw]
#[brw(little)]
pub struct MooRegisters32 {
    reg_mask: u32,
    #[brw(if(reg_mask & MooRegisters32::CR0_MASK != 0))]
    pub cr0: u32,
    #[brw(if(reg_mask & MooRegisters32::CR3_MASK != 0))]
    pub cr3: u32, // SMM dump only
    #[brw(if(reg_mask & MooRegisters32::EAX_MASK != 0))]
    pub eax: u32,
    #[brw(if(reg_mask & MooRegisters32::EBX_MASK != 0))]
    pub ebx: u32,
    #[brw(if(reg_mask & MooRegisters32::ECX_MASK != 0))]
    pub ecx: u32,
    #[brw(if(reg_mask & MooRegisters32::EDX_MASK != 0))]
    pub edx: u32,
    #[brw(if(reg_mask & MooRegisters32::ESI_MASK != 0))]
    pub esi: u32,
    #[brw(if(reg_mask & MooRegisters32::EDI_MASK != 0))]
    pub edi: u32,
    #[brw(if(reg_mask & MooRegisters32::EBP_MASK != 0))]
    pub ebp: u32,
    #[brw(if(reg_mask & MooRegisters32::ESP_MASK != 0))]
    pub esp: u32,
    #[brw(if(reg_mask & MooRegisters32::CS_MASK != 0))]
    pub cs: u32,
    #[brw(if(reg_mask & MooRegisters32::DS_MASK != 0))]
    pub ds: u32,
    #[brw(if(reg_mask & MooRegisters32::ES_MASK != 0))]
    pub es: u32,
    #[brw(if(reg_mask & MooRegisters32::FS_MASK != 0))]
    pub fs: u32,
    #[brw(if(reg_mask & MooRegisters32::GS_MASK != 0))]
    pub gs: u32,
    #[brw(if(reg_mask & MooRegisters32::SS_MASK != 0))]
    pub ss: u32,
    #[brw(if(reg_mask & MooRegisters32::EIP_MASK != 0))]
    pub eip: u32,
    #[brw(if(reg_mask & MooRegisters32::EFLAGS_MASK != 0))]
    pub eflags: u32,
    #[brw(if(reg_mask & MooRegisters32::DR6_MASK != 0))]
    pub dr6: u32,
    #[brw(if(reg_mask & MooRegisters32::DR7_MASK != 0))]
    pub dr7: u32,
}

impl PartialEq for MooRegisters32 {
    fn eq(&self, other: &Self) -> bool {
        self.cr0 == other.cr0
            && self.cr3 == other.cr3
            && self.eax == other.eax
            && self.ebx == other.ebx
            && self.ecx == other.ecx
            && self.edx == other.edx
            && self.cs == other.cs
            && self.ss == other.ss
            && self.ds == other.ds
            && self.es == other.es
            && self.fs == other.fs
            && self.gs == other.gs
            && self.esp == other.esp
            && self.ebp == other.ebp
            && self.esi == other.esi
            && self.edi == other.edi
            && self.eip == other.eip
            && self.eflags == other.eflags
    }
}

impl Default for MooRegisters32 {
    fn default() -> Self {
        Self {
            reg_mask: 0,
            cr0: 0,
            cr3: 0,
            eax: 0,
            ebx: 0,
            ecx: 0,
            edx: 0,
            cs: 0,
            ss: 0,
            ds: 0,
            es: 0,
            fs: 0,
            gs: 0,
            esp: 0,
            ebp: 0,
            esi: 0,
            edi: 0,
            eip: 0,
            eflags: 0,
            dr6: 0,
            dr7: 0,
        }
    }
}

impl From<&MooRegisters32Init> for MooRegisters32 {
    fn from(init: &MooRegisters32Init) -> Self {
        Self {
            reg_mask: MooRegisters32::ALL_SET, // All registers set
            cr0: init.cr0,
            cr3: init.cr3,
            eax: init.eax,
            ebx: init.ebx,
            ecx: init.ecx,
            edx: init.edx,
            cs: init.cs,
            ds: init.ds,
            es: init.es,
            fs: init.fs,
            gs: init.gs,
            ss: init.ss,
            esp: init.esp,
            ebp: init.ebp,
            esi: init.esi,
            edi: init.edi,
            eip: init.eip,
            eflags: init.eflags,
            dr6: init.dr6,
            dr7: init.dr7,
        }
    }
}

/// Convert a tuple of two [MooRegisters32Init] into a [MooRegisters1] based on the difference between them.
impl From<(&MooRegisters32Init, &MooRegisters32Init)> for MooRegisters32 {
    fn from(init: (&MooRegisters32Init, &MooRegisters32Init)) -> Self {
        let mut reg_mask = 0u32;

        if init.0.cr0 != init.1.cr0 {
            reg_mask |= MooRegisters32::CR0_MASK;
        }
        if init.0.cr3 != init.1.cr3 {
            reg_mask |= MooRegisters32::CR3_MASK;
        }
        if init.0.eax != init.1.eax {
            reg_mask |= MooRegisters32::EAX_MASK;
        }
        if init.0.ebx != init.1.ebx {
            reg_mask |= MooRegisters32::EBX_MASK;
        }
        if init.0.ecx != init.1.ecx {
            reg_mask |= MooRegisters32::ECX_MASK;
        }
        if init.0.edx != init.1.edx {
            reg_mask |= MooRegisters32::EDX_MASK;
        }
        if init.0.cs != init.1.cs {
            reg_mask |= MooRegisters32::CS_MASK;
        }
        if init.0.ds != init.1.ds {
            reg_mask |= MooRegisters32::DS_MASK;
        }
        if init.0.es != init.1.es {
            reg_mask |= MooRegisters32::ES_MASK;
        }
        if init.0.fs != init.1.fs {
            reg_mask |= MooRegisters32::FS_MASK;
        }
        if init.0.gs != init.1.gs {
            reg_mask |= MooRegisters32::GS_MASK;
        }
        if init.0.ss != init.1.ss {
            reg_mask |= MooRegisters32::SS_MASK;
        }

        if init.0.esp != init.1.esp {
            reg_mask |= MooRegisters32::ESP_MASK
        }
        if init.0.ebp != init.1.ebp {
            reg_mask |= MooRegisters32::EBP_MASK
        }
        if init.0.esi != init.1.esi {
            reg_mask |= MooRegisters32::ESI_MASK
        }
        if init.0.edi != init.1.edi {
            reg_mask |= MooRegisters32::EDI_MASK
        }
        if init.0.eip != init.1.eip {
            reg_mask |= MooRegisters32::EIP_MASK
        }
        if init.0.eflags != init.1.eflags {
            reg_mask |= MooRegisters32::EFLAGS_MASK
        }
        if init.0.dr6 != init.1.dr6 {
            reg_mask |= MooRegisters32::DR6_MASK;
        }
        if init.0.dr7 != init.1.dr7 {
            reg_mask |= MooRegisters32::DR7_MASK;
        }

        Self {
            reg_mask,
            cr0: init.1.cr0,
            cr3: init.1.cr3,
            eax: init.1.eax,
            ebx: init.1.ebx,
            ecx: init.1.ecx,
            edx: init.1.edx,
            cs: init.1.cs,
            ss: init.1.ss,
            ds: init.1.ds,
            es: init.1.es,
            fs: init.1.fs,
            gs: init.1.gs,
            esi: init.1.esi,
            edi: init.1.edi,
            ebp: init.1.ebp,
            esp: init.1.esp,
            eip: init.1.eip,
            eflags: init.1.eflags,
            dr6: init.1.dr6,
            dr7: init.1.dr7,
        }
    }
}

#[rustfmt::skip]
impl MooRegisters32 {
    pub const ALL_SET: u32 = 0x000F_FFFF; // All registers set mask

    pub const TOP_16_MASK: u32 = 0xFFFF_0000; // Mask for the top 16 bits of the register mask

    pub const CR0_MASK: u32 = 0x0000_0001; // CR0 register mask
    pub const CR3_MASK: u32 = 0x0000_0002; // CR3 register mask (SMM dump only)
    pub const EAX_MASK: u32 = 0x0000_0004; // EAX register mask
    pub const EBX_MASK: u32 = 0x0000_0008; // EBX register mask
    pub const ECX_MASK: u32 = 0x0000_0010; // ECX register mask
    pub const EDX_MASK: u32 = 0x0000_0020; // EDX register mask
    pub const ESI_MASK: u32 = 0x0000_0040; // ESI register mask
    pub const EDI_MASK: u32 = 0x0000_0080; // EDI register mask
    pub const EBP_MASK: u32 = 0x0000_0100; // EBP register mask
    pub const ESP_MASK: u32 = 0x0000_0200; // ESP register mask
    pub const CS_MASK: u32 = 0x0000_0400; // CS register mask
    pub const DS_MASK: u32 = 0x0000_0800; // DS register mask
    pub const ES_MASK: u32 = 0x0000_1000; // ES register mask
    pub const FS_MASK: u32 = 0x0000_2000; // FS register mask
    pub const GS_MASK: u32 = 0x0000_4000; // GS register mask
    pub const SS_MASK: u32 = 0x0000_8000; // SS register mask
    pub const EIP_MASK: u32 = 0x0001_0000; // EIP register mask
    pub const EFLAGS_MASK: u32 = 0x0002_0000; // EFLAGS register mask
    pub const DR6_MASK: u32 = 0x0004_0000; // DR6 register mask
    pub const DR7_MASK: u32 = 0x0008_0000; // DR7 register mask

    pub const SHUTDOWN_BIT: u32 = 0x8000_0000; // Indicates if the CPU shutdown. This should be the only bit set if set.

    pub const FLAG_CARRY: u32       = 0b0000_0000_0000_0001;
    pub const FLAG_RESERVED1: u32   = 0b0000_0000_0000_0010;
    pub const FLAG_PARITY: u32      = 0b0000_0000_0000_0100;
    pub const FLAG_RESERVED3: u32   = 0b0000_0000_0000_1000;
    pub const FLAG_AUX_CARRY: u32   = 0b0000_0000_0001_0000;
    pub const FLAG_RESERVED5: u32   = 0b0000_0000_0010_0000;
    pub const FLAG_ZERO: u32        = 0b0000_0000_0100_0000;
    pub const FLAG_SIGN: u32        = 0b0000_0000_1000_0000;
    pub const FLAG_TRAP: u32        = 0b0000_0001_0000_0000;
    pub const FLAG_INT_ENABLE: u32  = 0b0000_0010_0000_0000;
    pub const FLAG_DIRECTION: u32   = 0b0000_0100_0000_0000;
    pub const FLAG_OVERFLOW: u32    = 0b0000_1000_0000_0000;
    pub const FLAG_F15: u32         = 0b1000_0000_0000_0000; // Reserved bit 15
    pub const FLAG_MODE: u32        = 0b1000_0000_0000_0000;
    pub const FLAG_NT: u32          = 0b0100_0000_0000_0000; // Nested Task
    pub const FLAG_IOPL0: u32       = 0b0001_0000_0000_0000; // IO Privilege Level
    pub const FLAG_IOPL1: u32       = 0b0010_0000_0000_0000; // IO Privilege Level

    pub fn set_shutdown(&mut self, state: bool) {
        if state {
            // Clear out all other bits.
            self.reg_mask = MooRegisters32::SHUTDOWN_BIT;
        }
        else {
            self.reg_mask &= !MooRegisters32::SHUTDOWN_BIT;
        }
    }

    pub fn set_eax(&mut self, value: u32) {
        self.reg_mask |= MooRegisters32::EAX_MASK;
        self.eax = value;
    }
    pub fn set_ax(&mut self, value: u16) {
        self.reg_mask |= MooRegisters32::EAX_MASK;
        self.eax = (self.eax & MooRegisters32::TOP_16_MASK) | value as u32;
    }
    pub fn set_ebx(&mut self, value: u32) {
        self.reg_mask |= MooRegisters32::EBX_MASK;
        self.ebx = value;
    }
    pub fn set_bx(&mut self, value: u16) {
        self.reg_mask |= MooRegisters32::EBX_MASK;
        self.ebx = (self.ebx & MooRegisters32::TOP_16_MASK) | value as u32;
    }
    pub fn set_ecx(&mut self, value: u32) {
        self.reg_mask |= MooRegisters32::ECX_MASK;
        self.ecx = value;
    }
    pub fn set_cx(&mut self, value: u16) {
        self.reg_mask |= MooRegisters32::ECX_MASK;
        self.ecx = (self.ecx & MooRegisters32::TOP_16_MASK) | value as u32;
    }
    pub fn set_edx(&mut self, value: u32) {
        self.reg_mask |= MooRegisters32::EDX_MASK;
        self.edx = value;
    }
    pub fn set_dx(&mut self, value: u16) {
        self.reg_mask |= MooRegisters32::EDX_MASK;
        self.edx = (self.edx & MooRegisters32::TOP_16_MASK) | value as u32;
    }
    pub fn set_cs(&mut self, value: u16) {
        self.reg_mask |= MooRegisters32::CS_MASK;
        self.cs = value as u32;
    }
    pub fn set_ss(&mut self, value: u16) {
        self.reg_mask |= MooRegisters32::SS_MASK;
        self.ss = value as u32;
    }
    pub fn set_ds(&mut self, value: u16) {
        self.reg_mask |= MooRegisters32::DS_MASK;
        self.ds = value as u32;
    }
    pub fn set_es(&mut self, value: u16) {
        self.reg_mask |= MooRegisters32::ES_MASK;
        self.es = value as u32;
    }
    pub fn set_fs(&mut self, value: u16) {
        self.reg_mask |= MooRegisters32::FS_MASK;
        self.fs = value as u32;
    }
    pub fn set_gs(&mut self, value: u16) {
        self.reg_mask |= MooRegisters32::GS_MASK;
        self.gs = value as u32;
    }
    pub fn set_esp(&mut self, value: u32) {
        self.reg_mask |= MooRegisters32::ESP_MASK;
        self.esp = value;
    }
    pub fn set_ebp(&mut self, value: u32) {
        self.reg_mask |= MooRegisters32::EBP_MASK;
        self.ebp = value;
    }
    pub fn set_esi(&mut self, value: u32) {
        self.reg_mask |= MooRegisters32::ESI_MASK;
        self.esi = value;
    }
    pub fn set_edi(&mut self, value: u32) {
        self.reg_mask |= MooRegisters32::EDI_MASK;
        self.edi = value;
    }
    pub fn set_ip(&mut self, value: u16) {
        self.reg_mask |= MooRegisters32::EIP_MASK;
        self.eip = (self.eip & MooRegisters32::TOP_16_MASK) | value as u32;
    }
    pub fn set_eip(&mut self, value: u32) {
        self.reg_mask |= MooRegisters32::EIP_MASK;
        self.eip = value;
    }
    pub fn set_flags(&mut self, value: u16) {
        self.reg_mask |= MooRegisters32::EFLAGS_MASK;
        self.eflags = (self.eflags & MooRegisters32::TOP_16_MASK) | value as u32;
    }
    pub fn set_eflags(&mut self, value: u32) {
        self.reg_mask |= MooRegisters32::EFLAGS_MASK;
        self.eflags = value;
    }

    pub fn ax(&self) -> Option<u16> {
        if self.reg_mask & MooRegisters32::EAX_MASK != 0 {
            Some(self.eax as u16)
        } else {
            None
        }
    }
    pub fn bx(&self) -> Option<u16> {
        if self.reg_mask & MooRegisters32::EBX_MASK != 0 {
            Some(self.ebx as u16)
        } else {
            None
        }
    }
    pub fn cx(&self) -> Option<u16> {
        if self.reg_mask & MooRegisters32::ECX_MASK != 0 {
            Some(self.ecx as u16)
        } else {
            None
        }
    }
    pub fn dx(&self) -> Option<u16> {
        if self.reg_mask & MooRegisters32::EDX_MASK != 0 {
            Some(self.edx as u16)
        } else {
            None
        }
    }
    pub fn eax(&self) -> Option<u32> {
        if self.reg_mask & MooRegisters32::EAX_MASK != 0 {
            Some(self.eax)
        } else {
            None
        }
    }
    pub fn ebx(&self) -> Option<u32> {
        if self.reg_mask & MooRegisters32::EBX_MASK != 0 {
            Some(self.ebx)
        } else {
            None
        }
    }
    pub fn ecx(&self) -> Option<u32> {
        if self.reg_mask & MooRegisters32::ECX_MASK != 0 {
            Some(self.ecx)
        } else {
            None
        }
    }
    pub fn edx(&self) -> Option<u32> {
        if self.reg_mask & MooRegisters32::EDX_MASK != 0 {
            Some(self.edx)
        } else {
            None
        }
    }
    pub fn cs(&self) -> Option<u16> {
        if self.reg_mask & MooRegisters32::CS_MASK != 0 {
            Some(self.cs as u16)
        } else {
            None
        }
    }
    pub fn ss(&self) -> Option<u16> {
        if self.reg_mask & MooRegisters32::SS_MASK != 0 {
            Some(self.ss as u16)
        } else {
            None
        }
    }
    pub fn ds(&self) -> Option<u16> {
        if self.reg_mask & MooRegisters32::DS_MASK != 0 {
            Some(self.ds as u16)
        } else {
            None
        }
    }
    pub fn es(&self) -> Option<u16> {
        if self.reg_mask & MooRegisters32::ES_MASK != 0 {
            Some(self.es as u16)
        } else {
            None
        }
    }
    pub fn fs(&self) -> Option<u16> {
        if self.reg_mask & MooRegisters32::FS_MASK != 0 {
            Some(self.fs as u16)
        } else {
            None
        }
    }
    pub fn gs(&self) -> Option<u16> {
        if self.reg_mask & MooRegisters32::GS_MASK != 0 {
            Some(self.gs as u16)
        } else {
            None
        }
    }
    pub fn esp(&self) -> Option<u32> {
        if self.reg_mask & MooRegisters32::ESP_MASK != 0 {
            Some(self.esp)
        } else {
            None
        }
    }
    pub fn ebp(&self) -> Option<u32> {
        if self.reg_mask & MooRegisters32::EBP_MASK != 0 {
            Some(self.ebp)
        } else {
            None
        }
    }
    pub fn cr0(&self) -> Option<u32> {
        if self.reg_mask & MooRegisters32::CR0_MASK != 0 {
            Some(self.cr0)
        } else {
            None
        }
    }
    pub fn cr3(&self) -> Option<u32> {
        if self.reg_mask & MooRegisters32::CR3_MASK != 0 {
            Some(self.cr3)
        } else {
            None
        }
    }
    pub fn esi(&self) -> Option<u32> {
        if self.reg_mask & MooRegisters32::ESI_MASK != 0 {
            Some(self.esi)
        } else {
            None
        }
    }
    pub fn edi(&self) -> Option<u32> {
        if self.reg_mask & MooRegisters32::EDI_MASK != 0 {
            Some(self.edi)
        } else {
            None
        }
    }
    pub fn ip(&self) -> Option<u16> {
        if self.reg_mask & MooRegisters32::EIP_MASK != 0 {
            Some(self.eip as u16)
        } else {
            None
        }
    }
    pub fn eip(&self) -> Option<u32> {
        if self.reg_mask & MooRegisters32::EIP_MASK != 0 {
            Some(self.eip)
        } else {
            None
        }
    }
    pub fn flags(&self) -> Option<u16> {
        if self.reg_mask & MooRegisters32::EFLAGS_MASK != 0 {
            Some(self.eflags as u16)
        } else {
            None
        }
    }
    pub fn eflags(&self) -> Option<u32> {
        if self.reg_mask & MooRegisters32::EFLAGS_MASK != 0 {
            Some(self.eflags)
        } else {
            None
        }
    }
    pub fn dr6(&self) -> Option<u32> {
        if self.reg_mask & MooRegisters32::DR6_MASK != 0 {
            Some(self.dr6)
        } else {
            None
        }
    }
    pub fn dr7(&self) -> Option<u32> {
        if self.reg_mask & MooRegisters32::DR7_MASK != 0 {
            Some(self.dr7)
        } else {
            None
        }
    }

    pub fn is_valid(&self) -> bool {
        if self.reg_mask & MooRegisters32::EFLAGS_MASK != 0 {
            // We have flags
            if self.eflags & 0x0000_0002 == 0 {
                // Reserved flag bit 1 cannot be clear
                return false;
            }
        }
        true
    }
}


pub struct MooRegisters32Printer<'a> {
    pub regs: &'a MooRegisters32,
    pub cpu_type: MooCpuType,
    pub diff: Option<&'a MooRegisters32>,
}

macro_rules! diff_chr {
    ($self:expr, $reg:ident) => {
        if let Some(d) = $self.diff {
            if $self.regs.$reg != d.$reg {
                '*'
            } else {
                ' '
            }
        } else {
            ' '
        }
    };
}

impl Display for crate::types::MooRegisters32Printer<'_> {
    #[rustfmt::skip]
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let reg_str = format!(
            "CR0:{}{:08X}\n\
             EAX:{}{:08X} EBX:{}{:08X} ECX:{}{:08X} EDX:{}{:08X}\n\
             ESI:{}{:08X} EDI:{}{:08X} EBP:{}{:08X} ESP:{}{:08X} \n\
             CS:{}{:04X} DS:{}{:04X} ES:{}{:04X} FS:{}{:04X} GS:{}{:04X} SS:{}{:04X}\n\
             EIP:{}{:08X}\n\
             EFLAGS:{}{:08X} ",
            diff_chr!(self, cr0), self.regs.cr0,
            diff_chr!(self, eax), self.regs.eax,
            diff_chr!(self, ebx), self.regs.ebx,
            diff_chr!(self, ecx), self.regs.ecx,
            diff_chr!(self, edx), self.regs.edx,
            diff_chr!(self, esi), self.regs.esi,
            diff_chr!(self, edi), self.regs.edi,
            diff_chr!(self, ebp), self.regs.ebp,
            diff_chr!(self, esp), self.regs.esp,

            diff_chr!(self, cs), self.regs.cs,
            diff_chr!(self, ds), self.regs.ds,
            diff_chr!(self, es), self.regs.es,
            diff_chr!(self, fs), self.regs.fs,
            diff_chr!(self, gs), self.regs.gs,
            diff_chr!(self, ss), self.regs.ss,
            diff_chr!(self, eip), self.regs.eip,
            diff_chr!(self, eflags), self.regs.eflags,
        );

        // Expand flag info
        let f = self.regs.eflags;
        let c_chr = if MooRegisters32::FLAG_CARRY & f != 0 { 'C' } else { 'c' };
        let p_chr = if MooRegisters32::FLAG_PARITY & f != 0 { 'P' } else { 'p' };
        let a_chr = if MooRegisters32::FLAG_AUX_CARRY & f != 0 {
            'A'
        } else {
            'a'
        };
        let z_chr = if MooRegisters32::FLAG_ZERO & f != 0 { 'Z' } else { 'z' };
        let s_chr = if MooRegisters32::FLAG_SIGN & f != 0 { 'S' } else { 's' };
        let t_chr = if MooRegisters32::FLAG_TRAP & f != 0 { 'T' } else { 't' };
        let i_chr = if MooRegisters32::FLAG_INT_ENABLE & f != 0 {
            'I'
        } else {
            'i'
        };
        let d_chr = if MooRegisters32::FLAG_DIRECTION & f != 0 {
            'D'
        } else {
            'd'
        };
        let o_chr = if MooRegisters32::FLAG_OVERFLOW & f != 0 {
            'O'
        } else {
            'o'
        };
        let m_chr =
            if f & MooRegisters32::FLAG_F15 != 0 {
                '1'
            } else {
                '0'
            };

        let nt_chr = if f & MooRegisters32::FLAG_NT != 0 { '1' } else { '0' };
        let iopl0_chr = if f & MooRegisters32::FLAG_IOPL0 != 0 { '1' } else { '0' };
        let iopl1_chr = if f & MooRegisters32::FLAG_IOPL1 != 0 { '1' } else { '0' };

        write!(
            fmt,
            "{reg_str}{m_chr}{nt_chr}{iopl1_chr}{iopl0_chr}\
            {o_chr}{d_chr}{i_chr}{t_chr}{s_chr}{z_chr}0{a_chr}0{p_chr}1{c_chr}",
        )
    }
}
