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
    rom_size: u32,

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
    // This value is dictated by the Game ROM that's loaded in, not dynamic at all.
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

        // EXCEPTION!! MBC2 variants are special.
        match mbc_type {
            MbcType::MBC2 | MbcType::Mbc2Battery => {
                ram_size = 512;
            }
            _ => {} // do nothing since mbc2 is the odd one out really
        };

        Self {
            rom_size: rom_size,
            rom: data,                    // put it there as is, cause why not
            ram: vec![0 as u8; ram_size], // initialize preallocated, with zeroes
            mbc_type: mbc_type,
            rom_bank: 1, // start with bank 1, the default one
            ram_bank: 0,
            ram_enabled: false,
        }
    }

    fn get_mapped_rom_bank(&self) -> usize {
        match self.mbc_type {
            MbcType::RomOnly => 0,
            MbcType::MBC1 | MbcType::Mbc1Ram | MbcType::Mbc1RamBattery => {
                if self.rom_bank == 0 {
                    1
                } else {
                    self.rom_bank
                }
            }
            _ => {
                // JUST temporary default behaviour to get the emulator
                // to a core boot stage
                // TODO: Actually implement most of the MBC types so this
                // is usable.
                self.rom_bank
            }
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
            // NOW this is where it gets interesting?
            // 0x0103 - 0x0133 stores the actual nintendo logo, which is funny because any
            // unauthorized games would have to violate a trademark to ilegally make a game for the
            // nintendo GameBoyColor

            // Reading from the cartridge RAM (SWAPPABLE)
            0xA000..=0xBFFF => {
                // Handling RAM, if it's available.
                todo!()
            }

            // the addr is in the FIRST bank (BANK ZERO)
            0x0000..=0x3FFF => self.rom[addr as usize],

            // Reading from the swappable ROM (bank 1 and beyond)
            0x4000..=0x7FFF => {
                let bank = self.get_mapped_rom_bank();
                // let offset = (bank * 1024 * 16);
                // 16kb offset times the bank number
                let bank_offset = bank * 0x4000;
                let bank_offset = bank_offset as u8;

                // The trick here is to convert the addr
                // WHICH comes from the CPU, that is simply thinking
                // in 32bit addresses, of which, this branch will only get
                // the LATTER 16 kb address space.
                // Now, that addr needs to be converted into a simple 0 to N
                // based number, which is then used as the offset.
                let normalized_addr = addr - 0x4000;
                let normalized_addr = normalized_addr as u8;

                // The above trick exploits the fact that 0x4000 is precisely
                // the start of the address range this match statement hooks
                // into. WHICH provides a nice 0 to N number to map to ANY
                // bank, assuming I get the bank_offset calculation right

                self.rom[(normalized_addr + bank_offset) as usize]
            }

            _ => todo!(),
        }
    }
    fn write(&self, addr: u16, value: u8) {
        // TODO:
        todo!()
    }
}
