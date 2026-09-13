use crate::utils::Memory;

pub struct Io {
    sb: u8,         // this 0xFF01
    sc: u8,         // 0xFF02
    pub if_reg: u8, //0xFF0F
}

impl Memory for Io {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF01 => self.sb,
            0xFF02 => self.sc,
            0xFF0F => self.if_reg,
            0xFFFF => 0x0, // IE register
            _ => 0xFF,
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        // TODO:
        println!("Writing!: {:?}", value);
        match addr {
            0xFF01 => self.sb = value,
            0xFF02 => {
                self.sc = value;
                if value == 0x81 {
                    print!("{}", self.sb as char);
                    self.sc = 0x00;
                }
            }
            0xFF0F => self.if_reg = value,
            _ => {}
        }
    }
}
