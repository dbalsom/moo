![moo_reading_01](../img/cow_reading_01.png)

# MOO File Format Specification, Version 1.1

This document describes the structure of the **MOO** test file format used by CPU tests for the 8088, 8086, V20, V30,
80186, 80286 and 80386 CPUs. **MOO** stands for **Machine Opcode Operation File**.

All fields are little-endian.

### Changes in MOO 1.1

- Additional chunk types have been added:
    - RMSK
    - RG32
    - RM32
    - EA32

### Types

- `ASCII` - An ASCII printable-character byte
- `ASCII_ID` - A four-byte, space-padded (0x20) ASCII printable-character string
- `uint8` - An unsigned, 8-bit byte
- `uint16` - An unsigned, 16-bit, little-endian word
- `uint32` - An unsigned, 32-bit, little-endian double-word
- `uint64` - An unsigned, 64-bit, little-endian quad-word

## MOO File Overview

A **MOO** file consists of a `MOO ` chunk, followed by one or more `TEST` chunks concatenated together.

Each chunk has the following structure:

| Field        | Size (bytes) | Description                                       |
|--------------|--------------|---------------------------------------------------|
| Chunk Type   | 4            | `ASCII_ID` chunk type (e.g. `TEST`, `NAME`, etc.) |
| Chunk Length | 4            | `uint32` size of chunk payload data               |
| Chunk Data   | Variable     | Chunk payload bytes as described below            |

> ### NOTE:
> A conforming parser should use the chunk length field to advance to the next chunk - it SHOULD NOT assume that the
> next chunk immediately follows the previous. This allows additional fields to be added to chunks in future revisions
> of the MOO format without breaking backwards compatibility.
>
> A conforming parser should skip chunks it does not recognize by using the chunk length field.

## MOO File Structure

Chunks can contain other chunks within their payload, creating a hierarchical file structure.

The typical structure of a `MOO` file is:

- `MOO ` chunk
- `META` chunk
- `RMSK` or `RM32` chunk (optional)
- `TEST ` chunk
    - `NAME` chunk
    - `BYTS` chunk
    - `INIT` chunk
        - `REGS` or `RG32` chunk
        - `RAM ` chunk
        - `QUEU` chunk (optional)
    - `FINA` chunk
        - `REGS` or `RG32` chunk
        - `RMSK` or `RM32` chunk (optional)
        - `RAM ` chunk
        - `QUEU` chunk (optional)
    - `CYCL` chunk
    - `EXCP` chunk (optional)
    - `HASH` chunk
- ` TEST` next test chunk

## File-header Chunk: `MOO `

| Field         | Size (bytes) | Description                                          |
|---------------|--------------|------------------------------------------------------|
| Chunk Type    | 4            | `ASCII_ID` of `MOO ` (note the trailing space)       |
| Chunk Length  | 4            | `uint32` size of chunk data                          |
| Major Version | 1            | `uint8` Major `MOO` format version                   |
| Minor Version | 1            | `uint8` Minor `MOO` format version                   |
| Reserved      | 2            | 2x`uint8` reserved                                   |
| Test Count    | 4            | `uint32` Number of tests in file                     |
| CPU ID        | 4            | 4x`ASCII` ID of CPU being tested, padded with spaces |

---

- Current `CPU ID` values:
    - `8088`
    - `8086`
    - `V20 `
    - `V30 `
    - `186 `
    - `C286`
        - Tests captured with the Harris 80C286
    - `386E`
        - Tests captured with the 386EX

The `MOO ` header payload is at least 12 bytes as of file version 1.1, but may grow in future versions.
The current version of `MOO ` is version 1.1. Additional chunk types may be added without incrementing the format
version. Version increments will be reserved for changes to existing chunk types.

## Top-Level Chunks:

### a) `META`

A `META` chunk contains metadata common to all tests in a test file.

| Field         | Size (bytes) | Description                                                                                                     |
|---------------|--------------|-----------------------------------------------------------------------------------------------------------------|
| major_version | 1            | The major version of the MOO test file collection this file belongs to                                          |
| minor_version | 1            | The minor version of the MOO test file collection this file belongs to                                          |
| cpu_type      | 1            | An enumeration describing the exact CPU model used to generate the tests in the file                            |
| opcode        | 4            | `uint32` raw opcode for the tests in this file. Will be 0xFFFFFFFF if multiple opcodes are present in the file. |
| mnemonic      | 8            | 8x`ASCII` bytes containing the mnemonic name of the instruction being tested in this file, padded with spaces.  |
| test_ct       | 4            | `uint32` count of tests contained in the file (should match header count)                                       |
| file_seed     | 8            | `uint64` base seed used to generate the tests in this file                                                      |
| cpu_mode      | 1            | An enumeration describing the mode of the CPU being tested. For real-mode tests, this value is 0.               |
| reserved      | 3            | Reserved bytes.                                                                                                 |

See the [Enumerations and Bitfields](#enumerations-and-bitfields) section below for descriptions of the `cpu_type` and
`cpu_mode` enumerations.

### a) `RMSK` or `RM32`

A `RMSK` or `RM32` chunk may appear at the top level. A description of these chunks is given in the section below.

### b) `TEST`

Each `TEST` chunk represents a single CPU test case, containing multiple **subchunks**, concatenated.

| Field        | Size (bytes) | Description                                                                     |
|--------------|--------------|---------------------------------------------------------------------------------|
| Chunk Type   | 4            | `ASCII_ID` of `TEST`                                                            |
| Chunk Length | 4            | `uint32` length of payload containing `index` field and all following subchunks |
| Index        | 4            | `uint32` 0-based index of test in file                                          |

---

## Subchunks inside a `TEST`

Each subchunk inside the `TEST` chunk is:

| Field        | Size (bytes) | Description                                                                        |
|--------------|--------------|------------------------------------------------------------------------------------|
| Chunk Type   | 4            | `ASCII_ID` one of (`NAME`, `BYTS`, `INIT`, `FINA`, `CYCL`, `EXCP`, `HASH`, `IDX `) |
| Chunk Length | 4            | `uint32` length of payload                                                         |
| Chunk Data   | Variable     | Payload bytes as described in the following sections                               |

---

## Subchunk Types and Payload Formats

### 1. `NAME`

- Length-prefixed ASCII string.
- Format:

| Field       | Size (bytes) | Description                      |
|-------------|--------------|----------------------------------|
| Length      | 4            | `uint32` length of name in bytes |
| Name String | Variable     | ASCII encoded test name          |

The `NAME` chunk has a redundant length field to accomodate expansion.

---

### 2. `BYTS`

- Raw instruction bytes that comprise the current instruction being tested.
- Format:

| Field  | Size (bytes) | Description              |
|--------|--------------|--------------------------|
| Length | 4            | `uint32` number of bytes |
| Bytes  | Variable     | Raw byte values          |

The `BYTS` chunk has a redundant length field to accomodate expansion.

---

### 3. `INIT` and `FINA`

| Field        | Size (bytes) | Description                                                   |
|--------------|--------------|---------------------------------------------------------------|
| Chunk Type   | 4            | `ASCII_ID` of `INIT` or `FINA`                                |
| Chunk Length | 4            | `uint32` length of payload containing all following subchunks |
| Payload      | variable     | `REGS`, `RG32`, `RAM `, `QUEU`, `EA32` subchunks              |

- CPU state snapshots (initial and final).
- Payload consists of further subchunks of the following possible types:

| Subchunk Type | Description                     |
|---------------|---------------------------------|
| `REGS`        | 16-bit register data            |
| `RMSK`        | 16-bit register mask (optional) |
| `RG32`        | 32-bit register data            |
| `RM32`        | 32-bit register mask (optional) |
| `RAM `        | RAM entries                     |
| `QUEU`        | Queue data                      |
| `EA32`        | Effective address info          |

---

#### a) `REGS`

- Represents the regular, 16-bit x86 register file.
- Only registers that were modified by the instruction are stored in the final state, so a bitmask is included that
  indicates whether a register should be parsed or ignored.
- The size of this chunk is dependent on the number of bits set in the mask.
- The `REGS` chunk in the `INIT` chunk will have all register bits set, as the initial state always contains all
  registers.

| Field   | Size (bytes) | Description                                                                                |
|---------|--------------|--------------------------------------------------------------------------------------------|
| Bitmask | 2            | `uint16` bitmask indicating which registers are present (bit 0 = `ax`, bit 1 = `bx`, etc.) |
| Values  | 2 bytes each | `uint16` register values in order for each bit set in the bitmask                          |

From LSB to MSB, the order of registers in the bitfield is:

| 0  | 1  | 2  | 3  | 4  | 5  | 6  | 7  | 8  | 9  | 10 | 11 | 12 | 13    |
|----|----|----|----|----|----|----|----|----|----|----|----|----|-------|
| ax | bx | cx | dx | cs | ss | ds | es | sp | bp | si | di | ip | flags |

#### a) `RMSK`

- The `RMSK` chunk operates in a very similar fashion as the `REGS` chunk, representing each register in the regular
  16-bit register file. However, it contains masks for undefined register state in the event that an instruction leaves
  registers, portions of registers, or specific flags undefined.
- The `RMSK` chunk may appear at the top level of the file, in which case the same masks should be applied to every
  test in the file. If the `RMSK` chunk appears in the final state, it should only be applied to the registers for that
  specific test.
- Applying the masks in order to each of your CPU registers and the registers and the registers in the final test state
  should allow you to successfully ignore a test's undefined behavior.
- The size of this chunk is dependent on the number of bits set in the mask.

| Field   | Size (bytes) | Description                                                                                     |
|---------|--------------|-------------------------------------------------------------------------------------------------|
| Bitmask | 2            | `uint16` bitmask indicating which register masks are present (bit 0 = `ax`, bit 1 = `bx`, etc.) |
| Values  | 2 bytes each | `uint16` register mask values in order for each bit set in the bitmask                          |

From LSB to MSB, the order of registers in the bitfield is:

| 0  | 1  | 2  | 3  | 4  | 5  | 6  | 7  | 8  | 9  | 10 | 11 | 12 | 13    |
|----|----|----|----|----|----|----|----|----|----|----|----|----|-------|
| ax | bx | cx | dx | cs | ss | ds | es | sp | bp | si | di | ip | flags |

    
---

#### b) `RG32`

- Represents the 32-bit x86 register file introduced with the 386.
- Only registers that were modified by the instruction are stored in the final state, so a bitmask is included that
  indicates whether a register should be parsed or ignored.
- The size of this chunk is dependent on the number of bits set in the mask.
- The `REGS` chunk in the `INIT` chunk will have all register bits set, as the initial state always contains all
  registers.

| Field   | Size (bytes) | Description                                                                                  |
|---------|--------------|----------------------------------------------------------------------------------------------|
| Bitmask | 4            | `uint32` bitmask indicating which registers are present (bit 0 = `cr0`, bit 1 = `cr3`, etc.) |
| Values  | 4 bytes each | `uint32` register values in order for each bit set in the bitmask                            |

From LSB to MSB the order of registers in the bitfield is:

| 0   | 1   | 2   | 3   | 4   | 5   | 6   | 7   | 8   | 9   | 10 | 11 | 12 | 13 | 14 | 15 | 16  | 17     | 18  | 19  |
|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|----|----|----|----|----|----|-----|--------|-----|-----|
| cr0 | cr3 | eax | ebx | ecx | edx | esi | edi | ebp | esp | cs | ds | es | fs | gs | ss | eip | eflags | dr6 | dr7 |

For 16-bit segment registers such as `cs`, `ds`, etc., the upper two bytes should be ignored.

#### b) `RM32`

- The `RM32` chunk operates in a very similar fashion as the `RG32` chunk, representing each register in the regular
  32-bit x86 register file introduced with the 386. However, it contains masks for undefined register state in the event
  that an instruction leaves registers, portions of registers, or specific flags undefined.
- The `RM32` chunk may appear at the top level of the file, in which case the same masks should be applied to every
  test in the file. If the `RM32` chunk appears in the final state, it should only be applied to the registers for that
  specific test.
- The size of this chunk is dependent on the number of bits set in the mask.
- Applying the masks in order to each of your CPU registers and the registers and the registers in the final test state
  should allow you to successfully ignore a tests' undefined behavior.

| Field   | Size (bytes) | Description                                                                                       |
|---------|--------------|---------------------------------------------------------------------------------------------------|
| Bitmask | 4            | `uint32` bitmask indicating which register masks are present (bit 0 = `cr0`, bit 1 = `cr3`, etc.) |
| Values  | 4 bytes each | `uint32` register mask values in order for each bit set in the bitmask                            |

From LSB to MSB the order of register masks in the bitfield is:

| 0   | 1   | 2   | 3   | 4   | 5   | 6   | 7   | 8   | 9   | 10 | 11 | 12 | 13 | 14 | 15 | 16  | 17     | 18  | 19  |
|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|----|----|----|----|----|----|-----|--------|-----|-----|
| cr0 | cr3 | eax | ebx | ecx | edx | esi | edi | ebp | esp | cs | ds | es | fs | gs | ss | eip | eflags | dr6 | dr7 |

For 16-bit segment registers such as `cs`, `ds`, etc., the upper two bytes should be ignored.

---

#### c) `RAM `

- List of memory address-value entries. These values should be written at their indicated registers before the start of
  the test.
- Format:

| Field       | Size (bytes) | Description                                               |
|-------------|--------------|-----------------------------------------------------------|
| Entry Count | 4            | `uint32` number of RAM entries                            |
| Entries     | 5 bytes each | Each entry contains `uint32` address + `uint8` byte value |

---

#### d) `QUEU`

- Contents of the processor instruction queue. The queue should be initialized before the test to the specified
  contents, if cycle-accurate testing is being performed.
- Format:

| Field  | Size (bytes) | Description                       |
|--------|--------------|-----------------------------------|
| Length | 4            | `uint32` number of bytes in queue |
| Bytes  | Variable     | `uint8` x `length` queue bytes    |

---

#### e) `EA32`

- Information about the calculated effective address when an instruction has a ModR/M (and possibly SIB) byte.
  This information is not strictly needed to run the test, but is provided for convenience.

- This chunk is only present in tests for the 386.

| Field                | Size (bytes) | Description                          |
|----------------------|--------------|--------------------------------------|
| Register             | 1            | `uint8` effective segment register   |
| Segment Selector     | 2            | `uint16` segment selector value      |
| Segment Base Address | 4            | `uint32` segment base address        |
| Segment Limit        | 4            | `uint32` segment limit               |
| Offset               | 4            | `uint32` offset/index                |
| Linear Address       | 4            | `uint32` calculated linear address   |
| Physical Address     | 4            | `uint32` calculated physical_address |

The segment register is encoded as an enumeration:

| Value | Meaning       |
|-------|---------------|
| 0     | `CS` register |
| 1     | `SS` register |
| 2     | `DS` register |
| 3     | `ES` register |
| 4     | `FS` register |
| 5     | `GS` register |
| ...   | Invalid       |

---

The following chunks are again outside the `INIT` and `FINA` chunks, but within a `TEST` chunk.

### 4. `CYCL`

- List of CPU bus cycle states.
- Format:

| Field       | Size (bytes)  | Description                                                                                                                                                                                                                                                                                                                                                                                                  |
|-------------|---------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Cycle Count | 4             | `uint32` Number of cycles                                                                                                                                                                                                                                                                                                                                                                                    |
| Cycles      | 15 bytes each | Each cycle encoded as (in order):<br>• `pin_bitfield0` (1 byte)<br>• `address_latch` (`uint32`)<br>• `segment_status` (1 byte enum)<br>• `memory_status` (1 byte bitfield)<br>• `io_status` (1 byte bitfield)<br>• `pin_bitfield1` (1 byte)<br>• `data_bus` (`uint16`)<br>• `bus_status` (1 byte bitfield)<br>• `t_state` (1 byte enum)<br>• `queue_op_status` (1 byte enum)<br>• `queue_byte_read` (1 byte) |

See the section [Enumerations and Bitfields](#enumerations-and-bitfields) below for an explanation of these values.

---

### 4. `EXCP`

- An optional chunk present if the test executed an exception or interrupt.
- Currently only included with 286 and 386 tests due to an improved test generator.

| Field     | Size (bytes) | Description                            |
|-----------|--------------|----------------------------------------|
| Number    | 1            | `uint8` Exception or Interrupt Number  |
| Flag Addr | 4            | `uint32` Address of Flags on the Stack |

- When an exception or interrupt occurs, the flag register is pushed to the stack. For division exceptions in
  particular,
  the value of the flags register may include undefined flags that are tricky to emulate. You can use the provided
  address to mask the undefined flags from the value to assist in memory state validation.

---

### 5. `HASH`

- SHA-1 hash of the test data. The hashing method is subject to change. The hash is not intended to be used as error
  detection, but is simply intended to uniquely identify a test in an entire test suite. Test suites are checked for
  duplicate hashes before publication. A test may be regenerated or otherwise altered while maintaining the same hash.

- The hexadecimal ASCII representation of a hash may be added to a **revocation list** in a test suite in the event that
  a problematic or incorrect test is discovered.

| Field     | Size (bytes) | Description             |
|-----------|--------------|-------------------------|
| Hash Data | 20           | 20 x `uint8` SHA-1 hash |

---

## Enumerations and Bitfields

### Pin Bitfield #0 (`pin_bitfield0`)

| Bit | Description |
|-----|-------------|
| 0   | `ALE` pin** |
| 1   | `BHE` pin   |
| 2   | `READY` pin |
| 3   | `LOCK` pin  |

- The 8088, 8086, V20 and V30 tests only contain the `ALE` pin in this field.
- ****NOTE:** On 80386, `ALE` is synthesized by the inverse of the `ADS` pin, which was active-low.

### Pin Bitfield #1 (`pin_bitfield1`)

| Bit | Description |
|-----|-------------|
| 0   | BHE pin*    |

- *This pin is valid on 8086 and V30. For the 286 and 386, it was moved to pin_bitfield0.

### Segment Status (`segment_status`)

- Valid only for 8088, 8086, V20, V30

| Value | Meaning      |
|-------|--------------|
| 0     | `ES`         |
| 1     | `SS`         |
| 2     | `CS` or None |
| 3     | `DS`         |
| 4     | Not valid    |

---

### Memory and IO Status (`memory_status` and `io_status`)

| Bit | Description     |
|-----|-----------------|
| 0   | Write           |
| 1   | Advanced Write* |
| 2   | Read            |

- *Valid only for 8088, 8086, V20, V30

### Bus Status (`bus_status`)

- An bitfield representing the bus status pins of the CPU.
- On 8088, 8086, V20 and V30, this is an octal value encoding the CPU's S0-S2 status pins.
- On 80286, this value is an encoding of S0, S1, M/IO and COD/INTA lines.
- On 80386, this value is synthesized from the R, W, M/IO, and C/D lines.

#### 8088, 8086, V20, V30 Bus Status Decode

| Value | Abbreviation | Meaning               |
|-------|--------------|-----------------------|
| 0     | "INTA"       | Interrupt Acknowledge |
| 1     | "IOR"        | IO Read               |
| 2     | "IOW"        | IO Write              |
| 3     | "MEMR"       | Memory Read           |
| 4     | "MEMW"       | Memory Write          |
| 5     | "HALT"       | Halt                  |
| 6     | "CODE"       | Code Fetch            |
| 7     | "PASV"       | Passive               |

#### 80286 Bus Status Decode

| Value | Abbreviation | Meaning               |
|-------|--------------|-----------------------|
| 0     | "INTA"       | Interrupt Acknowledge |
| 1     | "PASV"       | Passive / Reserved    |
| 2     | "PASV"       | Passive / Reserved    |
| 3     | "PASV"       | Passive / Invalid     |
| 4     | "HALT"       | Halt                  |
| 5     | "MEMR"       | Memory Read           |
| 6     | "MEMW"       | Memory Write          |
| 7     | "PASV"       | Passive / Invalid     |
| 8     | "PASV"       | Passive / Reserved    |
| 9     | "IOR "       | IO Read               |
| 10    | "IOW "       | IO Write              |
| 11    | "PASV"       | Passive / Invalid     |
| 12    | "PASV"       | Passive / Reserved    |
| 13    | "CODE"       | Code Fetch            |
| 14    | "PASV"       | Passive / Reserved    |
| 15    | "PASV"       | Passive / Invalid     |

#### 80386 Bus Status Decode

| Value | Abbreviation | Meaning                         |
|-------|--------------|---------------------------------|
| 0     | "INTA"       | Interrupt Acknowledge / Passive |
| 1     | "PASV"       | Passive                         |
| 2     | "IOR"        | IO Read                         |
| 3     | "IOW"        | IO Write                        |
| 4     | "CODE"       | Code Fetch                      |
| 5     | "HALT"       | Halt                            |
| 6     | "MEMR"       | Memory Read                     |
| 7     | "MEMW"       | Memory Write                    |

---

### T-State (`t_state`)

| Value | 808X Meaning | 80286 Meaning | 80386 Meaning |
|-------|--------------|---------------|---------------|
| 0     | Ti           | Ti            | Ti            |
| 1     | T1           | Ts*           | T1            |
| 2     | T2           | Tc*           | T2            |
| 3     | T3           | --            | --            |
| 4     | T4           | --            | --            |
| 5     | Tw**         | --            | --            |

- T-State is not a status read directly from the CPU as it is not exposed in any way.
  T-State is calculated based on bus activity.

- *Intel renamed the T-states in documentation for the 80286, then changed their mind
  and changed them back in documentation for the 80386. You're free to treat Ts and Tc as
  T1 and T2.

- **For 80286 and 80386, explicit Tw states do not occur - wait states are effected by
  repeating Tc/T2.

---

### Queue Operation Status (`queue_op_status`)

- This is a two-bit bitfield representing the state of the CPU's queue status pins QS0 and QS1.
- Only valid for 8088, 8086, V20, V30

| Value | Abbr | Meaning                         |
|-------|------|---------------------------------|
| 0     | -    | No Queue Operation              |
| 1     | F    | First Byte Read From Queue      |
| 2     | E    | Queue Emptied (Flushed)         |
| 3     | S    | Subsequent Byte Read From Queue |

---

