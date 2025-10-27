#include "MooReader.h"
#include <iostream>
#include <iomanip>
#include <string>

// Helper function to print T-state
const char* GetTStateName(uint8_t t_state)
{
    const char* names[] = {"Ti", "T1", "T2", "T3", "T4", "Tw"};
    if (t_state < 6)
        return names[t_state];
    return "UNKNOWN";
}

// Helper function to print queue operation
const char* GetQueueOpName(uint8_t queue_op)
{
    const char* names[] = {"-", "F", "E", "S"};
    if (queue_op < 4)
        return names[queue_op];
    return "?";
}

void PrintRegisters(const Moo::Reader& reader, const Moo::Reader::RegisterState& regs)
{
    for (int i = 0; i < 32; i++)
    {
        if (regs.bitmask & (1 << i))
        {
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
    for (uint8_t byte : test.bytes)
    {
        std::cout << std::hex << std::setw(2) << std::setfill('0') << (int)byte << " ";
    }
    std::cout << std::dec << "\n";
    
    // Print initial state
    std::cout << "\n  Initial State:\n";
    std::cout << "    Registers:\n";
    PrintRegisters(reader, test.init_state.regs);
    
    std::cout << "    RAM entries: " << test.init_state.ram.size() << "\n";
    for (size_t i = 0; i < test.init_state.ram.size(); i++)
    {
        const auto& entry = test.init_state.ram[i];
        std::cout << "      [0x" << std::hex << std::setw(5) << std::setfill('0') 
                  << entry.address << "] = 0x" << std::setw(2) 
                  << (int)entry.value << std::dec << "\n";
    }
    
    if (test.init_state.has_queue)
    {
        std::cout << "    Queue (" << test.init_state.queue.bytes.size() << " bytes): ";
        for (uint8_t byte : test.init_state.queue.bytes)
        {
            std::cout << std::hex << std::setw(2) << std::setfill('0') << (int)byte << " ";
        }
        std::cout << std::dec << "\n";
    }
    
    // Print final state
    std::cout << "\n  Final State:\n";
    std::cout << "    Registers:\n";
    PrintRegisters(reader, test.final_state.regs);
    
    std::cout << "    RAM entries: " << test.final_state.ram.size() << "\n";
    for (size_t i = 0; i < test.final_state.ram.size(); i++)
    {
        const auto& entry = test.final_state.ram[i];
        std::cout << "      [0x" << std::hex << std::setw(5) << std::setfill('0') 
                  << entry.address << "] = 0x" << std::setw(2) 
                  << (int)entry.value << std::dec << "\n";
    }
    
    if (test.final_state.has_queue)
    {
        std::cout << "    Queue (" << test.final_state.queue.bytes.size() << " bytes): ";
        for (uint8_t byte : test.final_state.queue.bytes)
        {
            std::cout << std::hex << std::setw(2) << std::setfill('0') << (int)byte << " ";
        }
        std::cout << std::dec << "\n";
    }
    
    // Print cycles (first few only)
    std::cout << "\n  Cycles: " << test.cycles.size() << "\n";
    for (size_t i = 0; i < test.cycles.size(); i++)
    {
        const auto& cycle = test.cycles[i];
        std::cout << "    [" << i << "] Addr=0x" << std::hex << std::setw(5) 
                  << std::setfill('0') << cycle.address_latch
                  << " Data=0x" << std::setw(4) << cycle.data_bus
                  << std::dec << " Bus=" << reader.GetBusStatusName(cycle.bus_status)
                  << " T=" << GetTStateName(cycle.t_state)
                  << " Q=" << GetQueueOpName(cycle.queue_op_status) << "\n";
    }
    
    // Print exception if present
    if (test.has_exception)
    {
        std::cout << "\n  Exception:\n";
        std::cout << "    Number: " << (int)test.exception.number << "\n";
        std::cout << "    Flag Address: 0x" << std::hex << test.exception.flag_addr 
                  << std::dec << "\n";
    }
    
    // Print hash if present
    if (test.has_hash)
    {
        std::cout << "\n  Hash: ";
        for (int i = 0; i < 20; i++)
        {
            std::cout << std::hex << std::setw(2) << std::setfill('0') 
                      << (int)test.hash[i];
        }
        std::cout << std::dec << "\n";
    }
}

int main(int argc, char* argv[])
{
    if (argc < 2)
    {
        std::cerr << "Usage: " << argv[0] << " <moo_file> [max_tests_to_display]\n";
        std::cerr << "Example: " << argv[0] << " test.moo 3\n";
        return 1;
    }
    
    std::string filename = argv[1];
    int max_tests = -1; // Default to showing 10 tests
    
    if (argc >= 3)
    {
        max_tests = std::stoi(argv[2]);
    }
    
    try
    {
        Moo::Reader reader;
        
        std::cout << "Loading MOO file: " << filename << "\n";
        reader.AddFromFile(filename);
        std::cout << "File loaded, size: " << reader.data.size() << " bytes\n";
		reader.AddRevocationList("revocation_list.txt");
		std::cout << "Revocation list loaded, found " << reader.revocation_list.size() << " revoked tests.\n";
        
        std::cout << "Analyzing...\n";
        
        // Print file header info
        std::cout << "\n========================================\n";
        std::cout << "MOO File Information\n";
        std::cout << "========================================\n";
        std::cout << "Version: " << (int)reader.mooheader.version_major << "." << (int)reader.mooheader.version_minor << "\n";
        std::cout << "CPU: " << reader.mooheader.cpu_name << "\n";
        std::cout << "Test Count: " << reader.mooheader.test_count << "\n";
        
        // Print tests
        int tests_to_show = std::min((int)reader.tests.size(), max_tests);
        std::cout << "\nShowing " << tests_to_show << " of " << reader.tests.size() << " tests:\n";
        
        for (int i = 0; i < tests_to_show; i++)
        {
            PrintTest(reader.tests[i], reader);
        }
        
        if (reader.tests.size() > (size_t)max_tests)
        {
            std::cout << "\n... (" << (reader.tests.size() - max_tests) 
                      << " more tests not shown)\n";
        }
        
        std::cout << "\n========================================\n";
        std::cout << "Analysis complete!\n";
        std::cout << "========================================\n";
        
        return 0;
    }
    catch (const std::exception& e)
    {
        std::cerr << "Error: " << e.what() << "\n";
        return 1;
    }
}
