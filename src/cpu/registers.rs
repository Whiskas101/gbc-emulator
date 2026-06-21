pub struct Flag;
impl Flag {
    pub const Z: u8 = 0x80;
    pub const N: u8 = 0x40;
    pub const H: u8 = 0x20;
    pub const C: u8 = 0x10;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Reg8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Debug, Clone, Copy)]
pub enum Reg16 {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

pub struct Regs {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,

    // THIS is a very special one, not controlled directly by the programmer
    pub f: u8,
    pub h: u8,
    pub l: u8,

    // NOT making it public, hoping that capital access via a func getter
    // implies getting a 16 bit value. And direct access via public . op
    // to be an 8bit, not sure if this is the best of decisions.
    pc: u16, // 16 bit program counter
    sp: u16, // 16 bit stack pointer
}

impl Regs {
    pub fn new() -> Self {
        Regs {
            a: 0, // HIGH
            b: 0, // HIGH
            c: 0, // LOW
            d: 0, // HIGH
            e: 0, // LOW
            f: 0, // LOW

            h: 0, // HIGH
            l: 0, // LOW

            pc: 0,
            sp: 0,
        }
    }

    pub fn set_flag(&mut self, flag_mask: u8, set: bool) {
        if set {
            self.f |= flag_mask;
        } else {
            self.f &= !flag_mask;
        }
    }

    pub fn get_flag(&self, flag_mask: u8) -> bool {
        (self.f & flag_mask) != 0
    }

    pub fn update_flags(&mut self, z: bool, n: bool, h: bool, c: bool) {
        self.set_flag(Flag::Z, z);
        self.set_flag(Flag::N, n);
        self.set_flag(Flag::H, h);
        self.set_flag(Flag::C, c);
    }

    pub fn read8(&self, reg: Reg8) -> u8 {
        match reg {
            Reg8::A => self.a,
            Reg8::B => self.b,
            Reg8::C => self.c,
            Reg8::D => self.d,
            Reg8::E => self.e,
            Reg8::H => self.h,
            Reg8::L => self.l,
        }
    }

    pub fn write8(&mut self, reg: Reg8, val: u8) {
        match reg {
            Reg8::A => self.a = val,
            Reg8::B => self.b = val,
            Reg8::C => self.c = val,
            Reg8::D => self.d = val,
            Reg8::E => self.e = val,
            Reg8::H => self.h = val,
            Reg8::L => self.l = val,
        }
    }

    pub fn read16(&self, reg: Reg16) -> u16 {
        // reads the data from two registers as one 16bit value
        match reg {
            Reg16::AF => (self.a as u16) << 8 | self.f as u16,
            Reg16::BC => (self.b as u16) << 8 | self.c as u16,
            Reg16::DE => (self.d as u16) << 8 | self.e as u16,
            Reg16::HL => (self.h as u16) << 8 | self.l as u16,
            Reg16::SP => self.sp,
            Reg16::PC => self.pc,
        }
    }

    pub fn write16(&mut self, reg: Reg16, val: u16) {
        match reg {
            Reg16::AF => {
                self.a = ((val & 0xFF00) >> 8) as u8;
                // THE last nibble must be zeroed out for AF register
                // Reasons? the reference said so.
                self.f = (val & 0x00F0) as u8;
            }
            Reg16::BC => {
                self.b = ((val & 0xFF00) >> 8) as u8;
                self.c = (val & 0x00FF) as u8;
            }
            Reg16::DE => {
                self.d = ((val & 0xFF00) >> 8) as u8;
                self.e = (val & 0x00FF) as u8;
            }
            Reg16::HL => {
                self.h = ((val & 0xFF00) >> 8) as u8;
                self.l = (val & 0x00FF) as u8;
            }
            Reg16::SP => self.sp = val,
            Reg16::PC => panic!("You're not supposed to be able to write to PC using this method."),
        }
    }

    pub fn PC(&self) -> u16 {
        self.pc as u16
    }

    pub fn s_PC(&mut self, val: u16) {
        self.pc = val;
    }
}
