use crate::types::MooSegmentRegister;
use binrw::binrw;

#[derive(Clone, Debug)]
#[binrw]
#[brw(little)]
pub struct MooEffectiveAddress {
    pub base_segment: MooSegmentRegister,
    pub base_selector: u16,
    pub base_address: u32,
    pub base_limit: u32,
    pub offset: u32,
    pub linear_address: u32,
    pub physical_address: u32,
}

impl MooEffectiveAddress {
    pub fn new_real(
        base_segment: MooSegmentRegister,
        base_selector: u16,
        base_address: u32,
        base_limit: u32,
        offset: u32,
    ) -> Self {
        let linear_address = base_address.wrapping_add(offset);
        Self {
            base_segment,
            base_selector,
            base_address,
            base_limit,
            offset,
            linear_address,
            physical_address: linear_address,
        }
    }
}
