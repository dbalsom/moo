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
use moo::types::MooCpuType;
use serde::Deserialize;
use std::{collections::HashMap, path::Path, str::FromStr};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("Invalid options provided: {0}")]
    InvalidOptions(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Unknown error")]
    Unknown,
}

pub trait SchemaRecord {
    fn init(&mut self);
    fn opcode(&self) -> u16;
    fn extension(&self) -> Option<u8>;
}

pub struct SchemaDb<RecordType> {
    pub cpu_type: MooCpuType,
    pub records: Vec<RecordType>,
    pub record_hash: HashMap<(u16, u8), usize>,
}

impl<RecordType: for<'de> Deserialize<'de> + SchemaRecord> SchemaDb<RecordType> {
    pub fn from_file(cpu_type: MooCpuType, path: impl AsRef<Path>) -> Result<SchemaDb<RecordType>, SchemaError> {
        let mut csv_reader = csv::Reader::from_path(path.as_ref()).map_err(|e| SchemaError::IoError(e.into()))?;

        let mut records: Vec<RecordType> = Vec::new();
        let mut record_hash: HashMap<(u16, u8), usize> = HashMap::new();

        for result in csv_reader.deserialize::<RecordType>() {
            match result {
                Ok(mut record) => {
                    record.init();

                    let index = records.len();
                    records.push(record);
                    record_hash.insert(
                        (records[index].opcode(), records[index].extension().unwrap_or(0)),
                        index,
                    );
                }
                Err(e) => {
                    return Err(SchemaError::IoError(e.into()));
                }
            }
        }

        Ok(SchemaDb {
            cpu_type,
            records,
            record_hash,
        })
    }

    pub fn opcode(&self, opcode: u16, ext: u8) -> Option<&RecordType> {
        self.record_hash.get(&(opcode, ext)).map(|&index| &self.records[index])
    }
}

fn de_hex_u16<'de, D>(de: D) -> Result<u16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(de)?;
    let s = s.trim();
    // Accept "0x1A", "1a", "1A", allow underscores
    let s = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")).unwrap_or(s);
    let s = s.replace('_', "");
    u16::from_str_radix(&s, 16).map_err(serde::de::Error::custom)
}

fn de_hex_u32_opt<'de, D>(de: D) -> Result<Option<u32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(de)?;
    let s = s.trim();
    if s.is_empty() {
        return Ok(None);
    }
    // Accept "0x1A", "1a", "1A", allow underscores
    let s = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")).unwrap_or(s);
    let s = s.replace('_', "");
    u32::from_str_radix(&s, 16)
        .map(|v| Some(v))
        .map_err(serde::de::Error::custom)
}

fn de_ext_u8<'de, D>(de: D) -> Result<Option<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(de)?;
    let s = s.trim();
    if s.is_empty() {
        return Ok(None);
    }
    u8::from_str(&s).map(|v| Some(v)).map_err(serde::de::Error::custom)
}

fn de_bool<'de, D>(de: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(de)?;
    let s = s.trim().to_lowercase();
    // Assume empty is 'false'
    if s.is_empty() {
        return Ok(false);
    }
    match s.as_str() {
        "true" | "1" | "y" | "yes" => Ok(true),
        "false" | "0" | "n" | "no" => Ok(false),
        _ => Err(serde::de::Error::custom(format!("Invalid boolean value: {}", s))),
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct EditSchemaRecord {
    #[serde(rename = "op")]
    #[serde(deserialize_with = "de_hex_u16")]
    pub opcode_raw: u16,
    #[serde(rename = "ct")]
    pub count: Option<u32>,
    #[serde(rename = "g")]
    #[serde(deserialize_with = "de_ext_u8")]
    pub group: Option<u8>,
    #[serde(rename = "ex")]
    #[serde(deserialize_with = "de_ext_u8")]
    pub extension: Option<u8>,
    #[serde(rename = "f_umask")]
    #[serde(deserialize_with = "de_hex_u32_opt")]
    pub f_umask: Option<u32>,
}

impl SchemaRecord for EditSchemaRecord {
    fn init(&mut self) {
        // No additional initialization needed
    }

    fn opcode(&self) -> u16 {
        self.opcode_raw
    }

    fn extension(&self) -> Option<u8> {
        self.extension
    }
}
