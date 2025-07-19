use crate::types::MooCpuType;
use binrw::binrw;

#[derive(Debug, Default)]
#[binrw]
#[brw(little)]
pub struct MooFileMetadata {
    pub set_version_major: u8,
    pub set_version_minor: u8,
    pub cpu_type: MooCpuType,
    pub opcode: u32,
    pub mnemonic: [u8; 8],
    pub test_ct: u32,
    pub file_seed: u64,
    pub flag_mask: u32,
}

impl MooFileMetadata {
    pub fn new(
        set_version_major: u8,
        set_version_minor: u8,
        cpu_type: MooCpuType,
        opcode: u32,
    ) -> Self {
        Self {
            set_version_major,
            set_version_minor,
            cpu_type,
            opcode,
            mnemonic: [' ' as u8; 8],
            ..Default::default()
        }
    }

    pub fn with_test_count(mut self, test_count: u32) -> Self {
        self.test_ct = test_count;
        self
    }
    pub fn with_file_seed(mut self, file_seed: u64) -> Self {
        self.file_seed = file_seed;
        self
    }
    pub fn with_flag_mask(mut self, flag_mask: u32) -> Self {
        self.flag_mask = flag_mask;
        self
    }
    pub fn with_mnemonic(mut self, mnemonic: String) -> Self {
        for c in self.mnemonic.iter_mut() {
            *c = ' ' as u8;
        }
        let mnemonic = mnemonic.into_bytes();
        let mnemonic_len = std::cmp::min(mnemonic.len(), 8);
        self.mnemonic[0..mnemonic_len].copy_from_slice(&mnemonic.as_slice()[0..mnemonic_len]);
        self
    }

    pub fn mnemonic(&self) -> String {
        String::from_utf8_lossy(&self.mnemonic).to_string()
    }
}

#[derive(Clone, Debug)]
#[binrw]
#[brw(little)]
pub struct MooTestGenMetadata {
    pub seed: u64,
    pub gen_ct: u16,
}
