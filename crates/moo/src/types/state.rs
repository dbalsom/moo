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
