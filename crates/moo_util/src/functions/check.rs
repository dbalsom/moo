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
    commands::check::args::CheckParams,
    enums::{CheckErrorDetail, CheckErrorType},
    structs::CheckErrorStatus,
};
use std::{io::Cursor, path::Path};

use crate::file::group_extension_from_path;
use anyhow::Result;
use moo::{
    prelude::*,
    types::{MooBusState, MooCpuFamily, MooCpuMode, MooRamEntries},
};

pub fn check_metadata(metadata: &mut MooFileMetadata, file_path: impl AsRef<Path>, fix: bool) -> Vec<CheckErrorStatus> {
    let mut errors: Vec<CheckErrorStatus> = Vec::new();

    // Check that the CPU type is valid.
    let mnemonic_str = String::from_utf8_lossy(&metadata.mnemonic).trim().to_string();

    if mnemonic_str.is_empty() {
        errors.push(CheckErrorType::BadMetadata("Empty mnemonic in metadata!".to_string()).fixed(false));
    }

    // Additional metadata checks can go here.
    let extension = group_extension_from_path(&file_path);

    if metadata.group_extension() != extension {
        let mut fixed = false;

        let error_str = format!(
            "File group extension '{:?}' does not match metadata group extension '{:?}'",
            extension,
            metadata.group_extension()
        );

        if fix {
            metadata.extension = extension.unwrap_or(0xFF);
            fixed = true;
        }

        errors.push(CheckErrorType::BadMetadata(error_str).fixed(fixed));
    }

    errors
}

pub fn check_test(
    index: usize,
    test: &mut MooTest,
    metadata: &MooFileMetadata,
    opts: &CheckParams,
) -> Result<Option<CheckErrorDetail>> {
    let mut errors: Vec<CheckErrorStatus> = Vec::new();

    check_test_universal(test, metadata, opts, &mut errors)?;

    let mode = test.cpu_mode(metadata.cpu_type);
    match mode {
        MooCpuMode::RealMode => {
            check_test_real(test, metadata, opts.fix, &mut errors)?;
        }
        MooCpuMode::ProtectedMode => {
            check_test_protected(test, metadata, opts.fix, &mut errors)?;
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
    opts: &CheckParams,
    errors: &mut Vec<CheckErrorStatus>,
) -> Result<()> {
    check_disassembly(test, metadata, opts, errors)?;

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
    _test: &MooTest,
    _metadata: &MooFileMetadata,
    _fix: bool,
    _errors: &mut Vec<CheckErrorStatus>,
) -> Result<()> {
    Ok(())
}

pub fn check_disassembly(
    test: &mut MooTest,
    _metadata: &MooFileMetadata,
    opts: &CheckParams,
    errors: &mut Vec<CheckErrorStatus>,
) -> Result<()> {
    use marty_dasm::prelude::*;

    // Check disassembly
    let test_name = test.name().to_string();
    let test_name_trimmed = test_name.trim();
    if test_name_trimmed != test_name {
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

    let decoder_opts = DecoderOptions {
        cpu: CpuType::Intel80386,
        ..Default::default()
    };
    let mut decoder = Decoder::new(Cursor::new(&decode_vec), decoder_opts);

    let decode_result = decoder.decode_next();

    let log_decode_err = |test: &MooTest, e: &mut Vec<CheckErrorStatus>, fixed: bool| {
        e.push(
            CheckErrorType::DisassemblyError(format!(
                "Failed to decode instruction '{}' [{:0X?}]",
                test.name(),
                test.bytes(),
            ))
            .fixed(fixed),
        );
    };

    let mut output = String::new();
    let options = FormatOptions {
        ip: test.initial_state().regs().csip_linear_real().unwrap_or(0),
        iced_mnemonics: true,
        ..FormatOptions::default()
    };

    let marty_i = match decode_result {
        Ok(instr) => instr,
        Err(_e) => {
            // Decode failed, probably due to insufficient bytes.
            // Attempt to expand the bytes array by reading fetches from the initial RAM state.
            let ram = test.initial_state().ram.clone();
            let ram_entries = MooRamEntries::from(ram.as_slice());

            if opts.fix {
                if let Some(inst_offset) = ram_entries.find(test.bytes()) {
                    let fetches = ram_entries.get_consecutive_bytes(inst_offset);

                    let mut decoder = Decoder::new(Cursor::new(&fetches), decoder_opts);
                    match decoder.decode_next() {
                        Ok(instr) => {
                            log_decode_err(test, errors, true);
                            *test.bytes_mut() = instr.instruction_bytes.clone();

                            let mut output = String::new();
                            NasmFormatter.format_instruction(&instr, &options, &mut output);
                            *test.name_mut() = output;

                            instr
                        }
                        Err(_e) => {
                            log_decode_err(test, errors, false);
                            return Ok(());
                        }
                    }
                }
                else {
                    log_decode_err(test, errors, false);
                    return Ok(());
                }
            }
            else {
                log_decode_err(test, errors, false);
                return Ok(());
            }
        }
    };

    if opts.check_disassembly {
        NasmFormatter.format_instruction(&marty_i, &options, &mut output);

        if test_name_trimmed != output {
            // Disassembly does not match test name.
            let mut fixed = false;

            if opts.fix && opts.update_disassembly {
                *test.name_mut() = output.clone();
                fixed = true;
            }

            errors.push(
                CheckErrorType::DisassemblyError(format!(
                    "Disassembly does not match test name: '{}' != '{}'",
                    test_name_trimmed, output
                ))
                .fixed(fixed),
            )
        }
    }

    // if test_name_trimmed == "(bad)" {
    //     errors.push(CheckErrorType::DisassemblyError("No disassembly for instruction!".to_string()).fixed(false));
    // }

    Ok(())
}
