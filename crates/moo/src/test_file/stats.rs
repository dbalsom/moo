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
use super::MooTestFile;
use crate::{
    prelude::MooCycleState,
    types::{flags::MooCpuFlag, MooBusState, MooRegister},
};
use std::collections::HashSet;

pub struct MooTestFileStats {
    pub test_count: usize,
    pub total_cycles: usize,
    pub min_cycles: usize,
    pub max_cycles: usize,
    pub avg_cycles: f64,
    pub mem_reads: usize,
    pub mem_writes: usize,
    pub code_fetches: usize,
    pub io_reads: usize,
    pub io_writes: usize,
    pub wait_states: usize,

    pub exceptions_seen: Vec<u8>,
    pub registers_modified: Vec<MooRegister>,
    pub flags_set: Vec<MooCpuFlag>,
    pub flags_cleared: Vec<MooCpuFlag>,
    pub flags_modified: Vec<MooCpuFlag>,
    pub flags_always_set: Vec<MooCpuFlag>,
    pub flags_always_cleared: Vec<MooCpuFlag>,
}

fn into_sorted_vec<T: Ord>(set: HashSet<T>) -> Vec<T> {
    let mut v: Vec<T> = set.into_iter().collect();
    v.sort_unstable();
    v
}

impl MooTestFile {
    pub fn calc_stats(&mut self) -> MooTestFileStats {
        let test_ct = self.tests.len();

        let total_cycles = self.tests.iter().map(|t| t.cycles.len()).sum();
        let min_cycles = self.tests.iter().map(|t| t.cycles.len()).min().unwrap_or(0);
        let max_cycles = self.tests.iter().map(|t| t.cycles.len()).max().unwrap_or(0);
        let avg_cycles = if test_ct > 0 {
            total_cycles as f64 / test_ct as f64
        }
        else {
            0.0
        };

        let registers_modified: HashSet<MooRegister> = self
            .tests
            .iter()
            .filter(|t| t.exception.is_none())
            .flat_map(|t| t.diff_regs().iter().map(|diff| diff.register()).collect::<Vec<_>>())
            .collect();

        log::debug!("Calculated registers modified: {:?}", registers_modified);

        let (mem_reads, mem_writes, code_fetches, io_reads, io_writes) = if self.arch.contains("386") {
            // Only count read signal on ALE.
            let mem_reads = self
                .tests
                .iter()
                .map(|t| {
                    t.cycles
                        .iter()
                        .filter(|c| {
                            c.ale()
                                && c.bus_state(self.cpu_type) == MooBusState::MEMR
                                && (c.memory_status & MooCycleState::MRDC_BIT != 0)
                        })
                        .count()
                })
                .sum();

            let mem_writes = self
                .tests
                .iter()
                .map(|t| {
                    t.cycles
                        .iter()
                        .filter(|c| c.ale() && (c.memory_status & MooCycleState::MWTC_BIT != 0))
                        .count()
                })
                .sum();

            let code_fetches = self
                .tests
                .iter()
                .map(|t| {
                    t.cycles
                        .iter()
                        .filter(|c| {
                            c.ale()
                                && c.is_code_fetch(self.cpu_type)
                                && (c.memory_status & MooCycleState::MRDC_BIT != 0)
                        })
                        .count()
                })
                .sum();

            let io_reads = self
                .tests
                .iter()
                .map(|t| {
                    t.cycles
                        .iter()
                        .filter(|c| c.ale() && (c.io_status & MooCycleState::IORC_BIT != 0))
                        .count()
                })
                .sum();

            let io_writes = self
                .tests
                .iter()
                .map(|t| {
                    t.cycles
                        .iter()
                        .filter(|c| c.ale() && (c.io_status & MooCycleState::IOWC_BIT != 0))
                        .count()
                })
                .sum();

            (mem_reads, mem_writes, code_fetches, io_reads, io_writes)
        }
        else {
            // Other CPUs can wait for PASV bus to signal completed read/write.
            let mem_reads = self
                .tests
                .iter()
                .map(|t| {
                    t.cycles
                        .iter()
                        .filter(|c| {
                            c.bus_state(self.cpu_type) == MooBusState::PASV
                                && (c.memory_status & MooCycleState::MRDC_BIT != 0)
                        })
                        .count()
                })
                .sum();

            let mem_writes = self
                .tests
                .iter()
                .map(|t| {
                    t.cycles
                        .iter()
                        .filter(|c| {
                            c.bus_state(self.cpu_type) == MooBusState::PASV
                                && (c.memory_status & MooCycleState::MWTC_BIT != 0)
                        })
                        .count()
                })
                .sum();

            let code_fetches = self
                .tests
                .iter()
                .map(|t| {
                    t.cycles
                        .iter()
                        .filter(|c| {
                            c.bus_state(self.cpu_type) == MooBusState::PASV
                                && (c.memory_status & MooCycleState::MRDC_BIT != 0)
                                && c.is_code_fetch(self.cpu_type)
                        })
                        .count()
                })
                .sum();

            let io_reads = self
                .tests
                .iter()
                .map(|t| {
                    t.cycles
                        .iter()
                        .filter(|c| {
                            c.bus_state(self.cpu_type) == MooBusState::PASV
                                && (c.io_status & MooCycleState::IORC_BIT != 0)
                        })
                        .count()
                })
                .sum();

            let io_writes = self
                .tests
                .iter()
                .map(|t| {
                    t.cycles
                        .iter()
                        .filter(|c| {
                            c.bus_state(self.cpu_type) == MooBusState::PASV
                                && (c.io_status & MooCycleState::IOWC_BIT != 0)
                        })
                        .count()
                })
                .sum();

            (mem_reads, mem_writes, code_fetches, io_reads, io_writes)
        };

        let exceptions_seen = self
            .tests
            .iter()
            .filter_map(|t| {
                if let Some(exception) = &t.exception {
                    Some(exception.exception_num)
                }
                else {
                    None
                }
            })
            .collect();

        let (flags_set, flags_cleared): (HashSet<_>, HashSet<_>) = self.tests.iter().fold(
            (HashSet::default(), HashSet::default()),
            |(mut set_acc, mut clr_acc), t| {
                let fd = t.diff_flags();
                set_acc.extend(fd.set_flags.iter().cloned());
                clr_acc.extend(fd.cleared_flags.iter().cloned());
                (set_acc, clr_acc)
            },
        );

        // Flags that were set but never cleared; and cleared but never set.
        let flags_always_set: HashSet<_> = flags_set.difference(&flags_cleared).cloned().collect();
        let flags_always_cleared: HashSet<_> = flags_cleared.difference(&flags_set).cloned().collect();

        let flags_modified: HashSet<_> = flags_set.union(&flags_cleared).cloned().collect();

        MooTestFileStats {
            test_count: test_ct,
            total_cycles,
            min_cycles,
            max_cycles,
            avg_cycles,
            mem_reads,
            mem_writes,
            code_fetches,
            io_reads,
            io_writes,
            wait_states: 0,
            exceptions_seen,
            registers_modified: into_sorted_vec(registers_modified),
            flags_set: into_sorted_vec(flags_set),
            flags_cleared: into_sorted_vec(flags_cleared),
            flags_modified: into_sorted_vec(flags_modified),
            flags_always_set: into_sorted_vec(flags_always_set),
            flags_always_cleared: into_sorted_vec(flags_always_cleared),
        }
    }
}
