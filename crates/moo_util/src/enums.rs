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
use crate::structs::CheckErrorStatus;
use std::fmt::Display;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum CheckErrorType {
    #[default]
    NoError,
    BadFlagAddress {
        flag_addr:  u32,
        stack_addr: u32,
    },
    BadInitialState(String),
    CycleStateError(String),
    BadMetadata(String),
    DisassemblyError(String),
}

impl Display for CheckErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckErrorType::NoError => write!(f, "No Error"),
            CheckErrorType::BadFlagAddress { flag_addr, stack_addr } => {
                let signed_diff = (*stack_addr as i64) - (*flag_addr as i64);
                write!(
                    f,
                    "Bad flag address: 0x{:08X} does not match stack pointer 0x{:08X} Offset: {}",
                    flag_addr, stack_addr, signed_diff
                )
            }
            CheckErrorType::BadInitialState(e) => {
                write!(f, "Bad initial CPU state: {}", e)
            }
            CheckErrorType::CycleStateError(e) => {
                write!(f, "Cycle state error: {}", e)
            }
            CheckErrorType::BadMetadata(e) => {
                write!(f, "Bad test metadata: {}", e)
            }
            CheckErrorType::DisassemblyError(e) => {
                write!(f, "Disassembly error: {}", e)
            }
        }
    }
}

impl CheckErrorType {
    pub fn fixed(&self, fixed: bool) -> CheckErrorStatus {
        CheckErrorStatus {
            e_type: self.clone(),
            fixed,
        }
    }
}

#[derive(Clone, Debug)]
pub enum CheckErrorDetail {
    FileError(Vec<CheckErrorStatus>),
    TestError { index: usize, hash: String, errors: Vec<CheckErrorStatus> },
}

impl CheckErrorDetail {
    pub fn errors(&self) -> &[CheckErrorStatus] {
        match self {
            CheckErrorDetail::FileError(errors) => errors,
            CheckErrorDetail::TestError { errors, .. } => errors,
        }
    }
}
