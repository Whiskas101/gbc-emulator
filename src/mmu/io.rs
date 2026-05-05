use crate::utils::Memory;

pub enum MbcType {
    NoMbc,
    Mbc1,
    // MAYBE more idk
}

pub struct Io {}

impl Memory for Io {
    fn read(&self, addr: u16) -> u8 {
        // TODO:
        0x00
    }

    fn write(&self, addr: u16, value: u8) {
        // TODO:
        println!("Writing!: {:?}", value);
    }
}
