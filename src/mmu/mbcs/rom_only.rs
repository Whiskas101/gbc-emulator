pub use crate::mmu::MbcTrait;
pub use crate::mmu::mbc::BankingMode;

pub struct RomOnly;
impl RomOnly {
    pub fn new() -> Self {
        Self
    }
}
impl MbcTrait for RomOnly {
    fn ram_enabled(&self) -> bool {
        false
    }
    fn get_rom_bank(&self) -> usize {
        1
    }
    fn get_ram_bank(&self) -> usize {
        0
    }
    fn get_banking_mode(&self) -> BankingMode {
        BankingMode::ROM
    }
    fn map_rom_addr(&self, addr: u16) -> usize {
        addr as usize
    }
    fn write_rom(&mut self, _: u16, _: u8) {}
}
