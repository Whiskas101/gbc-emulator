use crate::{
    mmu::{MbcTrait, mbcs},
    utils::Memory,
};

// BUILT WITH THIS AS THE REFERENCE: https://gbdev.io/pandocs/The_Cartridge_Header.html

// pub enum MbcType {
//     RomOnly,
//     MBC1,
//     Mbc1Ram,
//     Mbc1RamBattery,
//     MBC2,
//     Mbc2Battery,
//     RomRam,
//     RomRamBattery,
//     MMM01,
//     Mmm01Ram,
//     Mmm01RamBattery,
//     Mbc3TimerBattery,
//     Mbc3TimerRamBattery,
//     MBC3,
//     Mbc3Ram,
//     Mbc3RamBattery,
//     MBC5,
//     Mbc5Ram,
//     Mbc5RamBattery,
//     Mbc5Rumble,
//     Mbc5RumbleRam,
//     Mbc5RumbleRamBattery,
//     MBC6,
//     Mbc7SensorRumbleRamBattery,
//     PocketCamera,
//     BandaiTama5,
//     HuC3,
//     HuC1RamBattery,
// }

pub struct Cartridge {
    rom: Vec<u8>, // for the data in the GAME ROM
    ram: Vec<u8>, // EXTERNAL ram, to be used for stuff like save files
    mbc: Box<dyn MbcTrait>,
}

impl Cartridge {
    pub fn new(data: Vec<u8>) -> Self {
        let mbc_byte = data[0x0147];
        let mbc: Box<dyn MbcTrait> = match mbc_byte {
            // 0x00 => MbcType::RomOnly,
            0x00 => Box::new(mbcs::RomOnly::new()),
            // 0x01 => MbcType::MBC1,
            // 0x02 => MbcType::Mbc1Ram,
            0x01..=0x02 => Box::new(mbcs::Mbc1::new()),

            // TODO: Implement the rest of the variants
            // 0x03 => MbcType::Mbc1RamBattery,
            // 0x05 => MbcType::MBC2,
            // 0x06 => MbcType::Mbc2Battery,
            // 0x08 => MbcType::RomRam,
            // 0x09 => MbcType::RomRamBattery,
            // 0x0B => MbcType::MMM01,
            // 0x0C => MbcType::Mmm01Ram,
            // 0x0D => MbcType::Mmm01RamBattery,
            // 0x0F => MbcType::Mbc3TimerBattery,
            // 0x10 => MbcType::Mbc3TimerRamBattery,
            // 0x11 => MbcType::MBC3,
            // 0x12 => MbcType::Mbc3Ram,
            // 0x13 => MbcType::Mbc3RamBattery,
            // 0x19 => MbcType::MBC5,
            // 0x1A => MbcType::Mbc5Ram,
            // 0x1B => MbcType::Mbc5RamBattery,
            // 0x1C => MbcType::Mbc5Rumble,
            // 0x1D => MbcType::Mbc5RumbleRam,
            // 0x1E => MbcType::Mbc5RumbleRamBattery,
            // 0x20 => MbcType::MBC6,
            // 0x22 => MbcType::Mbc7SensorRumbleRamBattery,
            // 0xFC => MbcType::PocketCamera,
            // 0xFD => MbcType::BandaiTama5,
            // 0xFE => MbcType::HuC3,
            // 0xFF => MbcType::HuC1RamBattery,
            _ => panic!(
                "Unknown cartridge! idk what you tryna load at {:#04X}",
                mbc_byte
            ),
        };

        let _rom_size = data[0x0148];
        let rom_size: u32 = match _rom_size {
            0x00..=0x08 => (32 * 1024) << _rom_size,
            0x52 => 1152 * 1024, // (1.1MiB) which is 72 banks
            0x53 => 1280 * 1024, // (1.2MiB) which is 80 banks
            0x54 => 1536 * 1024, // (1.5MiB) which is 96 banks
            _ => {
                println!("Unexpected value for rom_size! Defaulting to 0");
                0
            }
        };

        let ram_size_info = data[0x0149];
        let mut ram_size = match ram_size_info {
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

        // TODO: Handle this edge case when writing the MBC2 struct
        // EXCEPTION!! MBC2 variants are special.
        // match mbc_type {
        //     MbcType::MBC2 | MbcType::Mbc2Battery => {
        //         ram_size = 512;
        //     }
        //     _ => {} // do nothing since mbc2 is the odd one out really
        // };

        Self {
            rom: data,                    // put it there as is, cause why not
            ram: vec![0 as u8; ram_size], // initialize preallocated, with zeroes
            mbc: mbc,
        }
    }
}

impl Memory for Cartridge {
    fn read(&self, addr: u16) -> u8 {
        // Catridge is meant to fully capture and abstract over the concept
        // of the read op so that the actual CPU doesn't know, or rather
        // doesn't HAVE to know about what mountains are being moved
        // to get the data from the cartridge.
        match addr {
            0x0000..=0x7FFF => {
                let resolved_addr = self.mbc.map_rom_addr(addr);
                // This gives us a basic bounds check, returns 0xFF if it is out of
                // bounds, instead of panicking on malformed roms
                self.rom.get(resolved_addr).cloned().unwrap_or(0xFF)
            }
            0xA000..=0xBFFF => {
                if let Some(resolved_addr) = self.mbc.map_ram_addr(addr) {
                    self.ram.get(resolved_addr).cloned().unwrap_or(0xFF)
                } else {
                    0xFF
                }
            }
            _ => 0xFF,
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        // This is where the swap happens. If the cpu attempts to write to a particular register,
        // it will automatically swap the active bank.
        match addr {
            // No writes allowed here, strictly read only memory, it's the cartridge ROM
            // afterall
            0x0000..=0x7FFF => self.mbc.write_rom(addr, val),
            // THIS is the range that can be written to, since it's the cartridge RAM.
            0xA000..=0xBFFF => {
                let ram_length = self.ram.len();
                if let Some(resolved_addr) = self.mbc.map_ram_addr(addr) {
                    // i saw people recommend doing a wrap around
                    // so, wrapping around by the ram length
                    self.ram[resolved_addr % ram_length] = val;
                }
            }
            _ => panic!("NOT SURE what to here"),
        }
    }
}
