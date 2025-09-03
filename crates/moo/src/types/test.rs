use crate::{
    prelude::{MooCycleState, MooTestState},
    types::{MooException, MooRamEntries, MooRamEntry, MooTestGenMetadata},
};
use crate::types::MooRegisters;

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
}
