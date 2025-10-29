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
    enums::{CheckErrorDetail, CheckErrorType},
    structs::CheckErrorStatus,
};
use moo::{prelude::*, types::MooCpuMode};
use std::io::Cursor;

use anyhow::Result;
use moo::types::{MooBusState, MooCpuFamily};

pub fn check_metadata(metadata: &MooFileMetadata) -> Vec<CheckErrorStatus> {
    let mut errors: Vec<CheckErrorStatus> = Vec::new();

    // Check that the CPU type is valid.
    let mnemonic_str = String::from_utf8_lossy(&metadata.mnemonic).trim().to_string();

    if mnemonic_str.is_empty() {
        errors.push(CheckErrorType::BadMetadata("Empty mnemonic in metadata!".to_string()).fixed(false));
    }

    // Additional metadata checks can go here.

    errors
}

pub fn check_test(
    index: usize,
    test: &mut MooTest,
    metadata: &MooFileMetadata,
    fix: bool,
) -> Result<Option<CheckErrorDetail>> {
    let mut errors: Vec<CheckErrorStatus> = Vec::new();

    check_test_universal(test, metadata, fix, &mut errors)?;

    let mode = test.cpu_mode(metadata.cpu_type);
    match mode {
        MooCpuMode::RealMode => {
            check_test_real(test, metadata, fix, &mut errors)?;
        }
        MooCpuMode::ProtectedMode => {
            check_test_protected(test, metadata, fix, &mut errors)?;
        }
        _ => {
            log::warn!("Unsupported CPU mode for test check: {:?}", mode);
        }
    }

    if errors.is_empty() {
        Ok(None)
    }
    else {
        Ok(Some(CheckErrorDetail::TestError {
            index,
            hash: test.hash_string(),
            errors,
        }))
    }
}

pub fn check_test_universal(
    test: &mut MooTest,
    metadata: &MooFileMetadata,
    fix: bool,
    errors: &mut Vec<CheckErrorStatus>,
) -> Result<()> {
    check_disassembly(test, metadata, fix, errors)?;

    if test.cycles().is_empty() {
        errors.push(CheckErrorType::CycleStateError("No cycle states present!".to_string()).fixed(false));
    }

    let initial_queue = test.initial_state().queue();
    if initial_queue.is_empty() {
        // Test is not prefetched

        // Check that first cycle has asserted ALE.
        if test.cycles().first().unwrap().pins0 & MooCycleState::PIN_ALE == 0 {
            errors.push(
                CheckErrorType::CycleStateError("First cycle missing ALE for non-prefetched test".to_string())
                    .fixed(false),
            );
        }

        // the first cycle should be a code fetch at CS:IP.
        if let Some(csip) = test.initial_state().regs().csip_linear_real() {
            if csip != test.cycles().first().unwrap().address_bus {
                errors.push(
                    CheckErrorType::CycleStateError(format!(
                        "First cycle address 0x{:05X} does not match CS:IP 0x{:05X}",
                        test.cycles().first().unwrap().address_bus,
                        csip
                    ))
                    .fixed(false),
                );
            }
        }
        else {
            // Not having a valid CS:IP is an error!
            errors.push(CheckErrorType::BadInitialState("No valid CS:IP in real mode".to_string()).fixed(false));
        }
    }

    let mut must_halt = false;

    let family = MooCpuFamily::from(metadata.cpu_type);
    match family {
        MooCpuFamily::Intel80286 => {
            // 286-specific universal checks can go here.
            must_halt = true;
        }
        MooCpuFamily::Intel80386 => {
            // 386+ specific universal checks can go here.
            must_halt = true;
        }
        _ => {}
    }

    if must_halt {
        // Check that the last cycle is a HALT bus cycle.
        let last_bus_state = test.cycles().last().unwrap().bus_state;

        if !matches!(metadata.cpu_type.decode_status(last_bus_state), MooBusState::HALT) {
            errors.push(CheckErrorType::CycleStateError("Last cycle is not a HALT bus state".to_string()).fixed(false));
        }
    }

    Ok(())
}

pub fn check_test_real(
    test: &mut MooTest,
    metadata: &MooFileMetadata,
    fix: bool,
    errors: &mut Vec<CheckErrorStatus>,
) -> Result<()> {
    let initial_queue = test.initial_state().queue();
    if initial_queue.is_empty() {}

    let family = MooCpuFamily::from(metadata.cpu_type);
    match family {
        MooCpuFamily::Intel80286 => {
            // Check that top four flag bits are cleared.
            let initial_flags = test.initial_state().regs().flags();

            if initial_flags & 0xF000 != 0 {
                // If not, we can fix it by clearing them (if --fix is enabled).
                let mut fixed = false;
                if fix {
                    let regs = test.initial_state_mut().regs_mut();

                    match regs {
                        MooRegisters::Sixteen(regs16) => {
                            log::trace!(
                                "Fixing initial flags for real mode test by clearing top four bits: {:04X} -> {:04X}",
                                initial_flags as u16,
                                (initial_flags as u16) & 0x0FFF,
                            );
                            regs16.set_flags((initial_flags as u16) & 0x0FFF);
                            fixed = true;
                        }
                        _ => log::error!("Unsupported register set for real mode flag fixup"),
                    }
                }

                errors.push(
                    CheckErrorType::BadInitialState(format!(
                        "Top four flag bits must be cleared in real mode. Initial flags are: {:04X}",
                        initial_flags as u16
                    ))
                    .fixed(fixed),
                );
            }

            // Check that the flag address for an exception is valid.
            let sp_linear_real = test.initial_state().regs().sp_linear_real();
            if let Some(exception) = test.exception_mut() {
                let flag_addr = exception.flag_address;

                if let Some(sp_addr) = sp_linear_real {
                    if flag_addr != (sp_addr - 2) {
                        let mut fixed = false;
                        if fix {
                            log::trace!(
                                "Fixing flag address for real mode test exception: 0x{:05X} -> 0x{:05X}",
                                flag_addr,
                                sp_addr - 2
                            );
                            exception.flag_address = sp_addr - 2;
                            fixed = true;
                        }

                        // log::warn!(
                        //     "Bad stack address! flag_addr=0x{:08X}, sp_addr=0x{:08X}",
                        //     flag_addr,
                        //     sp_addr
                        // );
                        errors.push(
                            CheckErrorType::BadFlagAddress {
                                flag_addr,
                                stack_addr: sp_addr,
                            }
                            .fixed(fixed),
                        );
                    }
                }
                else {
                    errors.push(CheckErrorType::BadInitialState("No valid SP in real mode".to_string()).fixed(false));
                }
            }
        }
        MooCpuFamily::Intel80386 => {
            // 386+ specific universal checks can go here.
        }
        _ => {}
    }

    Ok(())
}

pub fn check_test_protected(
    test: &MooTest,
    metadata: &MooFileMetadata,
    fix: bool,
    errors: &mut Vec<CheckErrorStatus>,
) -> Result<()> {
    Ok(())
}

pub fn check_disassembly(
    test: &mut MooTest,
    metadata: &MooFileMetadata,
    fix: bool,
    errors: &mut Vec<CheckErrorStatus>,
) -> Result<()> {
    use marty_dasm::prelude::*;

    // Check disassembly
    let test_name_trimmed = test.name().trim();
    if test_name_trimmed != test.name().trim() {
        errors.push(
            CheckErrorType::DisassemblyError(format!(
                "Test name has leading or trailing whitespace: '{}'",
                test.name()
            ))
            .fixed(false),
        );
    }

    let decode_vec = test.bytes().to_vec();
    if decode_vec.is_empty() {
        errors.push(CheckErrorType::DisassemblyError("No instruction bytes to decode!".to_string()).fixed(false));
        return Ok(());
    }

    let mut decoder = Decoder::new(
        Cursor::new(&decode_vec),
        DecoderOptions {
            cpu: CpuType::Intel80386,
            ..Default::default()
        },
    );
    let marty_i = match decoder.decode_next() {
        Ok(instr) => instr,
        Err(e) => {
            errors.push(
                CheckErrorType::DisassemblyError(format!(
                    "Failed to decode instruction '{}' [{:0X?}]: {}",
                    test.name(),
                    test.bytes(),
                    e
                ))
                .fixed(false),
            );
            return Ok(());
        }
    };

    let mut output = String::new();
    let options = FormatOptions {
        ip: test.initial_state().regs().csip_linear_real().unwrap_or(0),
        iced_mnemonics: true,
        ..FormatOptions::default()
    };

    NasmFormatter.format_instruction(&marty_i, &options, &mut output);

    if test_name_trimmed != output {
        // Disassembly does not match test name.
        errors.push(
            CheckErrorType::DisassemblyError(format!(
                "Disassembly does not match test name: '{}' != '{}'",
                test_name_trimmed, output
            ))
            .fixed(false),
        )
    }

    // if test_name_trimmed == "(bad)" {
    //     errors.push(CheckErrorType::DisassemblyError("No disassembly for instruction!".to_string()).fixed(false));
    // }

    Ok(())
}
