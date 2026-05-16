#![allow(non_snake_case)]
use core::panic;

use crate::bus;

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

impl Instruction {
    pub fn is_single_cycle(&self) -> bool {}
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
    temp_val: u16,
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

    pub fn get_flag(&mut self, flag_mask: u8) -> bool {
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

    fn execute_step(&mut self, instr: Instruction, step: u8, bus: &mut bus::Bus) {
        match instr {
            Instruction::Ld(dest, src) => {
                let val = self.regs.read8(src);
                self.regs.write8(dest, val);
                self.state = CpuState::FetchOpCode;
            }

            Instruction::LdImm(reg8) => {
                let n8 = self.fetch_advance_pc(bus);

                self.regs.write8(reg8, n8); // since it's an internal write
                // no need to actually break this out into the execution cpu state
                self.state = CpuState::FetchOpCode;
            }

            Instruction::Ld16Imm(reg16) => match step {
                0 => {
                    self.temp_val = self.fetch_advance_pc(bus) as u16; // m cycle cost
                    self.state = CpuState::Executing {
                        instr: instr,
                        step: 1,
                    }
                }
                1 => {
                    let low = self.temp_val as u16;
                    let high = self.fetch_advance_pc(bus) as u16; // m cycle cost
                    let n16 = (high << 8) | low;

                    self.regs.write16(reg16, n16);
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for Ld16Imm"),
            },

            Instruction::LdHlInd(reg8) => match step {
                0 => {
                    let dest_arr = self.regs.read16(Reg16::HL);
                    let data = self.regs.read8(reg8);

                    bus.write(dest_arr, data);
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for LdHlInd"),
            },
            Instruction::LdHlIndImm => match step {
                0 => {
                    let n8 = self.fetch_advance_pc(bus); //costs a M cycle
                    self.temp_val = n8 as u16;
                    self.state = CpuState::Executing { instr, step: 1 }
                }
                1 => {
                    let dest_addr = self.regs.read16(Reg16::HL);
                    bus.write(dest_addr, self.temp_val as u8);
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for LdHlIndImm"),
            },
            Instruction::LdRegHlInd(reg8) => match step {
                0 => {
                    // get the value pointed to by HL and put it into r8
                    let addr = self.regs.read16(Reg16::HL);
                    let val = bus.read(addr); // one m cycle to read
                    self.regs.write8(reg8, val);

                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for LdRegHlInd"),
            },
            Instruction::LdReg16IndA(reg16) => match step {
                0 => {
                    let dest_addr = self.regs.read16(reg16);
                    let val = self.regs.read8(Reg8::A);
                    bus.write(dest_addr, val);

                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for LdReg16IndA"),
            },
            Instruction::LdImmIndA => match step {
                0 => {
                    // need to resolve the memory location n16 first
                    // starting with first lower 8 bit
                    let low = self.fetch_advance_pc(bus);
                    self.temp_val = low as u16;
                    self.state = CpuState::Executing { instr, step: 1 }
                }
                1 => {
                    let high = self.fetch_advance_pc(bus);
                    self.temp_val |= (high as u16) << 8;

                    self.state = CpuState::Executing { instr, step: 2 }
                }
                2 => {
                    // FINALLY have the full addr in temp_val
                    // safe to write
                    let val = self.regs.read8(Reg8::A);
                    bus.write(self.temp_val, val);
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid Step for LdImmIndA"),
            },
            Instruction::LdhImm8IndA => match step {
                // THIS is special in that even though it's going to write
                // a u8, it assumes a high byte of 0xFF
                // Bringing the actual write as a value between
                // 0xFF00 to 0xFFFF
                // INFO: Apparently this instruction and it's family
                // The LDH (H being the suffix for HRAM)
                // Was built for talking to the HRAM, because typical LD
                // was too slow.
                0 => {
                    // resolve the addr
                    let dest = self.fetch_advance_pc(bus);
                    // we don't care about figuring out the "high" half,
                    // we just attach the 0xFF00 to the fetched n8
                    self.temp_val = 0xFF00 | (dest as u16); // becomes a n16, with
                    // prepended value

                    self.state = CpuState::Executing { instr, step: 1 };
                }
                1 => {
                    // write op consuming another cycle
                    let val = self.regs.read8(Reg8::A);
                    bus.write(self.temp_val, val);
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid Step for LdhImm8IndA"),
            },
            Instruction::LdhCIndA => match step {
                0 => {
                    // A refers to the accumulator
                    // assume it has "accumulated" the  value into register A already
                    let val = self.regs.read8(Reg8::A); // cpu regs are zero cycle reads
                    // get the addr from reg C, which is also free, cause it's on the cpu
                    let target_addr = self.regs.read8(Reg8::C);
                    let target_addr = (0xFF00) | (target_addr as u16);
                    bus.write(target_addr, val);

                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid Step for LdhCIndA"),
            },
            Instruction::LdAReg16Ind(reg16) => match step {
                0 => {
                    // load into A, the data pointed to by the addr in
                    // the given reg
                    let addr = self.regs.read16(reg16);
                    let val = bus.read(addr);

                    self.regs.write8(Reg8::A, val);
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid Step for LdAReg16Ind"),
            },
            Instruction::LdAImm16Ind => match step {
                0 => {
                    // read the data by advancing the program counter
                    // since it's a 16 bit value
                    // will have to do this twice across two steps

                    self.temp_val = self.fetch_advance_pc(bus) as u16;
                    self.state = CpuState::Executing { instr, step: 1 };
                }
                1 => {
                    self.temp_val = self.temp_val | (self.fetch_advance_pc(bus) as u16) << 8;
                    self.state = CpuState::Executing { instr, step: 2 };
                }
                2 => {
                    let val = bus.read(self.temp_val);
                    self.regs.write8(Reg8::A, val);
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid Step for LdAImm16Ind"),
            },
            Instruction::LdhAImm8Ind => match step {
                0 => {
                    // get the lower bits
                    self.temp_val = self.fetch_advance_pc(bus) as u16;
                    // ensure that the upper bits are ONES
                    // That's what LDH is for.
                    self.temp_val |= 0xFF00;
                    self.state = CpuState::Executing { instr, step: 1 };
                }
                1 => {
                    // load the data at that temp_val addr into A
                    let val = bus.read(self.temp_val);
                    self.regs.write8(Reg8::A, val);
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for LdhAImm8Ind"),
            },
            Instruction::LdhACInd => match step {
                0 => {
                    // LDH Load high, so 0xFF00 high bits assumption
                    // Load the lower bits addr data from C
                    let addr = 0xFF00 | self.regs.read8(Reg8::C) as u16;
                    let val = bus.read(addr);
                    // Loading the temp val into the actual register
                    self.regs.write8(Reg8::A, val);
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for LdhACInd"),
            },
            Instruction::LdHliA => match step {
                // copy the data in reg A to the byte pointed to by HL
                // and then increment it
                0 => {
                    let val = self.regs.read8(Reg8::A);

                    let target_addr = self.regs.read16(Reg16::HL);
                    bus.write(target_addr, val);

                    // increment the actual update
                    self.regs.write16(Reg16::HL, target_addr.wrapping_add(1));

                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for LdHliA"),
            },
            Instruction::LdHldA => match step {
                // copy the data in reg A to the byte pointed to by HL
                // and then decrement it
                0 => {
                    let val = self.regs.read8(Reg8::A);

                    let target_addr = self.regs.read16(Reg16::HL);
                    bus.write(target_addr, val);

                    // increment the actual update
                    self.regs.write16(Reg16::HL, target_addr.wrapping_sub(1));

                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for LdHldA"),
            },
            Instruction::LdAhli => match step {
                // copying the byte pointed to by HL into reg A, then
                // incrementing HL
                0 => {
                    // resolve the val
                    let target_addr = self.regs.read16(Reg16::HL);
                    let val = bus.read(target_addr);

                    self.regs.write8(Reg8::A, val);

                    // inc
                    self.regs.write16(Reg16::HL, target_addr.wrapping_add(1));
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for LdAhli"),
            },
            Instruction::LdAHld => match step {
                // copying the byte pointed to by HL into reg A, then
                // incrementing HL
                0 => {
                    // resolve the val
                    let target_addr = self.regs.read16(Reg16::HL);
                    let val = bus.read(target_addr);

                    self.regs.write8(Reg8::A, val);

                    // inc
                    self.regs.write16(Reg16::HL, target_addr.wrapping_sub(1));
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for LdAhld"),
            },
            Instruction::Adc(operand8) => match operand8 {
                Operand8::Reg(reg8) => match step {
                    0 => {
                        let A = self.regs.read8(Reg8::A) as u16;
                        let r8 = self.regs.read8(reg8) as u16;
                        let carry = if self.regs.get_flag(Flag::C) { 1 } else { 0 };
                        let result = A + r8 + carry;

                        // half carry check
                        // essentially see if the sum of the LOWER
                        // nibbles leads to a overflow within the 4 bits
                        let h = (A & 0xF) + (r8 & 0xF) + carry > 0xF;

                        let c = result > 0xFF; // bigeer than 8 bit number =
                        // carry
                        //

                        // final result (we only cary about the last 8 bits)
                        let result = (result & 0xFF) as u8;

                        // accumulate that res in to A
                        self.regs.write8(Reg8::A, result);

                        self.regs.update_flags(result == 0, false, h, c);

                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Adc"),
                },
                Operand8::HlInd => match step {
                    0 => {}
                    1 => {}
                    _ => panic!("Invalid step for Adc"),
                },
            },
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
