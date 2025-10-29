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

pub mod stats;

use std::{
    collections::HashMap,
    io::{self, Cursor, Read, Seek, SeekFrom, Write},
};

use crate::{
    test::moo_test::MooTest,
    types::{
        chunks::{
            MooBytesChunk,
            MooChunkHeader,
            MooChunkType,
            MooFileHeader,
            MooHashChunk,
            MooNameChunk,
            MooTestChunk,
        },
        effective_address::MooEffectiveAddress,
        errors::MooError,
        MooCpuType,
        MooCycleState,
        MooException,
        MooFileMetadata,
        MooRamEntries,
        MooStateType,
        MooTestGenMetadata,
    },
    MOO_MAJOR_VERSION,
    MOO_MINOR_VERSION,
};

use binrw::{BinRead, BinResult};

use crate::{
    registers::{MooRegisters, MooRegisters16, MooRegisters32},
    test::test_state::MooTestState,
};
#[cfg(feature = "gzip")]
use flate2::read::GzDecoder;

/// A representation of a **MOO** test file.
///
/// A **MOO** test file is a binary file format used to store CPU tests for emulator validation
/// or research. It contains a series of tests, each with its own initial and final CPU states,
/// memory contents, and execution cycles. The file format is designed to be extensible, based on
/// sized chunks, similar to **RIFF**.
///
/// The [MooTestFile] struct abstracts the file format and provides methods to read from and write
/// to **MOO** test files. It supports optional gzip compression for storage efficiency, if the
/// `gzip` feature is enabled.
///
///
/// # Example
///
/// ```rust
///    use moo::prelude::*;
///    use std::io::Cursor;
///    // Read an entire MOO test file into memory
///    let bytes = std::fs::read("tests/test_data/00.MOO").expect("Failed to read MOO file");
///    // Wrap the slice in a Cursor to provide Seek
///    let mut cursor = Cursor::new(&bytes[..]);
///    // Read the MOO test file
///    let moo_file = MooTestFile::read(&mut cursor).expect("Failed to parse MOO file");
///    // Access test cases
///    for test in moo_file.tests() {
///        println!("Test Name: {}", test.name());
///    }
/// ```
pub struct MooTestFile {
    /// The major version of the **MOO** file format.
    major_version: u8,
    /// The minor version of the **MOO** file format.
    minor_version: u8,
    /// The encoded architecture tag as a String.
    arch: String,
    /// The decoded architecture tag as a [MooCpuType] enum.
    cpu_type: MooCpuType,
    /// A vector of all tests contained in the file as [MooTest] structs.
    tests: Vec<MooTest>,
    /// A map of test SHA1 hashes to their index in the tests vector, for quick lookup.
    hashes: HashMap<String, usize>,
    /// Optional metadata about the file, such as generator info.
    metadata: Option<MooFileMetadata>,
    /// Whether the file was read as gzip-compressed.
    compressed: bool,
}

/// Main implementation block
impl MooTestFile {
    /// Create a new empty `MooTestFile`.
    ///
    /// It is unlikely any users of this crate will need to call this directly. It is normally
    /// utilized by an external test generator to produce a set of tests from hardware.
    ///
    /// Arguments:
    /// * `major_version` - The major version of the MOO file format. Should not exceed [MOO_MAJOR_VERSION].
    /// * `minor_version` - The minor version of the MOO file format. Should not exceed [MOO_MINOR_VERSION].
    /// * `cpu_type` - The CPU architecture type as a [MooCpuType].
    /// * `capacity` - The initial capacity for the tests vector.
    pub fn new(major_version: u8, minor_version: u8, cpu_type: MooCpuType, capacity: usize) -> Self {
        if major_version > MOO_MAJOR_VERSION {
            panic!("major version should be <= {}", MOO_MAJOR_VERSION);
        }
        if minor_version > MOO_MINOR_VERSION {
            panic!("minor version should be <= {}", MOO_MINOR_VERSION);
        }

        Self {
            major_version,
            minor_version,
            arch: cpu_type.to_str().to_string(),
            cpu_type,
            tests: Vec::with_capacity(capacity),
            hashes: HashMap::with_capacity(capacity),
            metadata: None,
            compressed: false,
        }
    }

    /// Set the optional file Metadata struct
    pub fn set_metadata(&mut self, metadata: MooFileMetadata) {
        self.cpu_type = metadata.cpu_type;
        self.metadata = Some(metadata);
    }

    pub fn compressed(&self) -> bool {
        self.compressed
    }

    pub fn set_compressed(&mut self, compressed: bool) {
        self.compressed = compressed;
    }

    pub fn metadata(&self) -> Option<&MooFileMetadata> {
        self.metadata.as_ref()
    }

    pub fn add_test(&mut self, test: MooTest) {
        self.tests.push(test);
    }

    pub fn version(&self) -> u8 {
        self.major_version
    }

    pub fn cpu_type(&self) -> &MooCpuType {
        &self.cpu_type
    }

    pub fn arch(&self) -> &str {
        &self.arch
    }

    pub fn tests(&self) -> &[MooTest] {
        &self.tests
    }

    pub fn tests_mut(&mut self) -> &mut [MooTest] {
        &mut self.tests
    }

    pub fn test_ct(&self) -> usize {
        self.tests.len()
    }

    pub fn read<RS: Read + Seek>(reader: &mut RS) -> BinResult<MooTestFile> {
        // Seek to the start of the reader.
        reader.seek(SeekFrom::Start(0))?;

        let is_gz = MooTestFile::is_gzip_stream(reader)?; // This seeks back to 0.

        // If it's gz, decompress to a Vec and parse from a Cursor so we still have Read+Seek.
        #[cfg(feature = "gzip")]
        if is_gz {
            let mut compressed = Vec::new();
            reader.read_to_end(&mut compressed)?;
            let mut gz = GzDecoder::new(&compressed[..]);

            let mut decompressed = Vec::new();
            gz.read_to_end(&mut decompressed)?;

            let mut cursor = Cursor::new(decompressed);
            let mut test_file = MooTestFile::read_impl(&mut cursor)?;

            test_file.compressed = true;
            return Ok(test_file);
        }

        // If gzip is disabled but stream looks like gzip, return a helpful error.
        #[cfg(not(feature = "gzip"))]
        if is_gz {
            return Err(binrw::Error::Custom {
                pos: 0,
                err: Box::new(MooError::ParseError(
                    "Input appears to be gzip-compressed; rebuild with the `gzip` feature enabled.".to_string(),
                )),
            });
        }

        // Plain (non-gz) path: parse directly.
        MooTestFile::read_impl(reader)
    }

    /// Peek the first two bytes to detect gzip magic (0x1F, 0x8B). Seeks back to start.
    fn is_gzip_stream<R: Read + Seek>(reader: &mut R) -> io::Result<bool> {
        let mut magic = [0u8; 2];
        let start = reader.stream_position().unwrap_or(0);
        reader.read_exact(&mut magic).or_else(|e| {
            // If we can't even read 2 bytes, treat as not-gzip (rewind anyway).
            if e.kind() == io::ErrorKind::UnexpectedEof {
                Ok(())
            }
            else {
                Err(e)
            }
        })?;
        reader.seek(SeekFrom::Start(start))?;
        Ok(magic == [0x1F, 0x8B])
    }

    fn read_impl<R: Read + Seek>(reader: &mut R) -> BinResult<MooTestFile> {
        // Seek to the start of the reader.
        reader.seek(SeekFrom::Start(0))?;

        // Get reader len.
        let reader_len = MooTestFile::get_reader_len(reader)?;

        // Read the file header chunk.
        let header_chunk = MooChunkHeader::read(reader)?;
        if !matches!(header_chunk.chunk_type, MooChunkType::FileHeader) {
            return Err(binrw::Error::Custom {
                pos: reader.stream_position().unwrap_or(0),
                err: Box::new(MooError::ParseError(
                    "Expected FileHeader chunk at the start of the file.".to_string(),
                )),
            });
        }
        // Read the file header.
        let header: MooFileHeader = MooFileHeader::read(reader)?;

        let cpu_string = String::from_utf8_lossy(&header.cpu_id).to_string();
        let cpu_type = MooCpuType::from_str(&cpu_string).map_err(|e| binrw::Error::Custom {
            pos: reader.stream_position().unwrap_or(0),
            err: Box::new(MooError::ParseError(format!(
                "Invalid CPU type '{}': {}",
                cpu_string, e
            ))),
        })?;

        let mut new_file = MooTestFile::new(
            header.major_version,
            header.minor_version,
            cpu_type,
            header.test_count as usize,
        );

        log::debug!(
            "Reading MooTestFile: version {}.{}, arch: {} test_ct: {}",
            header.major_version,
            header.minor_version,
            new_file.arch,
            header.test_count
        );

        let mut in_test = false;
        let mut test_num = 0;
        let mut have_initial_state = false;
        let mut have_final_state = false;
        let cpu_type = MooCpuType::from_str(&new_file.arch).map_err(|e| binrw::Error::Custom {
            pos: reader.stream_position().unwrap_or(0),
            err: Box::new(MooError::ParseError(format!(
                "Invalid CPU type '{}': {}",
                new_file.arch, e
            ))),
        })?;

        // Read chunks until exhausted.
        loop {
            if test_num == header.test_count as usize {
                // We have read all tests, exit the loop.
                log::trace!("Reached expected test count: {}", test_num);
                log::trace!("{} bytes remaining in reader.", reader_len - reader.stream_position()?);
                break;
            }

            let top_level_chunk_offset = reader.stream_position()?;
            let chunk = MooChunkHeader::read(reader)?;

            // log::trace!(
            //     "Read chunk: {:?} pos: {:06X} size: {}",
            //     chunk.chunk_type,
            //     top_level_chunk_offset,
            //     chunk.size
            // );
            match chunk.chunk_type {
                MooChunkType::FileHeader => {
                    log::warn!("Unexpected FileHeader chunk!.");
                }
                MooChunkType::FileMetadata => {
                    // Read the file metadata chunk.
                    let metadata: MooFileMetadata = BinRead::read(reader)?;
                    log::debug!("Reading FileMetadata chunk: {:?}", metadata.mnemonic());
                    new_file.set_metadata(metadata);
                }
                MooChunkType::TestHeader => {
                    // Do a sanity check - did the previous test have both required states?
                    if in_test && (!have_initial_state || !have_final_state) {
                        return Err(binrw::Error::Custom {
                            pos: reader.stream_position().unwrap_or(0),
                            err: Box::new(MooError::ParseError(format!(
                                "Test {} did not have both initial and final states.",
                                test_num
                            ))),
                        });
                    }

                    // Reset the flags for the next test.
                    in_test = true;
                    have_initial_state = false;
                    have_final_state = false;

                    let mut test_name = String::new();
                    let mut test_bytes = Vec::new();

                    // Read the test chunk body.
                    //log::debug!("Reading test body for test {}", test_num);
                    let test_chunk = MooTestChunk::read(reader)?;
                    if test_chunk.index != (test_num as u32) {
                        log::warn!("Test index mismatch: expected {}, got {}", test_num, test_chunk.index);
                    }

                    test_num += 1;

                    // Read the test chunk length into a Cursor.
                    let mut test_buffer = vec![0; chunk.size as usize - size_of::<MooTestChunk>()];
                    // Read the test chunk body into the buffer.
                    reader.read_exact(&mut test_buffer)?;
                    let mut test_reader = Cursor::new(test_buffer);

                    let mut initial_state = MooTestState::default();
                    let mut final_state = MooTestState::default();

                    let mut hash: Option<[u8; 20]> = None;
                    let mut cycle_vec = Vec::new();

                    let mut exception = None;
                    let mut gen_metadata: Option<MooTestGenMetadata> = None;

                    loop {
                        // Read the next chunk type.
                        let bytes_remaining = test_reader.get_ref().len() - test_reader.position() as usize;
                        if bytes_remaining == 0 {
                            if hash.is_none() {
                                return Err(binrw::Error::Custom {
                                    pos: top_level_chunk_offset + test_reader.position(),
                                    err: Box::new(MooError::ParseError(
                                        "Test is missing required HASH chunk.".to_string(),
                                    )),
                                });
                            }

                            let hash_str = hash
                                .as_ref()
                                .unwrap()
                                .iter()
                                .map(|b| format!("{:02X}", b))
                                .collect::<String>();
                            if new_file.hashes.contains_key(&hash_str) {
                                log::warn!("Duplicate test hash detected: {} in test '{}'", hash_str, test_name);
                            }
                            else {
                                new_file.hashes.insert(hash_str, new_file.tests.len());
                            }

                            // Push the test to the file.
                            new_file.add_test(MooTest {
                                name: test_name.clone(),
                                gen_metadata: gen_metadata.clone(),
                                bytes: test_bytes.clone(),
                                initial_state: initial_state.clone(),
                                final_state: final_state.clone(),
                                cycles: cycle_vec.clone(),
                                exception: exception.clone(),
                                hash: hash.clone(),
                            });
                            break;
                        }
                        if bytes_remaining > 0 && bytes_remaining < 8 {
                            return Err(binrw::Error::Custom {
                                pos: top_level_chunk_offset + test_reader.position(),
                                err: Box::new(MooError::ParseError(format!(
                                    "Remaining data bytes ({}) too short to contain a valid chunk.",
                                    bytes_remaining
                                ))),
                            });
                        }

                        let next_chunk = MooChunkHeader::read(&mut test_reader)?;

                        match next_chunk.chunk_type {
                            MooChunkType::Name => {
                                // Read the name chunk.
                                let name_chunk: MooNameChunk = BinRead::read(&mut test_reader)?;
                                test_name = name_chunk.name.clone();
                                log::trace!("Reading NAME chunk: name: {} len: {}", name_chunk.name, name_chunk.len);
                            }
                            MooChunkType::Bytes => {
                                // Read the bytes chunk.
                                let bytes_chunk: MooBytesChunk = BinRead::read(&mut test_reader)?;
                                test_bytes = bytes_chunk.bytes;
                            }
                            MooChunkType::InitialState => {
                                initial_state = MooTestFile::read_state(
                                    MooStateType::Initial,
                                    &mut test_reader,
                                    next_chunk.size.into(),
                                    cpu_type,
                                )?;
                                have_initial_state = true;
                            }
                            MooChunkType::FinalState => {
                                final_state = MooTestFile::read_state(
                                    MooStateType::Final,
                                    &mut test_reader,
                                    next_chunk.size.into(),
                                    cpu_type,
                                )?;
                                have_final_state = true;
                            }
                            MooChunkType::CycleStates => {
                                // Read the cycle states chunk.
                                cycle_vec.clear();
                                let cycle_count: u32 = BinRead::read_le(&mut test_reader)?;
                                //log::debug!("Reading {} cycles", cycle_count);
                                for _ in 0..cycle_count {
                                    let cycle_state = MooCycleState::read(&mut test_reader)?;
                                    cycle_vec.push(cycle_state);
                                }
                            }
                            MooChunkType::Hash => {
                                // Read the hash chunk.
                                let hash_chunk = MooHashChunk::read(&mut test_reader)?;
                                // log::debug!(
                                //     "Reading HASH chunk, pos: {:06X} len: {}",
                                //     top_level_chunk_offset + chunk_offset,
                                //     next_chunk.size
                                // );
                                hash = Some(hash_chunk.hash);
                            }
                            MooChunkType::Exception => {
                                // Read the exception chunk.
                                let exception_chunk = MooException::read(&mut test_reader)?;
                                exception = Some(exception_chunk);
                            }
                            MooChunkType::GeneratorMetadata => {
                                let gen_metadata_chunk = MooTestGenMetadata::read(&mut test_reader)?;
                                gen_metadata = Some(gen_metadata_chunk);
                            }
                            _ => {
                                log::warn!(
                                    "Unexpected chunk type in test: {:?}, skipping next {} bytes",
                                    next_chunk.chunk_type,
                                    next_chunk.size
                                );
                                // Skip the chunk by advancing reader.
                                test_reader.seek(std::io::SeekFrom::Current(next_chunk.size as i64))?;
                            }
                        }
                    }
                }
                _ => break, // End of file or unknown chunk type
            }
        }

        Ok(new_file)
    }

    fn get_reader_len<RS: Read + Seek>(reader: &mut RS) -> BinResult<u64> {
        // Get the current position in the stream.
        let saved_pos = reader.stream_position()?;
        // Seek to the end of the stream.
        reader.seek(std::io::SeekFrom::End(0))?;
        // Get the length of the stream.
        let len = reader.stream_position()?;
        // Restore the original position.
        reader.seek(std::io::SeekFrom::Start(saved_pos))?;
        Ok(len)
    }

    fn read_state<RS: Read + Seek>(
        s_type: MooStateType,
        reader: &mut RS,
        data_len: u64,
        cpu_type: MooCpuType,
    ) -> BinResult<MooTestState> {
        let mut have_regs = false;
        let mut have_ram = false;
        let mut have_queue = false;

        let mut new_state = MooTestState {
            s_type,
            regs: MooRegisters::default_opt(cpu_type),
            descriptors: None,
            queue: Vec::new(),
            ea: None,
            ram: Vec::new(),
        };

        // Get stream length.
        let saved_pos = reader.stream_position()?;
        reader.seek(std::io::SeekFrom::End(0))?;
        let stream_end = reader.stream_position()?;
        // Restore stream pos.
        reader.seek(std::io::SeekFrom::Start(saved_pos))?;

        if data_len > (stream_end - saved_pos) {
            return Err(binrw::Error::Custom {
                pos: reader.stream_position().unwrap_or(0),
                err: Box::new(MooError::ParseError(
                    "Test state chunk is larger than the remaining stream data.".to_string(),
                )),
            });
        }

        let stream_end = saved_pos + data_len;

        loop {
            // Read the next chunk type.
            if reader.stream_position()? >= stream_end {
                return if have_regs && have_ram {
                    // RAM and registers are mandatory, queue is optional.
                    Ok(new_state)
                }
                else {
                    Err(binrw::Error::Custom {
                        pos: reader.stream_position().unwrap_or(0),
                        err: Box::new(MooError::ParseError(
                            "Test state chunk is missing required registers or RAM.".to_string(),
                        )),
                    })
                };
            }
            // Read the next chunk type.
            let next_chunk = MooChunkHeader::read(reader)?;

            match next_chunk.chunk_type {
                MooChunkType::Registers16 => {
                    // Read the registers chunk.
                    let regs = MooRegisters16::read(reader)?;
                    new_state.regs = MooRegisters::Sixteen(regs);
                    have_regs = true;
                }
                MooChunkType::Registers32 => {
                    let regs = MooRegisters32::read(reader)?;
                    new_state.regs = MooRegisters::ThirtyTwo(regs);
                    have_regs = true;
                }
                MooChunkType::Ram => {
                    // Read the RAM chunk.
                    let ram_entries = MooRamEntries::read(reader)?;
                    new_state.ram = ram_entries.entries;
                    have_ram = true;
                }
                MooChunkType::QueueState => {
                    // Read the queue chunk.
                    let queue = MooBytesChunk::read(reader)?;
                    new_state.queue = queue.bytes;
                    have_queue = true;
                }
                MooChunkType::EffectiveAddress32 => {
                    let ea = MooEffectiveAddress::read(reader)?;
                    new_state.ea = Some(ea);
                }
                _ => {
                    log::warn!("Unexpected chunk type in test state: {:?}", next_chunk.chunk_type);
                    // Skip the chunk by advancing reader.
                    reader.seek(std::io::SeekFrom::Current(next_chunk.size as i64))?;
                }
            }
        }
    }

    /// Write a [MooTestFile] to an implementor of [Write](std::io::Write) + [Seek](std::io::Seek).
    /// Arguments:
    /// * `writer` - The writer to write the MOO file to.
    /// * `preserve_hash` - If true, preserves the existing test hashes, if present. If false, test
    ///      hashes will be recalculated from the test data. Test hashes will be recalculated if
    ///      missing, regardless of this flag.
    pub fn write<WS: Write + Seek>(&self, writer: &mut WS, preserve_hash: bool) -> BinResult<()> {
        #[cfg(feature = "gzip")]
        let mut file_writer = if self.compressed {
            // Wrap the writer in a GzEncoder
            use flate2::{write::GzEncoder, Compression};
            let encoder = GzEncoder::new(writer, Compression::new(9));
            Box::new(encoder) as Box<dyn Write>
        }
        else {
            Box::new(writer) as Box<dyn Write>
        };

        #[cfg(not(feature = "gzip"))]
        let mut file_writer = writer;

        let mut cursor = Cursor::new(Vec::<u8>::new());

        // Write the file header chunk.
        MooChunkType::FileHeader.write(
            &mut cursor,
            &MooFileHeader {
                major_version: self.major_version,
                minor_version: self.minor_version,
                reserved: [0; 2],
                test_count: self.tests.len() as u32,
                cpu_id: self.arch.clone().into_bytes()[0..4]
                    .try_into()
                    .expect("CPU Name must be <=4 chars"),
            },
        )?;

        // Write the file metadata chunk, if present
        if let Some(metadata) = &self.metadata {
            MooChunkType::FileMetadata.write(&mut cursor, metadata)?;
        }

        // Write the file header + metadata to the file writer.
        file_writer.write_all(&cursor.into_inner())?;

        // Write all the tests.
        for (ti, test) in self.tests.iter().enumerate() {
            let mut cursor = Cursor::new(Vec::<u8>::new());
            test.write(ti, &mut cursor, preserve_hash)?;
            file_writer.write_all(&cursor.into_inner())?;
        }

        Ok(())
    }
}
