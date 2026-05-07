use crate::utils::Memory;

// BUILT WITH THIS AS THE REFERENCE: https://gbdev.io/pandocs/The_Cartridge_Header.html

pub enum MbcType {
    RomOnly,
    MBC1,
    Mbc1Ram,
    Mbc1RamBattery,
    MBC2,
    Mbc2Battery,
    RomRam,
    RomRamBattery,
    MMM01,
    Mmm01Ram,
    Mmm01RamBattery,
    Mbc3TimerBattery,
    Mbc3TimerRamBattery,
    MBC3,
    Mbc3Ram,
    Mbc3RamBattery,
    MBC5,
    Mbc5Ram,
    Mbc5RamBattery,
    Mbc5Rumble,
    Mbc5RumbleRam,
    Mbc5RumbleRamBattery,
    MBC6,
    Mbc7SensorRumbleRamBattery,
    PocketCamera,
    BandaiTama5,
    HuC3,
    HuC1RamBattery,
}

pub struct Cartridge {
    rom: Vec<u8>, // for the data in the GAME ROM
    ram: Vec<u8>, // EXTERNAL ram, to be used for stuff like save files

    // mbc stuff
    mbc_type: MbcType, // this dictates the TYPE of MBC that's on the cartridge. Necessary for
    // allocating memory accordingly

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
    pub fn new(data: Vec<u8>) -> Self {
        let mbc_byte = data[0x0147];
        let mbc_type = match mbc_byte {
            0x00 => MbcType::RomOnly,
            0x01 => MbcType::MBC1,
            0x02 => MbcType::Mbc1Ram,
            0x03 => MbcType::Mbc1RamBattery,
            0x05 => MbcType::MBC2,
            0x06 => MbcType::Mbc2Battery,
            0x08 => MbcType::RomRam,
            0x09 => MbcType::RomRamBattery,
            0x0B => MbcType::MMM01,
            0x0C => MbcType::Mmm01Ram,
            0x0D => MbcType::Mmm01RamBattery,
            0x0F => MbcType::Mbc3TimerBattery,
            0x10 => MbcType::Mbc3TimerRamBattery,
            0x11 => MbcType::MBC3,
            0x12 => MbcType::Mbc3Ram,
            0x13 => MbcType::Mbc3RamBattery,
            0x19 => MbcType::MBC5,
            0x1A => MbcType::Mbc5Ram,
            0x1B => MbcType::Mbc5RamBattery,
            0x1C => MbcType::Mbc5Rumble,
            0x1D => MbcType::Mbc5RumbleRam,
            0x1E => MbcType::Mbc5RumbleRamBattery,
            0x20 => MbcType::MBC6,
            0x22 => MbcType::Mbc7SensorRumbleRamBattery,
            0xFC => MbcType::PocketCamera,
            0xFD => MbcType::BandaiTama5,
            0xFE => MbcType::HuC3,
            0xFF => MbcType::HuC1RamBattery,

            _ => panic!(
                "Unknown cartridge! idk what you tryna load at {:#04X}",
                mbc_byte
            ),
        };

        let ram_size_info = data[0x0149];
        let ram_size = match ram_size_info {
            0x00 => 0,
            0x01 => 0, // docs say unused
            0x02 => 8 * 1024,
            0x03 => 32 * 1024,
            0x04 => 128 * 1024, // yes, its unsatisfyingly not in order
            0x05 => 64 * 1024,

            _addr => {
                println!("Unknown value for ramsizeinfo: {:#04X}", _addr);
                0
            }
        };

        Self {
            rom: data,                    // put it there as is, cause why not
            ram: vec![0 as u8; ram_size], // initialize preallocated, with zeroes
            mbc_type: mbc_type,
            rom_bank: 1, // start with bank 1, the default one
            ram_bank: 0,
            ram_enabled: false,
        }
    }
}

impl Memory for Cartridge {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            // NOW this is where it gets interesting?
            // 0x0103 - 0x0133 stores the actual nintendo logo, which is funny because any
            // unauthorized games would have to violate a trademark to ilegally make a game for the
            // nintendo GameBoyColor
            _ => todo!(),
        }
    }
    fn write(&self, addr: u16, value: u8) {
        // TODO:
        todo!()
    }
}
