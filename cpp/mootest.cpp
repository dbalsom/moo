/*
    A demonstration program that loads a MOO test file and dumps a human-readable
    description of the specified number of tests to stdout.
*/


#include "MooReader.h"
#include <iostream>
#include <iomanip>
#include <string>

void PrintRegisters(const Moo::Reader& reader, const Moo::Reader::RegisterState& regs)
{
    for (int i = 0; i < 32; i++) {
        if (regs.bitmask & (1 << i)) {
            std::cout << "      " << reader.GetRegisterName(i) << " = 0x" 
                      << std::hex << std::setw(4) << std::setfill('0') 
                      << regs.values[i] << std::dec << "\n";
        }
    }
}

void PrintTest(const Moo::Reader::Test& test, const Moo::Reader& reader)
{
    std::cout << "\n======================================\n";
    std::cout << "Test #" << test.index << ": " << test.name << "\n";
    std::cout << "======================================\n";
    
    // Print instruction bytes
    std::cout << "\nInstruction bytes (" << test.bytes.size() << "): ";
    for (const uint8_t byte : test.bytes) {
        std::cout << std::hex << std::setw(2) << std::setfill('0') << static_cast<int>(byte) << " ";
    }
    std::cout << std::dec << "\n";
    
    // Print initial state
    std::cout << "\n  Initial State:\n";
    std::cout << "    Registers:\n";
    PrintRegisters(reader, test.init_state.regs);
    
    std::cout << "    RAM entries: " << test.init_state.ram.size() << "\n";
    for (size_t i = 0; i < test.init_state.ram.size(); i++) {
        const auto& entry = test.init_state.ram[i];
        std::cout << "      [0x" << std::hex << std::setw(5) << std::setfill('0') 
                  << entry.address << "] = 0x" << std::setw(2) 
                  << static_cast<int>(entry.value) << std::dec << "\n";
    }
    
    if (test.init_state.has_queue) {
        std::cout << "    Queue (" << test.init_state.queue.bytes.size() << " bytes): ";
        for (const uint8_t byte : test.init_state.queue.bytes) {
            std::cout << std::hex << std::setw(2) << std::setfill('0') << (int)byte << " ";
        }
        std::cout << std::dec << "\n";
    }
    
    // Print final state
    std::cout << "\n  Final State:\n";
    std::cout << "    Registers:\n";
    PrintRegisters(reader, test.final_state.regs);
    
    std::cout << "    RAM entries: " << test.final_state.ram.size() << "\n";
    for (size_t i = 0; i < test.final_state.ram.size(); i++) {
        const auto& entry = test.final_state.ram[i];
        std::cout << "      [0x" << std::hex << std::setw(5) << std::setfill('0') 
                  << entry.address << "] = 0x" << std::setw(2) 
                  << static_cast<int>(entry.value) << std::dec << "\n";
    }
    
    if (test.final_state.has_queue) {
        std::cout << "    Queue (" << test.final_state.queue.bytes.size() << " bytes): ";
        for (const uint8_t byte : test.final_state.queue.bytes) {
            std::cout << std::hex << std::setw(2) << std::setfill('0') << static_cast<int>(byte) << " ";
        }
        std::cout << std::dec << "\n";
    }
    
    // Print cycles (first few only)
    std::cout << "\n  Cycles: " << test.cycles.size() << "\n";
    for (size_t i = 0; i < test.cycles.size(); i++) {
        const auto& cycle = test.cycles[i];
        std::cout << "    [" << i << "] Addr=0x" << std::hex << std::setw(5) 
                  << std::setfill('0') << cycle.address_latch
                  << " Data=0x" << std::setw(4) << cycle.data_bus
                  << std::dec << " Bus=" << reader.GetBusStatusName(cycle.bus_status)
                  << " T=" << reader.GetTStateName(cycle.t_state)
                  << " Q=" << reader.GetQueueOpName(cycle.queue_op_status) << "\n";
    }
    
    // Print exception if present
    if (test.has_exception) {
        std::cout << "\n  Exception:\n";
        std::cout << "    Number: " << (int)test.exception.number << "\n";
        std::cout << "    Flag Address: 0x" << std::hex << test.exception.flag_addr 
                  << std::dec << "\n";
    }
    
    // Print hash if present
    if (test.has_hash) {
        std::cout << "\n  Hash: ";
        for (int i = 0; i < 20; i++) {
            std::cout << std::hex << std::setw(2) << std::setfill('0') 
                      << static_cast<int>(test.hash[i]);
        }
        std::cout << std::dec << "\n";
    }
}

int main(int argc, char* argv[])
{
    if (argc < 2) {
        std::cerr << "Usage: " << argv[0] << " <moo_file> [max_tests_to_display]\n";
        std::cerr << "Example: " << argv[0] << " test.moo 3\n";
        return 1;
    }

    const std::string filename = argv[1];
    size_t max_tests = 10;

    if (argc >= 3) {
        max_tests = std::stoi(argv[2]);
    }
    
    try {
        Moo::Reader reader;
        
        std::cout << "Loading MOO file: " << filename << "\n";
        reader.AddFromFile(filename);
        try {
            reader.AddRevocationList("revocation_list.txt");
            std::cout << "Revocation list loaded, found " << reader.GetRevokedCount() << " revoked tests.\n";
        }
        catch (const std::exception& e) {
            std::cout << "Warning: Could not load revocation list: " << e.what() << "\n";
        }
        
        std::cout << "Analyzing...\n";
        
        // Print file header info
        std::cout << "\n========================================\n";
        std::cout << "MOO File Information\n";
        std::cout << "========================================\n";
        const auto header = reader.GetHeader();
        std::cout << "Version: " << static_cast<int>(header.version_major) << "." << static_cast<int>(header.version_minor) << "\n";
        std::cout << "CPU: " << header.cpu_name << "\n";
        std::cout << "Test Count: " << header.test_count << "\n";
        
        // Print tests
        const int tests_to_show = std::min(reader.size(), max_tests);
        std::cout << "\nShowing " << tests_to_show << " of " << reader.size() << " tests:\n";

        int count = 0;
        for (auto it = reader.begin(); it != reader.end() && count < 10; ++it, ++count) {
            PrintTest(*it, reader);
        }
        
        if (reader.size() > max_tests) {
            std::cout << "\n... (" << (reader.size() - max_tests) << " more tests not shown)\n";
        }
        
        std::cout << "\n========================================\n";
        std::cout << "Analysis complete!\n";
        std::cout << "========================================\n";
        
        return 0;
    }
    catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << "\n";
        return 1;
    }
}
