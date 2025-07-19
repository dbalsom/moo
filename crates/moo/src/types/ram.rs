use binrw::binrw;

#[derive(Clone, Debug, Default)]
#[binrw]
#[brw(little)]
pub struct MooRamEntries {
    pub entry_count: u32,
    #[br(count = entry_count)]
    pub entries: Vec<MooRamEntry>,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[binrw]
#[brw(little)]
pub struct MooRamEntry {
    pub address: u32,
    pub value:   u8,
}
