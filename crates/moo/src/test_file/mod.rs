use crate::types::{MooCpuType, MooRegisters, MooRegisters16};
use std::io::{Cursor, Read, Seek, Write};

use crate::types::chunks::{
    MooBytesChunk, MooChunkHeader, MooChunkType, MooFileHeader, MooHashChunk, MooNameChunk,
    MooTestChunk,
};
use crate::types::errors::MooError;
use crate::types::state::MooTestState;
use crate::types::{
    MooCycleState, MooException, MooFileMetadata, MooRamEntries, MooStateType,
    MooTest, MooTestGenMetadata,
};

use binrw::{BinRead, BinResult, BinWrite};
use sha1::Digest;

pub struct MooTestFile {
    version: u8,
    arch: String,
    tests: Vec<MooTest>,
    metadata: Option<MooFileMetadata>,
}

impl MooTestFile {
    pub fn new(version: u8, arch: String, capacity: usize) -> Self {
        Self {
            version,
            arch,
            tests: Vec::with_capacity(capacity),
            metadata: None,
        }
    }

    pub fn set_metadata(&mut self, metadata: MooFileMetadata) {
        self.metadata = Some(metadata);
    }

    pub fn metadata(&self) -> Option<&MooFileMetadata> {
        self.metadata.as_ref()
    }

    pub fn add_test(&mut self, test: MooTest) {
        self.tests.push(test);
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn arch(&self) -> &str {
        &self.arch
    }

    pub fn tests(&self) -> &[MooTest] {
        &self.tests
    }

    pub fn test_ct(&self) -> usize {
        self.tests.len()
    }

    pub fn read<RS: Read + Seek>(reader: &mut RS) -> BinResult<MooTestFile> {
        // Seek to the start of the reader.
        reader.seek(std::io::SeekFrom::Start(0))?;

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

        let mut new_file = MooTestFile::new(
            header.version,
            String::from_utf8_lossy(&header.cpu_name).to_string(),
            header.test_count as usize,
        );

        log::debug!(
            "Reading MooTestFile: version {}, arch: {} test_ct: {}",
            new_file.version,
            new_file.arch,
            header.test_count
        );

        let mut in_test = false;
        let mut test_num = 0;
        let mut have_initial_state = false;
        let mut have_final_state = false;
        let cpu_type = MooCpuType::from_str(&new_file.arch).map_err(
            |e| binrw::Error::Custom {
                pos: reader.stream_position().unwrap_or(0),
                err: Box::new(MooError::ParseError(format!(
                    "Invalid CPU type '{}': {}",
                    new_file.arch, e
                ))),
            },
        )?;

        // Read chunks until exhausted.
        loop {
            if test_num == header.test_count as usize {
                // We have read all tests, exit the loop.
                log::trace!("Reached expected test count: {}", test_num);
                log::trace!(
                    "{} bytes remaining in reader.",
                    reader_len - reader.stream_position()?
                );
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
                        log::warn!(
                            "Test index mismatch: expected {}, got {}",
                            test_num,
                            test_chunk.index
                        );
                    }

                    test_num += 1;

                    // Read the test chunk length into a Cursor.
                    let mut test_buffer = vec![0; chunk.size as usize - size_of::<MooTestChunk>()];
                    // Read the test chunk body into the buffer.
                    reader.read_exact(&mut test_buffer)?;
                    let mut test_reader = Cursor::new(test_buffer);

                    let mut initial_state = MooTestState::default();
                    let mut final_state = MooTestState::default();

                    let mut hash = None;
                    let mut cycle_vec = Vec::new();

                    let mut exception = None;
                    let mut gen_metadata: Option<MooTestGenMetadata> = None;

                    loop {
                        // Read the next chunk type.
                        let bytes_remaining =
                            test_reader.get_ref().len() - test_reader.position() as usize;
                        if bytes_remaining == 0 {
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
                                log::trace!(
                                    "Reading NAME chunk: name: {} len: {}",
                                    name_chunk.name,
                                    name_chunk.len
                                );
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
                                    cpu_type
                                )?;
                                have_initial_state = true;
                            }
                            MooChunkType::FinalState => {
                                final_state = MooTestFile::read_state(
                                    MooStateType::Final,
                                    &mut test_reader,
                                    next_chunk.size.into(),
                                    cpu_type
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
                                let gen_metadata_chunk =
                                    MooTestGenMetadata::read(&mut test_reader)?;
                                gen_metadata = Some(gen_metadata_chunk);
                            }
                            _ => {
                                log::warn!(
                                    "Unexpected chunk type in test: {:?}, skipping next {} bytes",
                                    next_chunk.chunk_type,
                                    next_chunk.size
                                );
                                // Skip the chunk by advancing reader.
                                test_reader
                                    .seek(std::io::SeekFrom::Current(next_chunk.size as i64))?;
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
            queue: Vec::new(),
            ram: MooRamEntries {
                entry_count: 0,
                entries: Vec::new(),
            },
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
                } else {
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
                    let regs = crate::types::MooRegisters32::read(reader)?;
                    new_state.regs = MooRegisters::ThirtyTwo(regs);
                    have_regs = true;
                }
                MooChunkType::Ram => {
                    // Read the RAM chunk.
                    let ram_entries = MooRamEntries::read(reader)?;
                    new_state.ram = ram_entries;
                    have_ram = true;
                }
                MooChunkType::QueueState => {
                    // Read the queue chunk.
                    let queue = MooBytesChunk::read(reader)?;
                    new_state.queue = queue.bytes;
                    have_queue = true;
                }
                _ => {
                    log::warn!(
                        "Unexpected chunk type in test state: {:?}",
                        next_chunk.chunk_type
                    );
                    // Skip the chunk by advancing reader.
                    reader.seek(std::io::SeekFrom::Current(next_chunk.size as i64))?;
                }
            }
        }
    }

    pub fn write<WS: Write + Seek>(&self, writer: &mut WS) -> BinResult<()> {
        // Write the file header chunk.
        MooChunkType::FileHeader.write(
            writer,
            &MooFileHeader {
                version: self.version,
                reserved: [0; 3],
                test_count: self.tests.len() as u32,
                cpu_name: self.arch.clone().into_bytes()[0..4]
                    .try_into()
                    .expect("CPU Name must be 4 chars long"),
            },
        )?;

        // Write the file metadata chunk, if present
        if let Some(metadata) = &self.metadata {
            MooChunkType::FileMetadata.write(writer, metadata)?;
        }

        // Write all the test chunks.
        for (ti, test) in self.tests.iter().enumerate() {
            let mut test_buffer = Cursor::new(Vec::new());

            // Write the test chunk body.
            MooTestChunk { index: ti as u32 }.write(&mut test_buffer)?;

            // Write the generator metadata chunk if present.
            if let Some(gen_metadata) = &test.gen_metadata {
                MooChunkType::GeneratorMetadata.write(&mut test_buffer, gen_metadata)?;
            }

            // Write the name chunk.
            let name_chunk = MooNameChunk {
                len: test.name.len() as u32,
                name: test.name.clone(),
            };
            MooChunkType::Name.write(&mut test_buffer, &name_chunk)?;

            // Write the bytes chunk.
            let bytes_chunk = MooBytesChunk {
                len: test.bytes.len() as u32,
                bytes: test.bytes.clone(),
            };
            MooChunkType::Bytes.write(&mut test_buffer, &bytes_chunk)?;

            // Write the initial state chunk.
            test.initial_state.write(&mut test_buffer)?;

            // Write the final state chunk.
            test.final_state.write(&mut test_buffer)?;

            let mut cycle_buffer = Cursor::new(Vec::new());
            // Write the count of cycles to the cycle buffer.
            (test.cycles.len() as u32).write_le(&mut cycle_buffer)?;
            // Write all the cycles to the cycle buffer.
            for cycle in &test.cycles {
                cycle.write(&mut cycle_buffer)?;
            }

            // Write the cycles chunk.
            MooChunkType::CycleStates.write(&mut test_buffer, &cycle_buffer.into_inner())?;

            // If an exception is present, write the exception chunk.
            if let Some(exception) = &test.exception {
                MooChunkType::Exception.write(&mut test_buffer, exception)?;
            }

            // Create the SHA1 hash from the current state of the test buffer.
            let hash = sha1::Sha1::digest(&test_buffer.get_ref()).to_vec();

            // Write the hash chunk.
            MooChunkType::Hash.write(&mut test_buffer, &hash)?;

            // Write the test chunk.
            MooChunkType::TestHeader.write(writer, &test_buffer.into_inner())?;
        }

        Ok(())
    }
}
