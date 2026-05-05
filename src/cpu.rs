const Z_FLAG: u8 = 0x80;
const N_FLAG: u8 = 0x40;
const H_FLAG: u8 = 0x20;
const C_FLAG: u8 = 0x10;

struct GameBoyCPU {
    // Where it all begins! (kinda)
    reg: Reg,
}

pub struct Reg {
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

impl Reg {
    pub fn new() -> Self {
        Reg {
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

    pub fn AF(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f as u16)
    }

    pub fn BC(&self) -> u16 {
        ((self.b as u16) << 8) | self.c as u16
    }

    pub fn DE(&self) -> u16 {
        ((self.d as u16) << 8) | self.e as u16
    }

    pub fn HL(&self) -> u16 {
        ((self.h as u16) << 8) | self.l as u16
    }

    pub fn SP(&self) -> u16 {
        self.sp as u16
    }
    pub fn PC(&self) -> u16 {
        self.pc as u16
    }

    pub fn s_AF(&mut self, val: u16) {
        // set the high bits
        self.a = ((val & 0xFF00) >> 8) as u8; // get MSB 8 bits
        // FOR SOME reason, the LAST 4 bits in F must ALWAYS, ALWAYS BE ZERO.
        self.f = (val & 0x00F0) as u8; // get the last 8 bits (mask ensures last 4 bits are zero)
    }
    pub fn s_BC(&mut self, val: u16) {
        self.b = ((val & 0xFF00) >> 8) as u8; // get MSB 8 bits
        self.c = (val & 0x00FF) as u8; // get the last 8 bits
    }
    pub fn s_DE(&mut self, val: u16) {
        self.d = ((val & 0xFF00) >> 8) as u8; // get MSB 8 bits
        self.e = (val & 0x00FF) as u8; // get the last 8 bits
    }
    pub fn s_HL(&mut self, val: u16) {
        self.h = ((val & 0xFF00) >> 8) as u8; // get MSB 8 bits
        self.l = (val & 0x00FF) as u8; // get the last 8 bits
    }
    pub fn s_SP(&mut self, val: u16) {
        self.sp = val;
    }
    pub fn s_PC(&mut self, val: u16) {
        self.pc = val;
    }
}

impl GameBoyCPU {
    fn new() -> Self {
        Self { reg: Reg::new() }
    }

    // fn fetch(&self) -> u16 {
    //     // fetch the next instruction
    // }

    // fn decode() -> GBCInst{
    //     // decode the next instruction
    // }
    //
    // fn execute() -> {
    //     // exec baby
    // }
}
