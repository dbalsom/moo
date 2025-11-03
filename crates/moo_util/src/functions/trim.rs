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
    schema_db::{EditSchemaRecord, SchemaDb},
};
use moo::prelude::{MooFileMetadata, MooTestFile};

pub fn trim_test(
    file: &mut MooTestFile,
    metadata: &MooFileMetadata,
    schema_db: &SchemaDb<EditSchemaRecord>,
    _opts: &EditParams,
) -> Result<bool, EditErrorDetail> {
    //let mut errors: Vec<EditErrorType> = Vec::new();
    let mut edited = false;

    let opcode = metadata.opcode as u16;
    let opcode_ext = metadata.group_extension().unwrap_or(0);

    if let Some(record) = schema_db.opcode(opcode, opcode_ext) {
        if let Some(count) = record.count {
            if count > 0 {
                if count < file.test_ct() as u32 {
                    log::debug!(
                        "Trimming tests for opcode {:02X}.{:1X} to count {}",
                        opcode,
                        opcode_ext,
                        count
                    );
                    edited = true;
                    file.trim_tests(count as usize);
                }
                else {
                    log::debug!(
                        "Not enough tests for opcode {:02X}.{:1X} per schema count: {} >= file count {}",
                        opcode,
                        opcode_ext,
                        count,
                        file.test_ct()
                    );
                }
            }
            else {
                log::warn!(
                    "Schema record for opcode {:02X}.{:1X} has zero test count: {}",
                    opcode,
                    opcode_ext,
                    count
                );
            }
        }
    }

    Ok(edited)
}
