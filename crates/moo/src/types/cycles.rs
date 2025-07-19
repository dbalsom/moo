use binrw::binrw;
use std::fmt::Display;

#[derive(Copy, Clone, Debug, Default)]
#[binrw]
#[brw(little)]
pub struct MooCycleState {
    pub pins0: u8,
    pub address_bus: u32,
    pub segment: u8,
    pub memory_status: u8,
    pub io_status: u8,
    pub pins1: u8,
    pub data_bus: u16,
    pub bus_state: u8,
    pub t_state: u8,
    pub queue_op: u8,
    pub queue_byte: u8,
}

impl Display for MooCycleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ale_str = if self.pins0 & MooCycleState::PIN_ALE != 0 {
            "A:"
        } else {
            "  "
        };

        let r_chr = if self.memory_status & 0b100 != 0 {
            "R"
        } else {
            "."
        };

        let w_chr = if self.memory_status & 0b001 != 0 {
            "W"
        } else {
            "."
        };

        write!(
            f,
            "{}{:06X}:{:04X} m:{}.{} i:... ",
            ale_str, self.address_bus, self.data_bus, r_chr, w_chr,
        )
    }
}

impl MooCycleState {
    pub const PIN_ALE: u8 = 0b0000_0001;
    pub const PIN_BHE: u8 = 0b0000_0010;
    pub const PIN_READY: u8 = 0b0000_0100;
    pub const PIN_LOCK: u8 = 0b0000_1000;
}
