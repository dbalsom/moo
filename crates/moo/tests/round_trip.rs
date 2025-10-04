use std::{
    fs::{self, File},
    io::{BufReader, BufWriter},
    path::Path,
};

use moo::test_file::MooTestFile;
use tempfile::tempdir;

#[test]
pub fn round_trip() {
    let test_data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/test_data");
    let input_file = test_data_dir.join("00.MOO");

    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let output_dir = temp_dir.path();
    let output_file = output_dir.join("00_out.MOO");

    // Open the input file
    let input = File::open(&input_file).expect("Failed to open input file");
    let mut reader = BufReader::new(input);

    // Parse the input file as a MooTestFile
    let test_file = MooTestFile::read(&mut reader).expect("Failed to parse input file");

    // Write the parsed file back to the output file
    {
        let output = File::create(&output_file).expect("Failed to create output file");
        let mut writer = BufWriter::new(output);
        test_file.write(&mut writer).expect("Failed to write output file");
    }

    // Compare the original and output files
    let original_content = fs::read(&input_file).expect("Failed to read original file");
    let output_content = fs::read(&output_file).expect("Failed to read output file");

    if original_content.len() != output_content.len() {
        panic!(
            "Files differ in size: original size = {}, output size = {}",
            original_content.len(),
            output_content.len()
        );
    }

    for (i, (byte1, byte2)) in original_content.iter().zip(&output_content).enumerate() {
        if byte1 != byte2 {
            panic!("Files differ at byte offset {}", i);
        }
    }
}
