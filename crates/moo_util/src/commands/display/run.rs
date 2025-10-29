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
use super::args::DisplayParams;
use crate::args::GlobalOptions;
use anyhow::Error;

use crate::util::print_banner;
use moo::{prelude::*, registers::MooRegistersPrinter, types::MooCycleStatePrinter};

pub const DISPLAY_INDENT: usize = 2;

pub fn run(_global: &GlobalOptions, params: &DisplayParams) -> Result<(), Error> {
    // Load the specified MOO file

    let moo_in = match std::fs::File::open(&params.in_path) {
        Ok(file) => {
            let mut file_reader = std::io::BufReader::new(file);
            let test_file = MooTestFile::read(&mut file_reader)?;

            println!(
                "Read {} tests from file: {}",
                test_file.test_ct(),
                params.in_path.to_string_lossy()
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
            params.in_path.to_string_lossy()
        ));
    }

    let metadata = moo_in.metadata().unwrap();

    if let Some(test_idx) = params.index {
        let mut indent: usize = DISPLAY_INDENT;

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
            regs: &test.initial_state().regs(),
            diff: None,
            indent: (indent as u32) * 2,
        };

        let final_regs_printer = MooRegistersPrinter {
            cpu_type: metadata.cpu_type,
            regs: &test.final_state().regs(),
            diff: Some(&test.initial_state().regs()),
            indent: (indent as u32) * 2,
        };

        let banner_msg = format!(
            "Displaying test {} [#{}/{}]:",
            test.hash_string(),
            test_idx,
            moo_in.test_ct()
        );

        print_banner(banner_msg.as_str());

        if let Some(gen_metadata) = test.gen_metadata() {
            println!("Metadata:");
            indent += DISPLAY_INDENT;
            println!("{:indent$}Seed: {:?}", "", gen_metadata.seed,);
            println!("{:indent$}Generation count: {}", "", gen_metadata.gen_ct,);
            indent -= DISPLAY_INDENT;
        }

        println!("Name: {}", test.name());
        println!("Bytes: {:02X?}", test.bytes());
        println!("Initial state:");
        println!("{:indent$}Registers:", "");
        println!("{}", initial_regs_printer);
        println!("{:indent$}Memory:", "");
        indent += DISPLAY_INDENT;
        for ram_entry in test.initial_state().ram() {
            println!("{:indent$}{:06X}: {:02X}", "", ram_entry.address, ram_entry.value);
        }
        indent -= DISPLAY_INDENT;
        println!("Final state:");
        println!("{:indent$}Registers:", "");
        println!("{}", final_regs_printer);
        println!("{:indent$}Memory:", "");
        indent += DISPLAY_INDENT;
        for ram_entry in test.final_state().ram() {
            println!("{:indent$}{:06X}: {:02X}", "", ram_entry.address, ram_entry.value);
        }
        indent -= DISPLAY_INDENT;

        let mut printer = MooCycleStatePrinter {
            cpu_type: metadata.cpu_type,
            address_latch: 0,
            state: MooCycleState::default(),
            show_cycle_num: true,
            cycle_num: 0,
        };

        println!();
        println!("{:indent$}Cycles ({}):", "", test.cycles().len());
        indent += DISPLAY_INDENT;
        for (_cycle_idx, cycle) in test.cycles().iter().enumerate() {
            if cycle.ale() {
                printer.address_latch = cycle.address_bus;
            }
            printer.state = *cycle;
            println!("{:indent$}{}", "", printer);
            printer.cycle_num = printer.cycle_num.wrapping_add(1);
        }
    }

    Ok(())
}
