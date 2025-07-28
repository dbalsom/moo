use std::fmt::Display;

use crate::types::MooCpuType;
use binrw::binrw;

#[derive(Clone)]
pub struct MooRegisters16Init {
    pub ax: u16,
    pub bx: u16,
    pub cx: u16,
    pub dx: u16,
    pub cs: u16,
    pub ss: u16,
    pub ds: u16,
    pub es: u16,
    pub sp: u16,
    pub bp: u16,
    pub si: u16,
    pub di: u16,
    pub ip: u16,
    pub flags: u16,
}

#[derive(Copy, Clone)]
#[binrw]
#[brw(little)]
pub struct MooRegisters16 {
    reg_mask: u16,
    #[brw(if(reg_mask & 0x0001 != 0))]
    pub ax: u16,
    #[brw(if(reg_mask & 0x0002 != 0))]
    pub bx: u16,
    #[brw(if(reg_mask & 0x0004 != 0))]
    pub cx: u16,
    #[brw(if(reg_mask & 0x0008 != 0))]
    pub dx: u16,
    #[brw(if(reg_mask & 0x0010 != 0))]
    pub cs: u16,
    #[brw(if(reg_mask & 0x0020 != 0))]
    pub ss: u16,
    #[brw(if(reg_mask & 0x0040 != 0))]
    pub ds: u16,
    #[brw(if(reg_mask & 0x0080 != 0))]
    pub es: u16,
    #[brw(if(reg_mask & 0x0100 != 0))]
    pub sp: u16,
    #[brw(if(reg_mask & 0x0200 != 0))]
    pub bp: u16,
    #[brw(if(reg_mask & 0x0400 != 0))]
    pub si: u16,
    #[brw(if(reg_mask & 0x0800 != 0))]
    pub di: u16,
    #[brw(if(reg_mask & 0x1000 != 0))]
    pub ip: u16,
    #[brw(if(reg_mask & 0x2000 != 0))]
    pub flags: u16,
}

impl PartialEq for MooRegisters16 {
    fn eq(&self, other: &Self) -> bool {
        self.ax == other.ax
            && self.bx == other.bx
            && self.cx == other.cx
            && self.dx == other.dx
            && self.cs == other.cs
            && self.ss == other.ss
            && self.ds == other.ds
            && self.es == other.es
            && self.sp == other.sp
            && self.bp == other.bp
            && self.si == other.si
            && self.di == other.di
            && self.ip == other.ip
            && self.flags == other.flags
    }
}

impl Default for MooRegisters16 {
    fn default() -> Self {
        Self {
            reg_mask: 0,
            ax: 0,
            bx: 0,
            cx: 0,
            dx: 0,
            cs: 0,
            ss: 0,
            ds: 0,
            es: 0,
            sp: 0,
            bp: 0,
            si: 0,
            di: 0,
            ip: 0,
            flags: 0,
        }
    }
}

impl From<MooRegisters16Init> for MooRegisters16 {
    fn from(init: MooRegisters16Init) -> Self {
        MooRegisters16::from(&init)
    }
}

impl From<&MooRegisters16Init> for MooRegisters16 {
    fn from(init: &MooRegisters16Init) -> Self {
        Self {
            reg_mask: MooRegisters16::ALL_SET, // All registers set
            ax: init.ax,
            bx: init.bx,
            cx: init.cx,
            dx: init.dx,
            cs: init.cs,
            ss: init.ss,
            ds: init.ds,
            es: init.es,
            sp: init.sp,
            bp: init.bp,
            si: init.si,
            di: init.di,
            ip: init.ip,
            flags: init.flags,
        }
    }
}

/// Convert a tuple of two `MooRegisters1Init` into a `MooRegisters1` based on the difference between them.
impl From<(&MooRegisters16Init, &MooRegisters16Init)> for MooRegisters16 {
    fn from(init: (&MooRegisters16Init, &MooRegisters16Init)) -> Self {
        let mut reg_mask = 0u16;

        if init.0.ax != init.1.ax {
            reg_mask |= 0x0001;
        }
        if init.0.bx != init.1.bx {
            reg_mask |= 0x0002;
        }
        if init.0.cx != init.1.cx {
            reg_mask |= 0x0004;
        }
        if init.0.dx != init.1.dx {
            reg_mask |= 0x0008;
        }
        if init.0.cs != init.1.cs {
            reg_mask |= 0x0010;
        }
        if init.0.ss != init.1.ss {
            reg_mask |= 0x0020;
        }
        if init.0.ds != init.1.ds {
            reg_mask |= 0x0040;
        }
        if init.0.es != init.1.es {
            reg_mask |= 0x0080;
        }
        if init.0.sp != init.1.sp {
            reg_mask |= 0x0100;
        }
        if init.0.bp != init.1.bp {
            reg_mask |= 0x0200;
        }
        if init.0.si != init.1.si {
            reg_mask |= 0x0400;
        }
        if init.0.di != init.1.di {
            reg_mask |= 0x0800;
        }
        if init.0.ip != init.1.ip {
            reg_mask |= 0x1000;
        }
        if init.0.flags != init.1.flags {
            reg_mask |= 0x2000;
        }

        Self {
            reg_mask,
            ax: init.1.ax,
            bx: init.1.bx,
            cx: init.1.cx,
            dx: init.1.dx,
            cs: init.1.cs,
            ss: init.1.ss,
            ds: init.1.ds,
            es: init.1.es,
            sp: init.1.sp,
            bp: init.1.bp,
            si: init.1.si,
            di: init.1.di,
            ip: init.1.ip,
            flags: init.1.flags,
        }
    }
}

#[rustfmt::skip]
impl MooRegisters16 {
    pub const ALL_SET: u16 = 0x3FFF; // All registers set mask

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
        self.reg_mask |= 0x0001;
        self.ax = value;
    }
    pub fn set_bx(&mut self, value: u16) {
        self.reg_mask |= 0x0002;
        self.bx = value;
    }
    pub fn set_cx(&mut self, value: u16) {
        self.reg_mask |= 0x0004;
        self.cx = value;
    }
    pub fn set_dx(&mut self, value: u16) {
        self.reg_mask |= 0x0008;
        self.dx = value;
    }
    pub fn set_cs(&mut self, value: u16) {
        self.reg_mask |= 0x0010;
        self.cs = value;
    }
    pub fn set_ss(&mut self, value: u16) {
        self.reg_mask |= 0x0020;
        self.ss = value;
    }
    pub fn set_ds(&mut self, value: u16) {
        self.reg_mask |= 0x0040;
        self.ds = value;
    }
    pub fn set_es(&mut self, value: u16) {
        self.reg_mask |= 0x0080;
        self.es = value;
    }
    pub fn set_sp(&mut self, value: u16) {
        self.reg_mask |= 0x0100;
        self.sp = value;
    }
    pub fn set_bp(&mut self, value: u16) {
        self.reg_mask |= 0x0200;
        self.bp = value;
    }
    pub fn set_si(&mut self, value: u16) {
        self.reg_mask |= 0x0400;
        self.si = value;
    }
    pub fn set_di(&mut self, value: u16) {
        self.reg_mask |= 0x0800;
        self.di = value;
    }
    pub fn set_ip(&mut self, value: u16) {
        self.reg_mask |= 0x1000;
        self.ip = value;
    }
    pub fn set_flags(&mut self, value: u16) {
        self.reg_mask |= 0x2000;
        self.flags = value;
    }

    pub fn ax(&self) -> Option<u16> {
        if self.reg_mask & 0x0001 != 0 {
            Some(self.ax)
        } else {
            None
        }
    }
    pub fn bx(&self) -> Option<u16> {
        if self.reg_mask & 0x0002 != 0 {
            Some(self.bx)
        } else {
            None
        }
    }
    pub fn cx(&self) -> Option<u16> {
        if self.reg_mask & 0x0004 != 0 {
            Some(self.cx)
        } else {
            None
        }
    }
    pub fn dx(&self) -> Option<u16> {
        if self.reg_mask & 0x0008 != 0 {
            Some(self.dx)
        } else {
            None
        }
    }
    pub fn cs(&self) -> Option<u16> {
        if self.reg_mask & 0x0010 != 0 {
            Some(self.cs)
        } else {
            None
        }
    }
    pub fn ss(&self) -> Option<u16> {
        if self.reg_mask & 0x0020 != 0 {
            Some(self.ss)
        } else {
            None
        }
    }
    pub fn ds(&self) -> Option<u16> {
        if self.reg_mask & 0x0040 != 0 {
            Some(self.ds)
        } else {
            None
        }
    }
    pub fn es(&self) -> Option<u16> {
        if self.reg_mask & 0x0080 != 0 {
            Some(self.es)
        } else {
            None
        }
    }
    pub fn sp(&self) -> Option<u16> {
        if self.reg_mask & 0x0100 != 0 {
            Some(self.sp)
        } else {
            None
        }
    }
    pub fn bp(&self) -> Option<u16> {
        if self.reg_mask & 0x0200 != 0 {
            Some(self.bp)
        } else {
            None
        }
    }
    pub fn si(&self) -> Option<u16> {
        if self.reg_mask & 0x0400 != 0 {
            Some(self.si)
        } else {
            None
        }
    }
    pub fn di(&self) -> Option<u16> {
        if self.reg_mask & 0x0800 != 0 {
            Some(self.di)
        } else {
            None
        }
    }
    pub fn ip(&self) -> Option<u16> {
        if self.reg_mask & 0x1000 != 0 {
            Some(self.ip)
        } else {
            None
        }
    }
    pub fn flags(&self) -> Option<u16> {
        if self.reg_mask & 0x2000 != 0 {
            Some(self.flags)
        } else {
            None
        }
    }

    pub fn is_valid(&self) -> bool {
        if self.reg_mask & 0x2000 != 0 {
            // We have flags
            if self.flags & 0x0002 == 0 {
                // Reserved flag bit 1 cannot be clear
                return false;
            }
        }
        true
    }
}

pub struct MooRegisters16Printer<'a> {
    pub regs: &'a MooRegisters16,
    pub cpu_type: MooCpuType,
    pub diff: Option<&'a MooRegisters16>,
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

impl Display for MooRegisters16Printer<'_> {
    #[rustfmt::skip]
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let reg_str = format!(
            "AX:{}{:04X} BX:{}{:04X} CX:{}{:04X} DX:{}{:04X}\n\
             SP:{}{:04X} BP:{}{:04X} SI:{}{:04X} DI:{}{:04X}\n\
             CS:{}{:04X} DS:{}{:04X} ES:{}{:04X} SS:{}{:04X}\n\
             IP:{}{:04X}\n\
             FLAGS:{}{:04X} ",
            diff_chr!(self, ax), self.regs.ax,
            diff_chr!(self, bx), self.regs.bx,
            diff_chr!(self, cx), self.regs.cx,
            diff_chr!(self, dx), self.regs.dx,
            diff_chr!(self, sp), self.regs.sp,
            diff_chr!(self, bp), self.regs.bp,
            diff_chr!(self, si), self.regs.si,
            diff_chr!(self, di), self.regs.di,
            diff_chr!(self, cs), self.regs.cs,
            diff_chr!(self, ds), self.regs.ds,
            diff_chr!(self, es), self.regs.es,
            diff_chr!(self, ss), self.regs.ss,
            diff_chr!(self, ip), self.regs.ip,
            diff_chr!(self, flags), self.regs.flags,
        );

        // Expand flag info
        let f = self.regs.flags;
        let c_chr = if MooRegisters16::FLAG_CARRY & f != 0 { 'C' } else { 'c' };
        let p_chr = if MooRegisters16::FLAG_PARITY & f != 0 { 'P' } else { 'p' };
        let a_chr = if MooRegisters16::FLAG_AUX_CARRY & f != 0 {
            'A'
        } else {
            'a'
        };
        let z_chr = if MooRegisters16::FLAG_ZERO & f != 0 { 'Z' } else { 'z' };
        let s_chr = if MooRegisters16::FLAG_SIGN & f != 0 { 'S' } else { 's' };
        let t_chr = if MooRegisters16::FLAG_TRAP & f != 0 { 'T' } else { 't' };
        let i_chr = if MooRegisters16::FLAG_INT_ENABLE & f != 0 {
            'I'
        } else {
            'i'
        };
        let d_chr = if MooRegisters16::FLAG_DIRECTION & f != 0 {
            'D'
        } else {
            'd'
        };
        let o_chr = if MooRegisters16::FLAG_OVERFLOW & f != 0 {
            'O'
        } else {
            'o'
        };
        let m_chr = if self.cpu_type.is_intel() {
            if matches!(self.cpu_type, MooCpuType::Intel80286) {
                if MooRegisters16::FLAG_F15 & f != 0 {
                    '1'
                } else {
                    '0'
                }
            } else {
                '1'
            }
        } else {
            if f & MooRegisters16::FLAG_MODE != 0 {
                'M'
            } else {
                'm'
            }
        };

        let nt_chr = if f & MooRegisters16::FLAG_NT != 0 { '1' } else { '0' };
        let iopl0_chr = if f & MooRegisters16::FLAG_IOPL0 != 0 { '1' } else { '0' };
        let iopl1_chr = if f & MooRegisters16::FLAG_IOPL1 != 0 { '1' } else { '0' };

        write!(
            fmt,
            "{reg_str}{m_chr}{nt_chr}{iopl1_chr}{iopl0_chr}\
            {o_chr}{d_chr}{i_chr}{t_chr}{s_chr}{z_chr}0{a_chr}0{p_chr}1{c_chr}",
        )
    }
}
