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
    commands::edit::args::EditParams,
    enums::{EditErrorDetail, EditErrorType},
};

use crate::schema_db::{EditSchemaRecord, SchemaDb};
use moo::{
    registers::{MooRegisters, MooRegisters16, MooRegisters32},
    test_file::MooTestFile,
    types::{MooCpuFamily, MooFileMetadata},
};

pub fn add_global_mask(
    file: &mut MooTestFile,
    metadata: &MooFileMetadata,
    schema_db: &SchemaDb<EditSchemaRecord>,
    _opts: &EditParams,
) -> Result<bool, EditErrorDetail> {
    let mut errors: Vec<EditErrorType> = Vec::new();
    let mut edited = false;

    let opcode = metadata.opcode as u16;
    let opcode_ext = metadata.group_extension().unwrap_or(0);

    if let Some(record) = schema_db.opcode(opcode, opcode_ext) {
        if let Some(mask) = &record.f_umask {
            log::debug!(
                "Have flag mask for opcode {:02X}.{:1X}: {:04X}",
                opcode,
                opcode_ext,
                mask
            );

            match MooCpuFamily::from(metadata.cpu_type) {
                MooCpuFamily::Intel80386 => {
                    // 32-bit register mask
                    let mut mask_value = *mask;
                    if mask_value & 0xFFFF_0000 == 0 {
                        // If the upper 16 bits are not used, set them to 0xFFFF
                        mask_value |= 0xFFFF_0000;
                    }

                    let mask_32 = MooRegisters32::from_flag_mask(mask_value);
                    let registers = MooRegisters::ThirtyTwo(mask_32);
                    file.set_register_mask(registers);
                    edited = true;
                }
                _ => {
                    // 16-bit register mask
                    let mask_16 = MooRegisters16::from_flag_mask(*mask as u16);
                    let registers = MooRegisters::Sixteen(mask_16);
                    file.set_register_mask(registers);
                    edited = true;
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(edited)
    }
    else {
        Err(EditErrorDetail::FileError(errors))
    }
}
