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

use binrw::{binrw, BinResult, BinWrite};
use std::io::{Cursor, Seek, Write};

#[derive(Copy, Clone, Debug)]
#[binrw]
#[brw(little)]
pub enum MooChunkType {
    #[brw(magic = b"MOO ")]
    FileHeader,
    #[brw(magic = b"TEST")]
    TestHeader,
    #[brw(magic = b"NAME")]
    Name,
    #[brw(magic = b"BYTS")]
    Bytes,
    #[brw(magic = b"INIT")]
    InitialState,
    #[brw(magic = b"FINA")]
    FinalState,
    #[brw(magic = b"REGS")]
    Registers16,
    #[brw(magic = b"REGX")]
    XRegisters,
    #[brw(magic = b"RG32")]
    Registers32,
    #[brw(magic = b"DC32")]
    Descriptors32,
    #[brw(magic = b"RAM ")]
    Ram,
    #[brw(magic = b"QUEU")]
    QueueState,
    #[brw(magic = b"CYCL")]
    CycleStates,
    #[brw(magic = b"HASH")]
    Hash,
    #[brw(magic = b"META")]
    FileMetadata,
    #[brw(magic = b"GMET")]
    GeneratorMetadata,
    #[brw(magic = b"EXCP")]
    Exception,
}

impl MooChunkType {
    pub fn write<WS, T>(&self, writer: &mut WS, payload: &T) -> BinResult<()>
    where
        WS: Write + Seek,
        T: BinWrite + binrw::meta::WriteEndian,
        for<'a> <T as BinWrite>::Args<'a>: Default,
    {
        let mut payload_buf = Cursor::new(Vec::new());

        payload.write_le(&mut payload_buf)?;

        let chunk = MooChunkHeader {
            chunk_type: *self,
            size: payload_buf.position() as u32,
        };

        // Write the chunk header
        chunk.write_le(writer)?;
        // Write the data
        writer
            .write_all(&payload_buf.into_inner())
            .map_err(|e| binrw::Error::Io(e))
    }
}

#[derive(Debug)]
#[binrw]
#[brw(little)]
pub struct MooChunkHeader {
    pub chunk_type: MooChunkType,
    pub size: u32,
}

#[derive(Debug)]
#[binrw]
#[brw(little)]
pub struct MooFileHeader {
    pub version: u8,
    pub reserved: [u8; 3],
    pub test_count: u32,
    pub cpu_name: [u8; 4],
}

#[derive(Debug)]
#[binrw]
#[brw(little)]
pub struct MooTestChunk {
    pub index: u32,
}

#[binrw]
#[brw(little)]
pub struct MooNameChunk {
    pub len: u32,
    #[br(count = len)]
    #[br(map = |x: Vec<u8>| String::from_utf8_lossy(&x).to_string())]
    #[bw(map = |x: &String| x.as_bytes().to_vec())]
    pub name: String,
}

#[derive(Debug)]
#[binrw]
#[brw(little)]
pub struct MooBytesChunk {
    pub len: u32,
    #[br(count = len)]
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
#[binrw]
#[brw(little)]
pub struct MooHashChunk {
    pub hash: [u8; 20],
}
