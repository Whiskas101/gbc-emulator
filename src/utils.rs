// TRAITS TO MAKE MY LIFE EASIER
//
//

pub trait Memory {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);
}
