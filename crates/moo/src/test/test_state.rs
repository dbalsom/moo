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

use std::io::{Cursor, Seek, Write};

use crate::{
    registers::*,
    types::{chunks::MooChunkType, effective_address::MooEffectiveAddress, MooRamEntries, MooRamEntry, MooStateType},
};

use binrw::BinResult;

/// A [MooTestState] represents a CPU state snapshot, either the initial state of the CPU before
/// test execution, or the final state of the CPU after test execution. The `s_type` field indicates
/// whether the state is initial or final, via the [MooStateType] enum.
#[derive(Clone, Default)]
pub struct MooTestState {
    /// The type of state (initial or final).
    pub s_type: MooStateType,
    /// The CPU registers for this state.
    pub regs: MooRegisters,
    /// The segment descriptors for this state, if applicable.
    pub descriptors: Option<MooDescriptors>,
    /// The effective address information for this state, if applicable.
    pub ea: Option<MooEffectiveAddress>,
    /// The instruction queue contents for this state.
    pub queue: Vec<u8>,
    /// The RAM contents for this state.
    pub ram: Vec<MooRamEntry>,
}

impl MooTestState {
    /// Create a new [MooTestState] instance.
    /// Typically called by the test generation code - you won't need to call this directly.
    pub fn new(
        s_type: MooStateType,
        regs_start: &MooRegistersInit,
        regs_final: Option<&MooRegistersInit>,
        ea: Option<MooEffectiveAddress>,
        queue: Vec<u8>,
        ram: Vec<MooRamEntry>,
    ) -> Self {
        let regs = if let Some(final_regs) = regs_final {
            MooRegisters::from((regs_start, final_regs))
        }
        else {
            MooRegisters::from(regs_start)
        };

        Self {
            s_type,
            regs,
            descriptors: None,
            ea,
            queue,
            ram,
        }
    }

    /// Return a reference to the [MooRegisters] for this state.
    pub fn regs(&self) -> &MooRegisters {
        &self.regs
    }

    /// Return a mutable reference to the [MooRegisters] for this state.
    pub fn regs_mut(&mut self) -> &mut MooRegisters {
        &mut self.regs
    }

    /// Return a reference to a slice representing the instruction queue contents for this state.
    pub fn queue(&self) -> &[u8] {
        &self.queue
    }

    /// Return a reference to a slice representing the RAM contents for this state as [MooRamEntry]s.
    pub fn ram(&self) -> &[MooRamEntry] {
        &self.ram
    }

    /// Return a reference to the [MooEffectiveAddress] for this state, if present.
    pub fn ea(&self) -> Option<&MooEffectiveAddress> {
        self.ea.as_ref()
    }

    /// Write this [MooTestState] to the given implementor of [Write] + [Seek] as a `MOO` `INIT` or
    /// `FINA` chunk, depending on the state's [MooStateType].
    pub fn write<WS: Write + Seek>(&self, writer: &mut WS) -> BinResult<()> {
        // Create a buffer to write our state data into, so we can write it to the final
        // chunk in one go.
        let mut state_buffer = Cursor::new(Vec::new());

        // Write the initial regs.
        let chunk_type = MooChunkType::from(&self.regs);
        chunk_type.write(&mut state_buffer, &self.regs)?;

        // Write the initial queue, if not empty.
        if !self.queue.is_empty() {
            MooChunkType::QueueState.write(&mut state_buffer, &self.queue)?;
        }

        // Write the EA chunk, if present.
        if let Some(ea) = &self.ea {
            MooChunkType::EffectiveAddress32.write(&mut state_buffer, ea)?;
        }

        // Write the RAM chunk.
        MooChunkType::Ram.write(
            &mut state_buffer,
            &MooRamEntries {
                entry_count: self.ram.len() as u32,
                entries: self.ram.clone(),
            },
        )?;

        match self.s_type {
            MooStateType::Initial => {
                // Write the initial state chunk.
                MooChunkType::InitialState.write(writer, &state_buffer.into_inner())?;
            }
            MooStateType::Final => {
                // Write the final state chunk.
                MooChunkType::FinalState.write(writer, &state_buffer.into_inner())?;
            }
        }

        Ok(())
    }
}
