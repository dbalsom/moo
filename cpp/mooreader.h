#pragma once

#include <array>
#include <cstdint>
#include <cstring>
#include <fstream>
#include <iostream>
#include <stdexcept>
#include <string>
#include <unordered_map>
#include <vector>

struct MooReader
{
	std::vector<uint8_t> data;
	size_t offset = 0;

	// Helper structures for reading
	struct ChunkHeader
	{
		std::string type; //4 characters
		size_t length{};
		size_t data_start{};
		size_t data_end{};
	};
	
	enum CPUType
	{
		CPU8088,
		CPU8086,
		CPUV20,
		CPUV30,
		CPU286,
		CPU386E,
		
		CPU_COUNT
	};

	struct MooHeader
	{
		uint8_t version{};
		uint8_t reserved[3] = {};
		uint32_t test_count{};
		std::string cpu_name;
		CPUType cpu_type;
	};

	struct RegisterState
	{
		uint32_t bitmask{};
		std::vector<uint32_t> values;
	};

	struct RamEntry
	{
		uint32_t address{};
		uint8_t value{};
	};

	struct QueueData
	{
		std::vector<uint8_t> bytes;
	};

	struct CpuState
	{
		RegisterState regs;
		std::vector<RamEntry> ram;
		QueueData queue;
		bool has_queue = false;
	};

	struct Cycle
	{
		uint8_t pin_bitfield0;
		uint32_t address_latch;
		uint8_t segment_status;
		uint8_t memory_status;
		uint8_t io_status;
		uint8_t pin_bitfield1;
		uint16_t data_bus;
		uint8_t bus_status;
		uint8_t t_state;
		uint8_t queue_op_status;
		uint8_t queue_byte_read;
	};

	struct Exception
	{
		uint8_t number;
		uint32_t flag_addr;
	};

	struct Test
	{
		uint32_t index{};
		std::string name;
		std::vector<uint8_t> bytes;
		CpuState init_state;
		CpuState final_state;
		std::vector<Cycle> cycles;
		bool has_exception = false;
		Exception exception;
		bool has_hash = false;
		std::array<uint8_t,20> hash = {};
	};

	MooHeader header;
	std::vector<Test> tests;
	
	// We just xor the values, since the input is already random (a hash)
	struct ArrayHash
	{
		size_t operator()(const std::array<uint8_t, 20>& arr) const noexcept
		{
			size_t hash = 0;
			uint32_t chunks[5];
			std::memcpy(chunks, arr.data(), 20);

			hash ^= chunks[0];
			hash ^= chunks[1];
			hash ^= chunks[2];
			hash ^= chunks[3];
			hash ^= chunks[4];
			
			return hash;
		}
	};
	std::unordered_map<std::array<uint8_t,20>,size_t,ArrayHash> test_map; // Maps hash to index in tests

	void LoadFromFile(std::string filename)
	{
		std::ifstream file(filename, std::ios::binary | std::ios::ate);
		if (!file.is_open())
		{
			throw std::runtime_error("Failed to open file: " + filename);
		}

		std::streamsize size = file.tellg();
		file.seekg(0, std::ios::beg);

		data.resize(size);
		if (!file.read(reinterpret_cast<char*>(data.data()), size))
		{
			throw std::runtime_error("Failed to read file: " + filename);
		}
		
		Analyze();
	}

	// Reading helper function
	template<typename DATA>
	DATA Read()
	{
		if (offset + sizeof(DATA) > data.size())
			throw std::runtime_error("Read past end of data");
		DATA value = DATA(data[offset]);
		for(int i=1; i<sizeof(DATA); ++i) //this loop will be optimized by the compiler
			value |= (DATA(data[offset+i])<<DATA(i*8));
		offset += sizeof(DATA);
		return value;
	}

	void ReadBytes(void* dest, size_t count)
	{
		if (offset + count > data.size())
			throw std::runtime_error("Read past end of data");
		std::memcpy(dest, &data[offset], count);
		offset += count;
	}

	ChunkHeader ReadChunkHeader()
	{
		ChunkHeader header;
		header.type.resize(4);
		ReadBytes(header.type.data(), 4);
		header.length = Read<uint32_t>();
		header.data_start = offset;
		header.data_end = offset+header.length;
		return header;
	}

	RegisterState ReadRegisters()
	{
		RegisterState regs;
		regs.bitmask = Read<uint16_t>();
		
		// Count set bits and read that many register values
		for (int i = 0; i < 16; i++)
		{
			if (regs.bitmask & (1 << i))
			{
				regs.values.push_back(Read<uint16_t>());
			}
		}
		return regs;
	}

	RegisterState ReadRegisters32()
	{
		RegisterState regs;
		regs.bitmask = Read<uint32_t>();
		
		// Count set bits and read that many register values
		for (int i = 0; i < 32; i++)
		{
			if (regs.bitmask & (1 << i))
			{
				regs.values.push_back(Read<uint32_t>());
			}
		}
		return regs;
	}

	std::vector<RamEntry> ReadRam()
	{
		uint32_t count = Read<uint32_t>();
		std::vector<RamEntry> entries;
		entries.reserve(count);
		
		for (uint32_t i = 0; i < count; i++)
		{
			RamEntry entry;
			entry.address = Read<uint32_t>();
			entry.value = Read<uint8_t>();
			entries.push_back(entry);
		}
		return entries;
	}

	QueueData ReadQueue()
	{
		QueueData queue;
		uint32_t length = Read<uint32_t>();
		queue.bytes.resize(length);
		ReadBytes(queue.bytes.data(), length);
		return queue;
	}

	CpuState ReadCpuState(size_t end_offset)
	{
		CpuState state;
		
		while (offset < end_offset)
		{
			ChunkHeader chunk = ReadChunkHeader();
			
			if (chunk.type == "REGS")
			{
				state.regs = ReadRegisters();
			}
			else if (chunk.type == "RG32")
			{
				state.regs = ReadRegisters32();
			}
			else if (chunk.type == "RAM ")
			{
				state.ram = ReadRam();
			}
			else if (chunk.type == "QUEU")
			{
				state.queue = ReadQueue();
				state.has_queue = true;
			}
			offset = chunk.data_end;
		}
		return state;
	}

	std::vector<Cycle> ReadCycles()
	{
		uint32_t count = Read<uint32_t>();
		std::vector<Cycle> cycles;
		cycles.reserve(count);
		
		for (uint32_t i = 0; i < count; i++)
		{
			Cycle cycle;
			cycle.pin_bitfield0 = Read<uint8_t>();
			cycle.address_latch = Read<uint32_t>();
			cycle.segment_status = Read<uint8_t>();
			cycle.memory_status = Read<uint8_t>();
			cycle.io_status = Read<uint8_t>();
			cycle.pin_bitfield1 = Read<uint8_t>();
			cycle.data_bus = Read<uint16_t>();
			cycle.bus_status = Read<uint8_t>();
			cycle.t_state = Read<uint8_t>();
			cycle.queue_op_status = Read<uint8_t>();
			cycle.queue_byte_read = Read<uint8_t>();
			cycles.push_back(cycle);
		}
		
		return cycles;
	}

	Test ReadTest()
	{
		Test test;
		ChunkHeader test_header = ReadChunkHeader();
		
		while (test_header.type != "TEST")
		{
			std::cout << "Skipping " << test_header.type << " chunk" << std::endl;
			offset = test_header.data_end;
			test_header = ReadChunkHeader();
		}
		
		test.index = Read<uint32_t>();
		while (offset < test_header.data_end)
		{
			ChunkHeader chunk = ReadChunkHeader();
			
			if (chunk.type == "NAME")
			{
				uint32_t name_len = Read<uint32_t>();
				test.name.resize(name_len);
				ReadBytes(&test.name[0], name_len);
			}
			else if (chunk.type == "BYTS")
			{
				uint32_t byte_count = Read<uint32_t>();
				test.bytes.resize(byte_count);
				ReadBytes(test.bytes.data(), byte_count);
			}
			else if (chunk.type == "INIT")
			{
				test.init_state = ReadCpuState(chunk.data_end);
			}
			else if (chunk.type == "FINA")
			{
				test.final_state = ReadCpuState(chunk.data_end);
			}
			else if (chunk.type == "CYCL")
			{
				test.cycles = ReadCycles();
			}
			else if (chunk.type == "EXCP")
			{
				test.exception.number = Read<uint8_t>();
				test.exception.flag_addr = Read<uint32_t>();
				test.has_exception = true;
			}
			else if (chunk.type == "HASH")
			{
				ReadBytes(test.hash.data(), 20);
				test.has_hash = true;
			}
			else if (chunk.type == "GMET")
			{
				// Skip generating metadata
			}
			else
			{
				// Skip unknown chunks
				std::cout << "Skipping unknown chunk " << chunk.type << std::endl;
			}
			
			// Ensure we're at the chunk boundary
			offset = chunk.data_end;
		}
		offset = test_header.data_end;
		
		return test;
	}
	
	void ReadMooHeader()
	{
		header.version = Read<uint8_t>();
		ReadBytes(header.reserved, 3);
		header.test_count = Read<uint32_t>();
		
		header.cpu_name.resize(4);
		ReadBytes(header.cpu_name.data(), 4);
		
		// Add new CPUs here and the CPUType enum
		if (header.cpu_name == "8088")
			header.cpu_type = CPU8088;
		else if (header.cpu_name == "8086")
			header.cpu_type = CPU8086;
		else if (header.cpu_name == "V20 ")
			header.cpu_type = CPUV20;
		else if (header.cpu_name == "V30 ")
			header.cpu_type = CPUV30;
		else if (header.cpu_name == "286 ")
			header.cpu_type = CPU286;
		else if (header.cpu_name == "386E")
			header.cpu_type = CPU386E;
		else
		{
			throw std::runtime_error("Unsupported CPU type: " + header.cpu_name);
		}
	}

	void Analyze()
	{
		offset = 0;
		tests.clear();
		test_map.clear();
		
		// First chunk must be "MOO "
		ChunkHeader first_chunk_header = ReadChunkHeader();
		if (first_chunk_header.type != "MOO ")
			throw std::runtime_error("Invalid MOO file - missing MOO header");
		ReadMooHeader();
		offset = first_chunk_header.data_end;
		
		// Read all tests
		tests.reserve(header.test_count);

		for (uint32_t i = 0; i < header.test_count; i++)
		{
			tests.push_back(ReadTest());
			test_map[tests.back().hash] = i;
		}
	}
	
	// Helper function to print bus status
	const char* GetBusStatusName(uint8_t status) const
	{
		CPUType cpu = header.cpu_type;
		
		switch(cpu)
		{
		case CPU8088:
		case CPU8086:
		case CPUV20:
		case CPUV30:
		{
			const char* names[] = {"INTA", "IOR", "IOW", "MEMR", "MEMW", "HALT", "CODE", "PASV"};
			if (status < 8)
				return names[status];
			break;
		}
			
		case CPU286:
		{
			const char* names[] = {"INTA", "PASV", "PASV", "PASV", "HALT", "MEMR", "MEMW", "PASV",
								   "PASV", "IOR ", "IOW ", "PASV", "PASV", "CODE", "PASV", "PASV"};
			if (status < 16)
				return names[status];
			break;
		}
		
		case CPU386E:
		{
			const char* names[] = {"INTA", "PASV", "IOR", "IOW", "CODE", "HALT", "MEMR", "MEMW"};
			if (status < 8)
				return names[status];
			break;
		}
		};
		return "UNKNOWN";
	}


	// Helper function to print register name
	const char* GetRegisterName(int bit_position) const
	{
		CPUType cpu = header.cpu_type;
		
		switch(cpu)
		{
		case CPU8088:
		case CPU8086:
		case CPUV20:
		case CPUV30:
		case CPU286:
		{
			const char* names[] = {"ax", "bx", "cx", "dx", "cs", "ss", "ds", "es", "sp", "bp", "si", "di", "ip", "flags"};
			if (bit_position >= 0 && bit_position < 14)
				return names[bit_position];
			break;
		}
		
		case CPU386E:
		{
			const char* names[] = {"cr0", "cr3", "eax", "ebx", "ecx", "edx", "esi", "edi", "ebp", "esp", 
								   "cs", "ds", "es", "fs", "gs", "ss", "eip", "eflags", "dr6", "dr7"};
			if (bit_position >= 0 && bit_position < 20)
				return names[bit_position];
			break;
		}
		}
		return "unknown";
	}
};

/*
todo:
revocation list?
move GetTStateName and GetQueueOpName to this class
try in granite
*/