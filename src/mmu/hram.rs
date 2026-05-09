use crate::utils::Memory;

pub struct Hram {}

impl Memory for Hram {
    fn read(&self, addr: u16) -> u8 {
        // TODO:
        0x00
    }
    fn write(&mut self, addr: u16, value: u8) {
        // TODO:
    }
}
