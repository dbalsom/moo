![moo_reading_01](../img/cow_reading_01.png)

# mooreader.h

This is a single-header `MOO` parsing library in C++11.

## Basic Usage

First, instantiate a `Moo::Reader`:

```cpp
    Moo::Reader reader;
```

Then add a MOO test file to the reader:

```cpp
    reader.AddFromFile("path/to/test.moo");
```

`Moo::Reader` provides iteration over `Moo::Test`s, so you can easily loop over all the tests in the reader:

```cpp
    for (const auto& test : reader) {
        // Access test.name, test.index, test.init_state, test.expected_state, etc.
    }
```    

A `Moo::Test` has the following structure:

```cpp
        uint32_t index;             // 0-based index of the test
        std::string name;           // The human-readable instruction disassembly
        std::vector<uint8_t> bytes; // The raw bytes that make up the instruction being tested
        CpuState init_state;        // The initial CPU state
        CpuState final_state;       // The final CPU state
        std::vector<Cycle> cycles;  // A vector of `Cycle` structures representing per-cycle execution
        bool has_exception;         // A flag indicating whether the test executed an exception
        Exception exception;        // An 'Exception' structure representing the exception
        bool has_hash;              // A flag indicating whether the test has a hash (should always be true)
        std::array<uint8_t, 20> hash;  // The raw 20-byte SHA1 hash of the test
```        

You can set up your emulator's initial register state like so:

```cpp
    for (const auto r : Moo::REG16Range()) {
        auto r = test.GetInitialRegister(r);
        
        // Assuming you have some method to set a register, and a helper function to convert 
        // register enums
        cpu.setRegister(MooRegToRegister(r), test.GetInitialRegister(r));
    }
```

Then you can set up the initial memory state:

```cpp
    for (const auto m : test.init_state.ram) {
        auto [address, value] = m;
        
        // Substitue whatever method allows you to write to memory
        // Mask of 0xFFFFF represents a 1MB address space - modify as needed
        cpu.getBus()->ram()[address & 0xFFFFF] = value;
    }
```    

You are now ready to run the instruction - step your emulator by one instruction.

```cpp
    auto cycles = cpu.stepToNextInstruction();
```

Now you can compare the final register state with your emulator register state:

```cpp
    for (const auto r : Moo::REG16Range()) {
        // Pass 'true' to GetFinalRegister to mask undefined flags
        const uint16_t expected = test.GetFinalRegister(r, true);
        const uint16_t actual = cpu.getRegister(MooRegToRegister(r));

        if (actual != expected) {
            std::ostringstream oss;
            oss << "expected "
                << std::hex << std::uppercase << std::setw(4) << std::setfill('0') << expected
                << ", got "
                << std::hex << std::uppercase << std::setw(4) << std::setfill('0') << actual
                << "\n";

            std::cerr << "Test FAILED: Register '" << r << "' mismatch: " << oss.str();
        }
    }
```

And then validate your final RAM state to your emulator's memory:

```cpp
    for (const auto m : test.final_state.ram) {
        const auto [address, expected] = m;
        const auto actual = cpu.getBus()->ram()[address & 0xFFFFF];

        if (actual != expected) {
            std::ostringstream addr_ss;
            addr_ss << std::hex << std::uppercase << std::setw(5) << std::setfill('0') << address;

            std::ostringstream msg;
            msg << "expected "
                << std::hex << std::uppercase << std::setw(2) << std::setfill('0') << static_cast<int>(expected)
                << ", got "
                << std::hex << std::uppercase << std::setw(2) << std::setfill('0') << static_cast<int>(actual)
                << "\n";

            std::cerr
                << "Test FAILED: Memory address mismatch at: ["
                << addr_ss.str()
                << "] "
                << msg.str();
        }
    }
```

## Examples

See the included example program `mootest.cpp` for a working of accessing test data.
