use crate::utils::Memory;

pub struct Vram {}
impl Memory for Vram {
    fn read(&self, addr: u16) -> u8 {
        // TODO:
        0x00
    }
    fn write(&self, addr: u16, value: u8) {
        // TODO:
    }
}
