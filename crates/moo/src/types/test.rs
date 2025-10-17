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

use crate::{
    prelude::{MooCycleState, MooTestState},
    types::{
        flags::{MooCpuFlag, MooCpuFlagsDiff},
        MooException,
        MooRamEntries,
        MooRamEntry,
        MooRegister,
        MooRegisterDiff,
        MooRegisters,
        MooTestGenMetadata,
    },
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MooComparison {
    Equal,
    RegisterMismatch,
    CycleCountMismatch(usize, usize),
    CycleAddressMismatch(u32, u32),
    CycleBusMismatch(u8, u8),
    MemoryAddressMismatch(MooRamEntry, MooRamEntry),
    MemoryValueMismatch(MooRamEntry, MooRamEntry),
    ALEMismatch(usize, bool, bool),
}

pub struct MooTest {
    pub(crate) name: String,
    pub(crate) gen_metadata: Option<MooTestGenMetadata>,
    pub(crate) bytes: Vec<u8>,
    pub(crate) initial_state: MooTestState,
    pub(crate) final_state: MooTestState,
    pub(crate) cycles: Vec<MooCycleState>,
    pub(crate) exception: Option<MooException>,
    pub(crate) hash: Option<[u8; 20]>,
}

impl MooTest {
    pub fn new(
        name: String,
        gen_metadata: Option<MooTestGenMetadata>,
        bytes: &[u8],
        initial_state: MooTestState,
        final_state: MooTestState,
        cycles: &[MooCycleState],
        exception: Option<MooException>,
        hash: Option<[u8; 20]>,
    ) -> Self {
        Self {
            name,
            gen_metadata,
            bytes: bytes.to_vec(),
            initial_state,
            final_state,
            cycles: cycles.to_vec(),
            exception,
            hash,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn initial_regs(&self) -> &MooRegisters {
        &self.initial_state.regs
    }
    pub fn final_regs(&self) -> &MooRegisters {
        &self.final_state.regs
    }
    pub fn initial_mem_state(&self) -> &MooRamEntries {
        &self.initial_state.ram
    }
    pub fn final_mem_state(&self) -> &MooRamEntries {
        &self.final_state.ram
    }
    pub fn cycles(&self) -> &[MooCycleState] {
        &self.cycles
    }

    pub fn hash_string(&self) -> String {
        if let Some(hash) = &self.hash {
            hash.iter().map(|b| format!("{:02x}", b)).collect()
        }
        else {
            "##NOHASH##".to_string()
        }
    }

    pub fn exception(&self) -> Option<&MooException> {
        self.exception.as_ref()
    }

    pub fn compare(&self, other: &MooTest) -> MooComparison {
        if self.final_state.regs != other.final_state.regs {
            return MooComparison::RegisterMismatch;
        }
        if self.cycles.len() != other.cycles.len() {
            return MooComparison::CycleCountMismatch(self.cycles.len(), other.cycles.len());
        }
        for ((i, this_cycle), other_cycle) in self.cycles.iter().enumerate().zip(other.cycles.iter()) {
            // The address bus is inconsistent except at ALE, so only compare if ALE bit is set.
            if this_cycle.pins0 & MooCycleState::PIN_ALE != 0 {
                if other_cycle.pins0 & MooCycleState::PIN_ALE == 0 {
                    return MooComparison::ALEMismatch(i, true, false);
                }

                if this_cycle.address_bus != other_cycle.address_bus {
                    return MooComparison::CycleAddressMismatch(this_cycle.address_bus, other_cycle.address_bus);
                }

                if this_cycle.bus_state != other_cycle.bus_state {
                    return MooComparison::CycleBusMismatch(this_cycle.bus_state, other_cycle.bus_state);
                }
            }
            else if other_cycle.pins0 & MooCycleState::PIN_ALE != 0 {
                return MooComparison::ALEMismatch(i, false, true);
            }
        }

        for (this_ram_entry, other_ram_entry) in self
            .initial_state
            .ram
            .entries
            .iter()
            .zip(other.initial_state.ram.entries.iter())
        {
            if this_ram_entry.address != other_ram_entry.address {
                return MooComparison::MemoryAddressMismatch(*this_ram_entry, *other_ram_entry);
            }
            if this_ram_entry.value != other_ram_entry.value {
                return MooComparison::MemoryValueMismatch(*this_ram_entry, *other_ram_entry);
            }
        }

        MooComparison::Equal
    }

    pub fn diff_flags(&self) -> MooCpuFlagsDiff {
        let mut set_flags: Vec<MooCpuFlag> = Vec::new();
        let mut cleared_flags: Vec<MooCpuFlag> = Vec::new();

        let flags_changed = match (&self.initial_state.regs, &self.final_state.regs) {
            (MooRegisters::Sixteen(regs16_0), MooRegisters::Sixteen(regs16_1)) => {
                if let Some(flags) = regs16_1.flags() {
                    (regs16_0.flags as u32) ^ (flags as u32)
                }
                else {
                    return MooCpuFlagsDiff::default();
                }
            }
            (MooRegisters::ThirtyTwo(regs32_0), MooRegisters::ThirtyTwo(regs32_1)) => {
                if let Some(flags) = regs32_1.eflags() {
                    regs32_0.eflags ^ flags
                }
                else {
                    return MooCpuFlagsDiff::default();
                }
            }
            _ => 0,
        };

        if flags_changed == 0 {
            return MooCpuFlagsDiff::default();
        }

        //log::debug!("Flags changed: {:08X}", flags_changed);

        for i in 0..32 {
            let flag_mask = 1 << i;

            if flags_changed & flag_mask == 0 {
                continue;
            }

            // Check if flag is set
            let is_set = match &self.final_state.regs {
                MooRegisters::Sixteen(regs16_1) => (regs16_1.flags as u32) & flag_mask != 0,
                MooRegisters::ThirtyTwo(regs32_1) => regs32_1.eflags & flag_mask != 0,
            };

            if is_set {
                if let Some(flag) = MooCpuFlag::from_bit(i as u8) {
                    //log::debug!("Flag set: {:?}", flag);
                    set_flags.push(flag);
                }
            }

            // Check if flag is cleared
            let is_cleared = match &self.final_state.regs {
                MooRegisters::Sixteen(regs16_1) => (regs16_1.flags as u32) & flag_mask == 0,
                MooRegisters::ThirtyTwo(regs32_1) => regs32_1.eflags & flag_mask == 0,
            };

            if is_cleared {
                if let Some(flag) = MooCpuFlag::from_bit(i as u8) {
                    //log::debug!("Flag cleared: {:?}", flag);
                    cleared_flags.push(flag);
                }
            }
        }

        MooCpuFlagsDiff {
            set_flags,
            cleared_flags,
        }
    }

    pub fn diff_regs(&self) -> Vec<MooRegisterDiff> {
        let mut diff_regs = Vec::new();

        match (&self.initial_state.regs, &self.final_state.regs) {
            (MooRegisters::Sixteen(regs16_0), MooRegisters::Sixteen(regs16_1)) => {
                if regs16_0.ax != regs16_1.ax {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::EAX,
                        initial:  regs16_0.ax as u32,
                        r#final:  regs16_1.ax as u32,
                    });
                }
                if regs16_0.bx != regs16_1.bx {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::EBX,
                        initial:  regs16_0.bx as u32,
                        r#final:  regs16_1.bx as u32,
                    });
                }
                if regs16_0.cx != regs16_1.cx {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::ECX,
                        initial:  regs16_0.cx as u32,
                        r#final:  regs16_1.cx as u32,
                    });
                }
                if regs16_0.dx != regs16_1.dx {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::EDX,
                        initial:  regs16_0.dx as u32,
                        r#final:  regs16_1.dx as u32,
                    });
                }
                if regs16_0.si != regs16_1.si {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::ESI,
                        initial:  regs16_0.si as u32,
                        r#final:  regs16_1.si as u32,
                    });
                }
                if regs16_0.di != regs16_1.di {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::EDI,
                        initial:  regs16_0.di as u32,
                        r#final:  regs16_1.di as u32,
                    });
                }
                if regs16_0.bp != regs16_1.bp {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::EBP,
                        initial:  regs16_0.bp as u32,
                        r#final:  regs16_1.bp as u32,
                    });
                }
                if regs16_0.sp != regs16_1.sp {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::ESP,
                        initial:  regs16_0.sp as u32,
                        r#final:  regs16_1.sp as u32,
                    });
                }
                if regs16_0.ip != regs16_1.ip {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::EIP,
                        initial:  regs16_0.ip as u32,
                        r#final:  regs16_1.ip as u32,
                    });
                }
                if regs16_0.flags != regs16_1.flags {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::EFLAGS,
                        initial:  regs16_0.flags as u32,
                        r#final:  regs16_1.flags as u32,
                    });
                }
                if regs16_0.cs != regs16_1.cs {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::CS,
                        initial:  regs16_0.cs as u32,
                        r#final:  regs16_1.cs as u32,
                    });
                }
                if regs16_0.ds != regs16_1.ds {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::DS,
                        initial:  regs16_0.ds as u32,
                        r#final:  regs16_1.ds as u32,
                    });
                }
                if regs16_0.es != regs16_1.es {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::ES,
                        initial:  regs16_0.es as u32,
                        r#final:  regs16_1.es as u32,
                    });
                }
            }
            (MooRegisters::ThirtyTwo(regs32_0), MooRegisters::ThirtyTwo(regs32_1)) => {
                if let Some(cr0) = regs32_1.cr0() {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::CR0,
                        initial:  regs32_0.cr0,
                        r#final:  cr0,
                    });
                }
                if let Some(cr3) = regs32_1.cr3() {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::CR3,
                        initial:  regs32_0.cr3,
                        r#final:  cr3,
                    });
                }
                if let Some(eax) = regs32_1.eax() {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::EAX,
                        initial:  regs32_0.eax,
                        r#final:  eax,
                    });
                }
                if let Some(ebx) = regs32_1.ebx() {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::EBX,
                        initial:  regs32_0.ebx,
                        r#final:  ebx,
                    });
                }
                if let Some(ecx) = regs32_1.ecx() {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::ECX,
                        initial:  regs32_0.ecx,
                        r#final:  ecx,
                    });
                }
                if let Some(edx) = regs32_1.edx() {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::EDX,
                        initial:  regs32_0.edx,
                        r#final:  edx,
                    });
                }
                if let Some(esi) = regs32_1.esi() {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::ESI,
                        initial:  regs32_0.esi,
                        r#final:  esi,
                    });
                }
                if let Some(edi) = regs32_1.edi() {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::EDI,
                        initial:  regs32_0.edi,
                        r#final:  edi,
                    });
                }
                if let Some(ebp) = regs32_1.ebp() {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::EBP,
                        initial:  regs32_0.ebp,
                        r#final:  ebp,
                    });
                }
                if let Some(esp) = regs32_1.esp() {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::ESP,
                        initial:  regs32_0.esp,
                        r#final:  esp,
                    });
                }
                if let Some(eip) = regs32_1.eip() {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::EIP,
                        initial:  regs32_0.eip,
                        r#final:  eip,
                    });
                }
                if let Some(eflags) = regs32_1.eflags() {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::EFLAGS,
                        initial:  regs32_0.eflags,
                        r#final:  eflags,
                    });
                }
                if let Some(cs) = regs32_1.cs() {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::CS,
                        initial:  regs32_0.cs,
                        r#final:  cs as u32,
                    });
                }
                if let Some(ds) = regs32_1.ds() {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::DS,
                        initial:  regs32_0.ds,
                        r#final:  ds as u32,
                    });
                }
                if let Some(es) = regs32_1.es() {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::ES,
                        initial:  regs32_0.es,
                        r#final:  es as u32,
                    });
                }
                if let Some(fs) = regs32_1.fs() {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::FS,
                        initial:  regs32_0.fs,
                        r#final:  fs as u32,
                    });
                }
                if let Some(gs) = regs32_1.gs() {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::GS,
                        initial:  regs32_0.gs,
                        r#final:  gs as u32,
                    });
                }
                if let Some(ss) = regs32_1.ss() {
                    diff_regs.push(MooRegisterDiff {
                        register: MooRegister::SS,
                        initial:  regs32_0.ss,
                        r#final:  ss as u32,
                    });
                }
            }
            _ => {
                // Different types, cannot compare
            }
        }

        diff_regs
    }
}
