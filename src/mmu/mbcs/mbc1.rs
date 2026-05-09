pub use crate::mmu::MbcTrait;
pub use crate::mmu::mbc::BankingMode;


pub struct Mbc1 {
    rom_bank: usize,
    ram_bank: usize,
    ram_enabled: bool,
    banking_mode: BankingMode, // 0 for ROM banking, 1 for RAM banking.
}

impl Mbc1 {
    pub fn new() -> Self {
        Self {
            rom_bank: 0,
            ram_bank: 1,
            ram_enabled: false,
            banking_mode: BankingMode::ROM
        }        
    }
}

impl MbcTrait for Mbc1 {
    

    fn get_banking_mode(&self) -> BankingMode {
        self.banking_mode
    }

    fn ram_enabled(&self) -> bool {
        self.ram_enabled
    }

    fn get_rom_bank(&self) -> usize {
        match self.banking_mode {
            BankingMode::ROM => {
                // in ROM banking mode
                // the 2 bits in the ram_bank, are now attached
                // as the 5th and 6th bits to the rom_bank
                self.rom_bank | (self.ram_bank << 5)
            }
            BankingMode::RAM => {
                // RAM banking mode, i.e. the game needs more SRAM,
                // so the rom_bank is not getting the extra bits
                self.rom_bank
            }
        }
        // self.rom_bank
    }

    fn get_ram_bank(&self) -> usize {
        match self.banking_mode {
            BankingMode::ROM => {
                // sram is LOCKED to just one 8kb bank!
                0 as usize
            }
            BankingMode::RAM => self.ram_bank, // SRAM is given priority in bank mode 1
            //  there are now upto 4 ram banks!
        }
    }

    fn map_rom_addr(&self, addr: u16) -> usize {
        if addr < 0x4000 {
            // The address in question is about bank 0
            let bank = match self.banking_mode {
                BankingMode::ROM => 0,
                BankingMode::RAM => 0 | self.ram_bank << 5,
            };
            // the bank offset
            (bank * 0x4000) 
            // the addr mask (14 bits, 16kb block)
            + (addr & 0x3FFF) as usize // don't actually NEED the mask, since
            // the match statement guarantees that the addr is between 0x0000 and
            // 0x4000 (excl)
        } else {
            let bank  = self.get_rom_bank();
            (bank * 0x4000) + (addr & 0x3FFF) as usize
        }
    }

    fn write_rom(&mut self, addr: u16, val: u8) {
        match addr {
            // RAM ENABLE
            0x0000..=0x1FFF => match val {
                0x0A => self.ram_enabled = true,
                0x00 | _ => self.ram_enabled = false, // non 0x0A values disable it
            },

            // ROM bank select
            0x2000..=0x3FFF => {
                // extract the last 5 bits for the ROM bank
                let mut bank = (val & 0x1F) as usize;
                if bank == 0 {
                    // a quirk native to JUST the MBC1, intercept
                    // 0 to 1
                    bank = 1
                };
                self.rom_bank = bank;
            }

            // RAM bank select
            0x4000..=0x5FFF => {
                self.ram_bank = (val & 0b11) as usize; // get the last 2 bits
            }

            // Banking mode select
            0x6000..=0x7FFF => {
                self.banking_mode = BankingMode::from_u8(val & 0x1);
            }

            _ => {
                //Not sure what to do here
                println!("Writing to a unhandled location!!");
            }
        }
    }

}


