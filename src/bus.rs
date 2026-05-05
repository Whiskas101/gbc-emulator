use std::ops::RangeInclusive;

use crate::mmu::{Cartridge, Hram, Wram};

pub struct Bus {
    wram: Wram,
    hram: Hram,
    cartridge: Cartridge,
}

impl Bus {
    pub fn resolve(&self, addr: u16) {
        match addr {
            // Cartridge ROM (fixed)
            0x0000..=0x3FFF => {}

            // Cartridge ROM (switchable bank thing idk how it works exactly? )
            0x4000..=0x7FFF => {}

            _ => panic!("something went wrong!"),
        }
    }
}
