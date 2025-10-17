use super::args::DisplayParams;
use crate::args::GlobalOptions;
use anyhow::Error;

use moo::{
    prelude::*,
    types::{MooCycleStatePrinter, MooRegistersPrinter},
};

pub fn run(global: &GlobalOptions, params: &DisplayParams) -> Result<(), Error> {
    // Load the specified MOO file

    let moo_in = match std::fs::File::open(&params.in_file) {
        Ok(file) => {
            let mut file_reader = std::io::BufReader::new(file);
            let test_file = MooTestFile::read(&mut file_reader)?;

            println!(
                "Read {} tests from file: {}",
                test_file.test_ct(),
                params.in_file.to_string_lossy()
            );
            test_file
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Error opening file: {}", e));
        }
    };

    if moo_in.metadata().is_none() {
        return Err(anyhow::anyhow!(
            "MOO file {} is missing metadata chunk",
            params.in_file.to_string_lossy()
        ));
    }

    let metadata = moo_in.metadata().unwrap();

    if let Some(test_idx) = params.index {
        // Display a specific test
        if test_idx >= moo_in.test_ct() {
            return Err(anyhow::anyhow!(
                "Test index {} is out of range (0-{})",
                test_idx,
                moo_in.test_ct() - 1
            ));
        }

        let test = &moo_in.tests()[test_idx];

        let initial_regs_printer = MooRegistersPrinter {
            cpu_type: metadata.cpu_type,
            regs: &test.initial_regs(),
            diff: None,
            indent: 4,
        };

        let final_regs_printer = MooRegistersPrinter {
            cpu_type: metadata.cpu_type,
            regs: &test.final_regs(),
            diff: Some(&test.initial_regs()),
            indent: 4,
        };

        println!(
            "Displaying test {} [#{}/{}]:",
            test.hash_string(),
            test_idx,
            moo_in.test_ct()
        );
        println!("Name: {}", test.name());
        println!("Bytes: {:02X?}", test.bytes());
        println!("Initial state:");
        println!("  Registers:");
        println!("{}", initial_regs_printer);
        println!("  Memory:");
        for ram_entry in &test.initial_mem_state().entries {
            println!("    {:06X}: {:02X}", ram_entry.address, ram_entry.value);
        }
        println!("Final state:");
        println!("  Registers:");
        println!("{}", final_regs_printer);
        println!("  Memory:");
        for ram_entry in &test.final_mem_state().entries {
            println!("    {:06X}: {:02X}", ram_entry.address, ram_entry.value);
        }

        let mut printer = MooCycleStatePrinter {
            cpu_type: metadata.cpu_type,
            address_latch: 0,
            state: MooCycleState::default(),
        };
        println!("  Cycles ({}):", test.cycles().len());
        for (_cycle_idx, cycle) in test.cycles().iter().enumerate() {
            if cycle.ale() {
                printer.address_latch = cycle.address_bus;
            }
            printer.state = *cycle;
            println!("    {}", printer);
        }
    }

    Ok(())
}
