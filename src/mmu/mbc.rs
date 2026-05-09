#[derive(Clone, Copy)]
pub enum BankingMode {
    ROM,
    RAM,
}

impl BankingMode {
    pub fn from_u8(val: u8) -> Self {
        match val & 1 {
            0 => BankingMode::ROM,
            1 => BankingMode::RAM,
            _ => panic!("Banking mode can only be 0 or 1"),
        }
    }
}

pub trait MbcTrait: Send {
    fn ram_enabled(&self) -> bool;
    fn get_rom_bank(&self) -> usize;
    fn get_ram_bank(&self) -> usize;
    fn get_banking_mode(&self) -> BankingMode;

    fn map_rom_addr(&self, addr: u16) -> usize {
        // Default implementation, should be overridden in MBC1 or other quirky
        // bank controllers
        let bank_offset = self.get_rom_bank() * 0x4000;

        // a masking trick, using 0x3FFF which is 0b11111111111111 in binary
        // a 14 bit mask. This just means, we only track the 16kb portion.
        // 0x3FFF is 16383. a really efficient way to extract just the offset
        // from the address
        let relative_addr = (addr & 0x3FFF) as usize;
        let safe_addr = relative_addr + bank_offset;
        safe_addr
    }

    fn map_ram_addr(&self, addr: u16) -> Option<usize> {
        if !self.ram_enabled() {
            return None;
        }

        // using a 13 bit mask, instead of 14 bit, since ram bank is 8kb
        let normalized_addr = (addr & 0x1FFF) as usize;
        let bank_size = 0x2000; // 8192
        let bank_offset = (self.get_ram_bank() * bank_size) as usize;

        let resolved_addr = (normalized_addr + bank_offset) as usize;

        Some(resolved_addr)
    }

    fn write_rom(&mut self, addr: u16, val: u8);
}
