use core::panic;

use crate::{
    mmu::{Cartridge, Hram, Io, Oam, Vram, Wram},
    utils::Memory,
};

// IMPORTANT THINGS TO KNOW
// Bus has 2bytes for address
// Bus has 1byte for data

pub struct Bus {
    wram: Wram,
    hram: Hram,
    vram: Vram,
    io: Io,
    oam: Oam,

    cartridge: Cartridge,
}

impl Bus {
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            // Cartridge ROM
            0x0000..=0x7FFF => self.cartridge.read(addr),
            0x8000..=0x9FFF => self.vram.read(addr),

            // THIS one is for the Catridge's RAM, which fascinates me further
            // people were putting RAM in their games. LITERALLY shipping ram with the game.
            0xA000..=0xBFFF => self.cartridge.read(addr),

            //
            0xC000..=0xDFFF => self.wram.read(addr),

            // ECHO RAM
            0xE000..=0xFDFF => self.wram.read(addr - 0x2000),

            //
            0xFE00..=0xFE9F => self.oam.read(addr),

            // Nintendo forbade accessing this memory, so to be faithful
            0xFEA0..=0xFEFF => {
                // panic!("thou shall ninetendont");
                return 0x00;
            }

            // To handle I/O Stuff, keypresses etc
            0xFF00..=0xFF7F => self.io.read(addr),

            0xFF80..=0xFFFE => self.hram.read(addr),

            0xFFFF..=0xFFFF => self.io.read(addr), // FOR HANDLING INTERRUPTS

            _ => panic!("something went wrong!"),
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            // Cartridge ROM
            0x0000..=0x7FFF => {
                //something
                self.cartridge.write(addr, val)
            }
            0x8000..=0x9FFF => self.vram.write(addr, val),
            0xA000..=0xBFFF => self.cartridge.write(addr, val),

            0xC000..=0xDFFF => self.wram.write(addr, val),
            // ECHO RAM?!??!
            0xE000..=0xFDFF => self.wram.write(addr - 0x2000, val),

            0xFE00..=0xFE9F => self.oam.write(addr, val),
            // Nintendo forbade accessing this memory, so to be faithful
            0xFEA0..=0xFEFF => {
                // panic!("thou shall ninetendont");
            }
            0xFF00..=0xFF7F => self.io.write(addr, val),
            0xFF80..=0xFFFE => self.hram.write(addr, val),
            0xFFFF..=0xFFFF => self.io.write(addr, val),

            _ => panic!("something went wrong!"),
        }
    }
}
