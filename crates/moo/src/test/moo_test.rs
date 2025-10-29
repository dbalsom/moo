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
    prelude::MooCycleState,
    registers::{MooRegister, MooRegisterDiff, MooRegisters},
    test::{comparison::MooComparison, test_state::MooTestState},
    types::{
        chunks::{MooBytesChunk, MooChunkType, MooNameChunk, MooTestChunk},
        flags::{MooCpuFlag, MooCpuFlagsDiff},
        MooCpuFamily,
        MooCpuMode,
        MooException,
        MooOperandSize,
        MooSegmentSize,
        MooTestGenMetadata,
    },
};
use binrw::{BinResult, BinWrite};
use sha1::Digest;
use std::io::{Cursor, Seek, Write};

macro_rules! push_or_return {
    ($vec:expr, $item:expr, $ret:expr) => {{
        $vec.push($item);
        if $ret {
            return $vec;
        }
    }};
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

/// An individual test case for a particular CPU.
/// A `MooTest` at minimum contains:
///  - A human-readable name
///  - The raw bytes that comprise the instruction(s) being tested
///  - A initial CPU state containing register and memory values (as a [MooTestState])
///  - A final CPU state containing register and memory values (as a [MooTestState])
///  - A sequence of [MooCycleState] entries representing the cpu cycles that occurred
///    during execution of the instruction(s)
///  - An optional [MooException] if an exception was raised during execution
///  - A SHA-1 hash of the test used to uniquely identify it
impl MooTest {
    /// Create a new [MooTest].
    /// # Arguments
    /// * `name` - A human-readable name for the test. This is typically the disassembly of the instruction(s) being tested.
    /// * `gen_metadata` - An optional [MooTestGenMetadata](crate::types::MooTestGenMetadata) struct containing information about how the test was generated.
    /// * `bytes` - The raw bytes that comprise the instruction(s) being tested.
    /// * `initial_state` - A [MooTestState] struct describing the initial CPU state before execution.
    /// * `final_state` - A [MooTestState] struct describing the final CPU state after execution.
    /// * `cycles` - A slice of [MooCycleState] structs representing the cpu cycles that occurred during execution.
    /// * `exception` - An optional [MooException] if an exception was raised during execution.
    /// * `hash` - An optional SHA-1 hash of the test used to uniquely identify it. If not provided, the hash will be calculated when the test is written using [MooTestFile::write](crate::prelude::MooTestFile::write).
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

    /// Retrieve the human-readable name of the test (typically the disassembly of the instruction(s) being tested).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Retrieve the optional test generation metadata for the test.
    pub fn gen_metadata(&self) -> Option<&MooTestGenMetadata> {
        self.gen_metadata.as_ref()
    }

    /// Retrieve a reference to a slice of the raw bytes that comprise the instruction(s) being tested.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Retrieve a reference to the [MooTestState] representing the initial CPU state.
    pub fn initial_state(&self) -> &MooTestState {
        &self.initial_state
    }

    /// Retrieve a mutable reference to the [MooTestState] representing the initial CPU state.
    pub fn initial_state_mut(&mut self) -> &mut MooTestState {
        &mut self.initial_state
    }

    /// Retrieve a reference to the [MooTestState] representing the final CPU state.
    pub fn final_state(&self) -> &MooTestState {
        &self.final_state
    }

    /// Retrieve a mutable reference to the [MooTestState] representing the final CPU state.
    pub fn final_state_mut(&mut self) -> &mut MooTestState {
        &mut self.final_state
    }

    /// Retrieve a reference to a slice of the [MooCycleState] entries representing the cpu cycles
    /// that occurred during execution.
    pub fn cycles(&self) -> &[MooCycleState] {
        &self.cycles
    }

    /// Retrieve the SHA-1 hash of the test as a hexadecimal ASCII string.
    /// If the hash is not available, returns the literal string "##NOHASH##".
    pub fn hash_string(&self) -> String {
        if let Some(hash) = &self.hash {
            hash.iter().map(|b| format!("{:02x}", b)).collect()
        }
        else {
            "##NOHASH##".to_string()
        }
    }

    /// Retrieve an optional reference to any [MooException].
    /// A [MooException] will be present if an exception was raised during test execution.
    pub fn exception(&self) -> Option<&MooException> {
        self.exception.as_ref()
    }

    /// Retrieve an optional mutable reference to any [MooException].
    /// A [MooException] will be present if an exception was raised during test execution.
    pub fn exception_mut(&mut self) -> Option<&mut MooException> {
        self.exception.as_mut()
    }

    /// Compare two MooTests and return a vector of differences as [MooComparison] entries.
    /// Arguments:
    /// * `other` - The other [MooTest] to compare against.
    /// * `return_first` - If true, the function will return after finding the first difference.
    /// Returns:
    /// A vector of [MooComparison] entries representing the differences found between the two tests.
    /// If no differences are found, the vector will be empty.
    /// If `return_first` is true, the vector will contain at most one entry.
    pub fn compare(&self, other: &MooTest, return_first: bool) -> Vec<MooComparison> {
        let mut differences = Vec::new();

        if self.final_state.regs != other.final_state.regs {
            push_or_return!(differences, MooComparison::RegisterMismatch, return_first);
        }
        if self.cycles.len() != other.cycles.len() {
            push_or_return!(
                differences,
                MooComparison::CycleCountMismatch(self.cycles.len(), other.cycles.len()),
                return_first
            );
        }
        for ((i, this_cycle), other_cycle) in self.cycles.iter().enumerate().zip(other.cycles.iter()) {
            // The address bus is inconsistent except at ALE, so only compare if ALE bit is set.
            if this_cycle.pins0 & MooCycleState::PIN_ALE != 0 {
                if other_cycle.pins0 & MooCycleState::PIN_ALE == 0 {
                    push_or_return!(differences, MooComparison::ALEMismatch(i, true, false), return_first);
                }

                if this_cycle.address_bus != other_cycle.address_bus {
                    push_or_return!(
                        differences,
                        MooComparison::CycleAddressMismatch(this_cycle.address_bus, other_cycle.address_bus),
                        return_first
                    );
                }

                if this_cycle.bus_state != other_cycle.bus_state {
                    push_or_return!(
                        differences,
                        MooComparison::CycleBusMismatch(this_cycle.bus_state, other_cycle.bus_state),
                        return_first
                    );
                }
            }
            else if other_cycle.pins0 & MooCycleState::PIN_ALE != 0 {
                push_or_return!(differences, MooComparison::ALEMismatch(i, false, true), return_first);
            }
        }

        for (this_ram_entry, other_ram_entry) in self
            .initial_state()
            .ram()
            .iter()
            .zip(other.initial_state().ram().iter())
        {
            if this_ram_entry.address != other_ram_entry.address {
                push_or_return!(
                    differences,
                    MooComparison::MemoryAddressMismatch(*this_ram_entry, *other_ram_entry),
                    return_first
                );
            }
            if this_ram_entry.value != other_ram_entry.value {
                push_or_return!(
                    differences,
                    MooComparison::MemoryValueMismatch(*this_ram_entry, *other_ram_entry),
                    return_first
                );
            }
        }

        differences
    }

    /// Determine the differences in CPU flags between the initial and final states.
    /// Returns a [MooCpuFlagsDiff] struct containing the flags that were set, cleared,
    /// and those that remained unmodified.
    pub fn diff_flags(&self) -> MooCpuFlagsDiff {
        let mut set_flags: Vec<MooCpuFlag> = Vec::new();
        let mut cleared_flags: Vec<MooCpuFlag> = Vec::new();
        let mut unmodified_set_flags: Vec<MooCpuFlag> = Vec::new();
        let mut unmodified_cleared_flags: Vec<MooCpuFlag> = Vec::new();

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

            // Check if flag is set
            let is_set = match &self.final_state.regs {
                MooRegisters::Sixteen(regs16_1) => (regs16_1.flags as u32) & flag_mask != 0,
                MooRegisters::ThirtyTwo(regs32_1) => regs32_1.eflags & flag_mask != 0,
            };

            // Check if flag is cleared
            let is_cleared = match &self.final_state.regs {
                MooRegisters::Sixteen(regs16_1) => (regs16_1.flags as u32) & flag_mask == 0,
                MooRegisters::ThirtyTwo(regs32_1) => regs32_1.eflags & flag_mask == 0,
            };

            if let Some(flag) = MooCpuFlag::from_bit(i as u8) {
                // Check if flags are unmodified
                if flags_changed & flag_mask == 0 {
                    if is_set {
                        unmodified_set_flags.push(flag);
                    }
                    else if is_cleared {
                        unmodified_cleared_flags.push(flag);
                    }
                    continue;
                }

                if is_set {
                    set_flags.push(flag);
                }

                if is_cleared {
                    cleared_flags.push(flag);
                }
            }
        }

        MooCpuFlagsDiff {
            set: set_flags,
            cleared: cleared_flags,
            unmodified_set: unmodified_set_flags,
            unmodified_cleared: unmodified_cleared_flags,
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

    pub fn cpu_mode(&self, _cpu_family: impl Into<MooCpuFamily>) -> MooCpuMode {
        // A lack of any descriptors indicates real mode.
        if self.initial_state.descriptors.is_none() {
            return MooCpuMode::RealMode;
        }
        else {
            // For 286, we need to look at the MSW register mode bit.
            // For 386, we need to look at the CR0 bits and flag bits.
        }
        MooCpuMode::RealMode
    }

    pub fn segment_size(&self, cpu_family: impl Into<MooCpuFamily>) -> MooSegmentSize {
        match self.cpu_mode(cpu_family) {
            MooCpuMode::RealMode => MooSegmentSize::Sixteen,
            MooCpuMode::ProtectedMode => {
                // In protected mode, segment size is determined by the descriptor.
                if let Some(_descriptors) = &self.initial_state.descriptors {
                    // if let Some(cs_descriptor) = descriptors.get(&MooRegister::CS) {
                    //     return cs_descriptor.size;
                    // }
                }
                MooSegmentSize::Sixteen // Default to 16 if unknown
            }
            _ => MooSegmentSize::Sixteen, // Default to 16 for other modes
        }
    }

    pub fn operand_size(&self, cpu_family: impl Into<MooCpuFamily>) -> MooOperandSize {
        let cpu_family = cpu_family.into();
        match cpu_family {
            MooCpuFamily::Intel8086 | MooCpuFamily::NecV30 | MooCpuFamily::Intel80186 | MooCpuFamily::Intel80286 => {
                MooOperandSize::Sixteen
            }
            MooCpuFamily::Intel80386 => self
                .segment_size(cpu_family)
                .operand_size(self.has_operand_size_override(cpu_family)),
        }
    }

    pub fn has_operand_size_override(&self, cpu_family: impl Into<MooCpuFamily>) -> bool {
        let cpu_family = cpu_family.into();
        match cpu_family {
            MooCpuFamily::Intel8086 | MooCpuFamily::NecV30 | MooCpuFamily::Intel80186 | MooCpuFamily::Intel80286 => {
                false
            }
            MooCpuFamily::Intel80386 => {
                // In 386 mode, check if the operand size override prefix (0x66) is present in the instruction bytes.
                self.bytes.contains(&0x66)
            }
        }
    }

    pub fn has_address_size_override(&self, cpu_family: impl Into<MooCpuFamily>) -> bool {
        let cpu_family = cpu_family.into();
        match cpu_family {
            MooCpuFamily::Intel8086 | MooCpuFamily::NecV30 | MooCpuFamily::Intel80186 | MooCpuFamily::Intel80286 => {
                false
            }
            MooCpuFamily::Intel80386 => {
                // In 386 mode, check if the address size override prefix (0x67) is present in the instruction bytes.
                self.bytes.contains(&0x67)
            }
        }
    }

    /// Write a [MooTest] to an implementor of [Write](std::io::Write) + [Seek](std::io::Seek).
    /// Arguments:
    /// * `index` - The index of the test.
    /// * `writer` - The writer to write the MOO file to.
    /// * `preserve_hash` - If true, preserves the existing test hash, if present. If false, the
    ///      test hash will be recalculated from the test data. The test hash will be recalculated if
    ///      missing, regardless of this flag.
    pub fn write<WS: Write + Seek>(&self, index: usize, writer: &mut WS, preserve_hash: bool) -> BinResult<()> {
        let mut test_buffer = Cursor::new(Vec::new());

        // Write the test chunk body.
        MooTestChunk { index: index as u32 }.write(&mut test_buffer)?;

        // Write the generator metadata chunk if present.
        if let Some(gen_metadata) = &self.gen_metadata {
            MooChunkType::GeneratorMetadata.write(&mut test_buffer, gen_metadata)?;
        }

        // Write the name chunk.
        let name_chunk = MooNameChunk {
            len:  self.name.len() as u32,
            name: self.name.clone(),
        };
        MooChunkType::Name.write(&mut test_buffer, &name_chunk)?;

        // Write the bytes chunk.
        let bytes_chunk = MooBytesChunk {
            len:   self.bytes.len() as u32,
            bytes: self.bytes.clone(),
        };
        MooChunkType::Bytes.write(&mut test_buffer, &bytes_chunk)?;

        // Write the initial state chunk.
        self.initial_state.write(&mut test_buffer)?;

        // Write the final state chunk.
        self.final_state.write(&mut test_buffer)?;

        let mut cycle_buffer = Cursor::new(Vec::new());
        // Write the count of cycles to the cycle buffer.
        (self.cycles.len() as u32).write_le(&mut cycle_buffer)?;
        // Write all the cycles to the cycle buffer.
        for cycle in &self.cycles {
            cycle.write(&mut cycle_buffer)?;
        }

        // Write the cycles chunk.
        MooChunkType::CycleStates.write(&mut test_buffer, &cycle_buffer.into_inner())?;

        // If an exception is present, write the exception chunk.
        if let Some(exception) = &self.exception {
            MooChunkType::Exception.write(&mut test_buffer, exception)?;
        }

        if preserve_hash && self.hash.is_some() {
            // Write the existing hash chunk.
            MooChunkType::Hash.write(&mut test_buffer, self.hash.as_ref().unwrap())?;
        }
        else {
            // Create the SHA1 hash from the current state of the test buffer.
            let hash = sha1::Sha1::digest(&test_buffer.get_ref()).to_vec();
            MooChunkType::Hash.write(&mut test_buffer, &hash)?;
        }

        // Write the test chunk.
        MooChunkType::TestHeader.write(writer, &test_buffer.into_inner())?;

        Ok(())
    }
}
