#![allow(non_snake_case)]
use crate::bus;
pub const Z_FLAG: u8 = 0x80;
pub const N_FLAG: u8 = 0x40;
pub const H_FLAG: u8 = 0x20;
pub const C_FLAG: u8 = 0x10;

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
}

#[derive(Debug, Clone, Copy)]
pub enum Cond {
    NotZero,  // NZ
    Zero,     // Z
    NotCarry, // NC
    Carry,    // C
    Always,   // for unconditional jumping
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operand8 {
    Reg(Reg8),
    HlInd,
}

// Thanks to https://rgbds.gbdev.io/docs/v1.0.1/gbz80.7
// For the gbz80 cpu instruction reference
// The instruction ordering here matches the reference exactly
// (i sure hope it does)
#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    // LOAD INSTRUCTIONS
    Ld(Reg8, Reg8),     // LD r8, r8
    LdImm(Reg8),        // LD r8, n8
    Ld16Imm(Reg16),     // LD r16, n16
    LdHlInd(Reg8),      // LD [HL], r8
    LdHlIndImm,         // LD [HL], n8
    LdRegHlInd(Reg8),   // LD r8, [HL]
    LdReg16IndA(Reg16), // LD [r16], A
    LdImmIndA,          // LD [n16], A
    LdhImm8IndA,        // LDH [n16], A // n16 is encoded as 8bit low byte, hence the Imm8
    LdhCIndA,           // LDH [C], A
    LdAReg16Ind(Reg16), // LD A, [r16]
    LdAImm16Ind,        // LD A, [n16]
    LdhAImm8Ind,        // LDH A, [n16] // again n16 encoded as 8bit low byte & highbyte as FF
    LdhACInd,           // LDH A, [C]
    LdHliA,             // LD [HLI], A
    LdHldA,             // LD [HLD], A
    LdAhli,             // LD A, [HLI]
    LdAHld,             // LD A, [HLD]

    // 8 BIT ARITHMETIC INSTRUCTIONS
    Adc(Operand8), // ADC A,r8 || ADC A,[HL]
    AdcImm,        // ADC A,n8
    Add(Operand8), // ADD A,r8 || ADD A,[HL]
    AddImm,        // ADD A,n8
    Cp(Operand8),  // CP A,r8 || CP A,[HL]
    CpImm,         // CP A,n8
    Dec(Operand8), // DEC r8 || DEC [HL]
    Inc(Operand8), // INC r8 || INC [HL]
    Sbc(Operand8), // SBC A,r8 || SBC A,[HL]
    SbcImm,        // SBC A,n8
    Sub(Operand8), // SUB A,r8 || SUB A,[HL]
    SubImm,        // SUB A,n8

    // 16 BIT ARITHMETIC INSTRUCTIONS
    Add16(Reg16), //ADD HL,r16
    Dec16(Reg16), // DEC r16
    Inc16(Reg16), // INC r16

    // BITWISE LOGIC INST
    And(Operand8), // AND A,r8 || AND A,[HL]
    AndImm,        // AND A,n8
    Cpl,           // CPL
    Or(Operand8),  // OR A,r8 || OR A,[HL]
    OrImm,         // OR A,n8
    Xor(Operand8), // XOR A,r8 || XOR A,[HL]
    XorImm,        // XOR A,n8

    // BIT FLAG INST
    Bit(u8, Operand8), //BIT u3,r8 || BIT u3,[HL]
    Res(u8, Operand8), // RES u3,r8 || RES u3,[HL]
    Set(u8, Operand8), // SET u3,r8 || SET u3,[HL]

    // BIT SHIFT INST
    Rl(Operand8),   // RL r8 || RL [HL]
    RlA,            // RLA
    Rlc(Operand8),  // RLC r8 || RLC [HL]
    RlcA,           // RLCA
    Rr(Operand8),   // RR r8 || RR [HL]
    RrA,            // RRA
    Rrc(Operand8),  // RRC r8 || RRC [HL]
    RrcA,           // RRCA
    Sla(Operand8),  // SLA r8 || SLA [HL]
    Sra(Operand8),  // SRA r8 || SRA [HL]
    Srl(Operand8),  // SRL r8 || SRL [HL]
    Swap(Operand8), // SWAP r8 || SWAP [HL]

    // JUMPS n SUBROUTINE Inst
    Call(Cond), // CALL n16 || CALL cc,n16
    JpHl,       // JP HL
    Jp(Cond),   // JP n16 || JP cc,n16
    Jr(Cond),   // JR n16 || JR cc,n16
    Ret(Cond),  // RET cc || RET
    Reti,       // RETI
    Rst(u8),    // RST vec

    // CARRY FLAG INST
    Ccf, // CCF
    Scf, // SCF

    // STACK MANIPULATION INST
    //        // ADD HL,SP // Already covered by Add16(Reg16::SP)
    AddSpImm, // ADD SP,e8
    //        // DEC SP // Already covered by Dec16(Reg16::SP)
    //        // INC SP // Already covered by Inc16(Reg16::SP)
    //        // LD SP,n16 // Already covered by Ld16Imm(Reg16::SP)
    LdImm16Sp,   // LD [n16],SP
    LdHlSpImm,   // LD HL,SP+e8
    LdSpHl,      // LD SP,HL
    Pop(Reg16),  // POP AF || POP r16
    Push(Reg16), // PUSH AF || PUSH r16

    // INTERRUPT RELATED INST
    Di,   // DI
    Ei,   // EI
    Halt, // HALT

    // Miscellaneous Inst
    Daa,  // DAA
    Nop,  // NOP
    Stop, // STOP
}

pub enum CpuState {
    // ready to read the next inst
    FetchOpCode,
    FetchCbOpCode,

    Decode { opcode: u8 },
    DecodeCb { cb_opcode: u8 },

    // executing an instruction, and what step within that instruction it is at
    // currently
    Executing { instr: Instruction, step: u8 },

    // Waiting for some interrupt
    Halted,
}

pub struct GameBoyCPU {
    // Where it all begins! (kinda)
    pub state: CpuState,
    regs: Regs,
    temp_val: u8,
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
        match reg {
            Reg16::AF => (self.a as u16) << 8 | self.f as u16,
            Reg16::BC => (self.b as u16) << 8 | self.c as u16,
            Reg16::DE => (self.d as u16) << 8 | self.e as u16,
            Reg16::HL => (self.h as u16) << 8 | self.l as u16,
            Reg16::SP => self.sp,
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
        }
    }

    pub fn PC(&self) -> u16 {
        self.pc as u16
    }

    pub fn s_PC(&mut self, val: u16) {
        self.pc = val;
    }
}

impl GameBoyCPU {
    fn new() -> Self {
        Self {
            regs: Regs::new(),
            state: CpuState::FetchOpCode,
            temp_val: 0,
        }
    }

    // MEANT to simulate 1 Machine cycle
    // which is 4 T states
    fn fetch_advance_pc(&mut self, bus: &bus::Bus) -> u8 {
        let pc = self.regs.pc;
        let val = bus.read(pc);

        self.regs.s_PC(pc.wrapping_add(1));
        val
    }

    fn execute_step(&mut self, inst: Instruction, step: u8, bus: &mut bus::Bus) {
        match inst {
            Instruction::Ld(reg8, reg9) => todo!(),
            Instruction::LdImm(reg8) => todo!(),
            Instruction::Ld16Imm(reg16) => todo!(),
            Instruction::LdHlInd(reg8) => todo!(),
            Instruction::LdHlIndImm => todo!(),
            Instruction::LdRegHlInd(reg8) => todo!(),
            Instruction::LdReg16IndA(reg16) => todo!(),
            Instruction::LdImmIndA => todo!(),
            Instruction::LdhImm8IndA => todo!(),
            Instruction::LdhCIndA => todo!(),
            Instruction::LdAReg16Ind(reg16) => todo!(),
            Instruction::LdAImm16Ind => todo!(),
            Instruction::LdhAImm8Ind => todo!(),
            Instruction::LdhACInd => todo!(),
            Instruction::LdHliA => todo!(),
            Instruction::LdHldA => todo!(),
            Instruction::LdAhli => todo!(),
            Instruction::LdAHld => todo!(),
            Instruction::Adc(operand8) => todo!(),
            Instruction::AdcImm => todo!(),
            Instruction::Add(operand8) => todo!(),
            Instruction::AddImm => todo!(),
            Instruction::Cp(operand8) => todo!(),
            Instruction::CpImm => todo!(),
            Instruction::Dec(operand8) => todo!(),
            Instruction::Inc(operand8) => todo!(),
            Instruction::Sbc(operand8) => todo!(),
            Instruction::SbcImm => todo!(),
            Instruction::Sub(operand8) => todo!(),
            Instruction::SubImm => todo!(),
            Instruction::Add16(reg16) => todo!(),
            Instruction::Dec16(reg16) => todo!(),
            Instruction::Inc16(reg16) => todo!(),
            Instruction::And(operand8) => todo!(),
            Instruction::AndImm => todo!(),
            Instruction::Cpl => todo!(),
            Instruction::Or(operand8) => todo!(),
            Instruction::OrImm => todo!(),
            Instruction::Xor(operand8) => todo!(),
            Instruction::XorImm => todo!(),
            Instruction::Bit(_, operand8) => todo!(),
            Instruction::Res(_, operand8) => todo!(),
            Instruction::Set(_, operand8) => todo!(),
            Instruction::Rl(operand8) => todo!(),
            Instruction::RlA => todo!(),
            Instruction::Rlc(operand8) => todo!(),
            Instruction::RlcA => todo!(),
            Instruction::Rr(operand8) => todo!(),
            Instruction::RrA => todo!(),
            Instruction::Rrc(operand8) => todo!(),
            Instruction::RrcA => todo!(),
            Instruction::Sla(operand8) => todo!(),
            Instruction::Sra(operand8) => todo!(),
            Instruction::Srl(operand8) => todo!(),
            Instruction::Swap(operand8) => todo!(),
            Instruction::Call(cond) => todo!(),
            Instruction::JpHl => todo!(),
            Instruction::Jp(cond) => todo!(),
            Instruction::Jr(cond) => todo!(),
            Instruction::Ret(cond) => todo!(),
            Instruction::Reti => todo!(),
            Instruction::Rst(_) => todo!(),
            Instruction::Ccf => todo!(),
            Instruction::Scf => todo!(),
            Instruction::AddSpImm => todo!(),
            Instruction::LdImm16Sp => todo!(),
            Instruction::LdHlSpImm => todo!(),
            Instruction::LdSpHl => todo!(),
            Instruction::Pop(reg16) => todo!(),
            Instruction::Push(reg16) => todo!(),
            Instruction::Di => todo!(),
            Instruction::Ei => todo!(),
            Instruction::Halt => todo!(),
            Instruction::Daa => todo!(),
            Instruction::Nop => todo!(),
            Instruction::Stop => todo!(),
        }
    }

    fn tick(&mut self, bus: &mut bus::Bus) {
        // Fetch

        // Decode
        // Execute
    }

    fn execute(&mut self, opcode: u8, bus: &bus::Bus) {}
}
