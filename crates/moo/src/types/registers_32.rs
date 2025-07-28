use crate::types::MooCpuType;
use binrw::binrw;
use std::fmt::Display;

#[cfg(feature = "use_arduinox86_client")]
use arduinox86_client::RemoteCpuRegistersV1;

pub struct MooRegisters32Init {
    pub cr0: u32,
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
    pub const ALL_SET: u32 = 0x0003_FFFF; // All registers set mask

    pub const TOP_16_MASK: u32 = 0xFFFF_0000; // Mask for the top 16 bits of the register mask

    pub const CR0_MASK: u32 = 0x0000_0001; // CR0 register mask
    pub const EAX_MASK: u32 = 0x0000_0002; // EAX register mask
    pub const EBX_MASK: u32 = 0x0000_0004; // EBX register mask
    pub const ECX_MASK: u32 = 0x0000_0008; // ECX register mask
    pub const EDX_MASK: u32 = 0x0000_0010; // EDX register mask
    pub const ESI_MASK: u32 = 0x0000_0020; // ESI register mask
    pub const EDI_MASK: u32 = 0x0000_0040; // EDI register mask
    pub const EBP_MASK: u32 = 0x0000_0080; // EBP register mask
    pub const ESP_MASK: u32 = 0x0000_0100; // ESP register mask
    pub const CS_MASK: u32 = 0x0000_0200; // CS register mask
    pub const DS_MASK: u32 = 0x0000_0400; // DS register mask
    pub const ES_MASK: u32 = 0x0000_0800; // ES register mask
    pub const FS_MASK: u32 = 0x0000_1000; // FS register mask
    pub const GS_MASK: u32 = 0x0000_2000; // GS register mask
    pub const SS_MASK: u32 = 0x0000_4000; // SS register mask
    pub const EIP_MASK: u32 = 0x0000_8000; // EIP register mask
    pub const EFLAGS_MASK: u32 = 0x0001_0000; // EFLAGS register mask
    pub const DR6_MASK: u32 = 0x0002_0000; // DR6 register mask
    pub const DR7_MASK: u32 = 0x0004_0000; // DR7 register mask

    pub const FLAG_CARRY: u16       = 0b0000_0000_0000_0001;
    pub const FLAG_RESERVED1: u16   = 0b0000_0000_0000_0010;
    pub const FLAG_PARITY: u16      = 0b0000_0000_0000_0100;
    pub const FLAG_RESERVED3: u16   = 0b0000_0000_0000_1000;
    pub const FLAG_AUX_CARRY: u16   = 0b0000_0000_0001_0000;
    pub const FLAG_RESERVED5: u16   = 0b0000_0000_0010_0000;
    pub const FLAG_ZERO: u16        = 0b0000_0000_0100_0000;
    pub const FLAG_SIGN: u16        = 0b0000_0000_1000_0000;
    pub const FLAG_TRAP: u16        = 0b0000_0001_0000_0000;
    pub const FLAG_INT_ENABLE: u16  = 0b0000_0010_0000_0000;
    pub const FLAG_DIRECTION: u16   = 0b0000_0100_0000_0000;
    pub const FLAG_OVERFLOW: u16    = 0b0000_1000_0000_0000;
    pub const FLAG_F15: u16         = 0b1000_0000_0000_0000; // Reserved bit 15
    pub const FLAG_MODE: u16        = 0b1000_0000_0000_0000;
    pub const FLAG_NT: u16          = 0b0100_0000_0000_0000; // Nested Task
    pub const FLAG_IOPL0: u16       = 0b0001_0000_0000_0000; // IO Privilege Level
    pub const FLAG_IOPL1: u16       = 0b0010_0000_0000_0000; // IO Privilege Level

    pub fn set_ax(&mut self, value: u16) {
        self.reg_mask |= MooRegisters32::EAX_MASK;
        self.eax = (self.eax & MooRegisters32::TOP_16_MASK) | value as u32;
    }
    pub fn set_bx(&mut self, value: u16) {
        self.reg_mask |= MooRegisters32::EBX_MASK;
        self.ebx = (self.ebx & MooRegisters32::TOP_16_MASK) | value as u32;
    }
    pub fn set_cx(&mut self, value: u16) {
        self.reg_mask |= MooRegisters32::ECX_MASK;
        self.ecx = (self.ecx & MooRegisters32::TOP_16_MASK) | value as u32;
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
    pub fn esi(&self) -> Option<u32> {
        if self.reg_mask & MooRegisters32::ESI_MASK != 0 {
            Some(self.esi)
        } else {
            None
        }
    }
    pub fn di(&self) -> Option<u32> {
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

