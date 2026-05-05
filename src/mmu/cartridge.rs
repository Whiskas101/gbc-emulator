use crate::utils::Memory;

pub enum MbcType {
    NoMbc,
    Mbc1,
    // MAYBE more idk
}

pub struct Cartridge {
    rom: Vec<u8>, // for the data in the GAME ROM
    ram: Vec<u8>, // EXTERNAL ram, to be used for stuff like save files

    // mbc stuff
    mbc_type: MbcType,
    // holds which bank is currently active
    // active justmeans mapped to the 0x4000 - 0x7FFF
    // which is 2nd half of the memory address
    // INFO: referenced here for future me https://gbdev.io/pandocs/Memory_Map.html
    rom_bank: usize, //defaults to 1

    // This one is like the previous one, except ie holds which RAM bank is mapped to 0xA000 - 0xBFFF
    ram_bank: usize, // defaults to 0

    // Cartridge RAM is enabled by the game, before use
    ram_enabled: bool,
}

impl Cartridge {
    pub fn new() -> Self {
        // from what I read, it needs to have the nintendo logo hex dump within the  
        let rom = vec![""]

    }
}

impl Memory for Cartridge {
    fn read(&self, addr: u16) -> u8 {
        // TODO:
        0x00
    }
    fn write(&self, addr: u16, value: u8) {
        // TODO:
    }
}
