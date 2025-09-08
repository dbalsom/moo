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

use crate::types::chunks::MooChunkType;
use crate::types::{MooRamEntries, MooRamEntry, MooRegisters, MooRegistersInit, MooStateType};
use binrw::BinResult;
use std::io::{Cursor, Write};

#[derive(Clone, Default)]
pub struct MooTestState {
    pub s_type: MooStateType,
    pub regs: MooRegisters,
    pub queue: Vec<u8>,
    pub ram: MooRamEntries,
}

impl MooTestState {
    pub fn new(
        s_type: MooStateType,
        regs_start: &MooRegistersInit,
        regs_final: Option<&MooRegistersInit>,
        queue: Vec<u8>,
        ram: Vec<MooRamEntry>,
    ) -> Self {
        let regs = if let Some(final_regs) = regs_final {
            MooRegisters::from((regs_start, final_regs))
        } else {
            MooRegisters::from(regs_start)
        };

        let ram_entries = MooRamEntries {
            entry_count: ram.len() as u32,
            entries: ram,
        };
        Self {
            s_type,
            regs,
            queue,
            ram: ram_entries,
        }
    }

    pub fn regs(&self) -> &MooRegisters {
        &self.regs
    }

    pub fn queue(&self) -> &[u8] {
        &self.queue
    }

    pub fn ram(&self) -> &[MooRamEntry] {
        &self.ram.entries
    }

    pub fn write<W: Write + std::io::Seek>(&self, writer: &mut W) -> BinResult<()> {
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

        // Write the RAM chunk.
        MooChunkType::Ram.write(&mut state_buffer, &self.ram)?;

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
