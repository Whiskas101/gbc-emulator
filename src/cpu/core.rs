#![allow(non_snake_case)]
use super::registers::{Reg8, Reg16, Regs};
use crate::bus;
use crate::cpu::{Cond, Flag, Instruction, Operand8};
use core::panic;

pub enum CpuState {
    // ready to read the next inst
    FetchOpCode,
    FetchCbOpCode,

    // executing an instruction, and what step within that instruction it is at
    // currently
    Executing { instr: Instruction, step: u8 },

    // Waiting for some interrupt
    Halted,
}

struct IncResult {
    res: u8,
    flags: (bool, bool, bool, bool),
}

pub struct GameBoyCPU {
    // Where it all begins! (kinda)
    pub state: CpuState,
    pub ime: bool,
    regs: Regs,
    temp_val: u16,
}

impl GameBoyCPU {
    pub fn new() -> Self {
        Self {
            regs: Regs::new(),
            state: CpuState::FetchOpCode,
            temp_val: 0,
            ime: false,
        }
    }

    // MEANT to simulate 1 Machine cycle
    // which is 4 T states
    fn fetch_advance_pc(&mut self, bus: &bus::Bus) -> u8 {
        let pc = self.regs.read16(Reg16::PC);
        let val = bus.read(pc);

        self.regs.s_PC(pc.wrapping_add(1));
        val
    }

    fn check_cond(&self, cond: Cond) -> bool {
        match cond {
            Cond::NotZero => !self.regs.get_flag(Flag::Z),
            Cond::Zero => self.regs.get_flag(Flag::Z),
            Cond::NotCarry => !self.regs.get_flag(Flag::C),
            Cond::Carry => self.regs.get_flag(Flag::C),
            Cond::Always => true,
        }
    }

    fn alu_add(&mut self, val: u8, use_carry: bool) {
        let a = self.regs.read8(Reg8::A) as u16;
        let val16 = val as u16;
        let carry = if use_carry && self.regs.get_flag(Flag::C) {
            1
        } else {
            0
        };

        let result = a + val16 + carry;

        let z = (result & 0xFF) == 0;
        let n = false;
        let h = ((a & 0xF) + (val16 & 0xF) + carry) > 0x0F;
        let c = result > 0xFF;

        self.regs.write8(Reg8::A, (result & 0xFF) as u8);
        self.regs.update_flags(z, n, h, c);
    }

    fn alu_sub(&mut self, val: u8, use_carry: bool) {
        let a = self.regs.read8(Reg8::A);
        let carry = if use_carry && self.regs.get_flag(Flag::C) {
            1
        } else {
            0
        };

        let result = a.wrapping_sub(val).wrapping_sub(carry);
        let z = result == 0;
        let n = true; // cause it's a subtraction

        // This ones a real confusing trick
        // A - val - carry is equivalent to A - (val + carry) because mafs
        // In that case, the only way an under flow can occur is if A is smaller than the RHS,
        // (val + carry), which is exactly what the below flags use:
        // A < val + carry = UNDERFLOW
        let h = (a & 0x0F) < (val & 0x0F) + carry;
        let c = (a as u16) < (val as u16) + (carry as u16);

        self.regs.write8(Reg8::A, result);
        self.regs.update_flags(z, n, h, c);
    }

    fn alu_cp(&mut self, val: u8) {
        // identical to SUB, it just doesn't write the results to A
        // fascinating stuff
        let a = self.regs.read8(Reg8::A);
        let result = a.wrapping_sub(val);

        let z = result == 0;
        let n = true;
        let h = (a & 0x0F) < (val & 0x0F);
        let c = a < val;

        self.regs.update_flags(z, n, h, c);
    }

    fn alu_and(&mut self, val: u8) {
        let result = self.regs.read8(Reg8::A) & val;
        self.regs.write8(Reg8::A, result);
        // and op always sets H to 1
        self.regs.update_flags(result == 0, false, true, false);
    }

    fn alu_or(&mut self, val: u8) {
        let result = self.regs.read8(Reg8::A) | val;
        self.regs.write8(Reg8::A, result);
        self.regs.update_flags(result == 0, false, false, false);
    }

    fn alu_xor(&mut self, val: u8) {
        let result = self.regs.read8(Reg8::A) ^ val;
        self.regs.write8(Reg8::A, result);
        self.regs.update_flags(result == 0, false, false, false);
    }

    fn inc8(&mut self, val: u8) -> IncResult {
        // since it's the 8 bit variant it must handle the flags as well
        let result = val.wrapping_add(1);

        let z = result == 0;
        let n = false;
        let h = (val & 0xF) + 1 > 0x0F;
        let c = self.regs.get_flag(Flag::C); // no change to carry flag

        return IncResult {
            res: result,
            flags: (z, n, h, c),
        };
    }

    fn dec8(&mut self, val: u8) -> IncResult {
        let result = val.wrapping_sub(1);

        let z = result == 0;
        let n = true;
        let h = (val & 0xF) < 1; // can only underflow if the current val is 0
        let c = self.regs.get_flag(Flag::C); // no change to carry flag

        return IncResult {
            res: result,
            flags: (z, n, h, c),
        };
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
                        let r8 = self.regs.read8(reg8);
                        self.alu_add(r8, true);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Adc"),
                },
                Operand8::HlInd => match step {
                    // add the byte pointed to by HL plus the carry flag to A
                    // like how ADC A, r8 works
                    0 => {
                        // get the byte pointed to by HL
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = bus.read(target_addr);
                        self.alu_add(val, true);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Adc"),
                },
            },
            Instruction::AdcImm => match step {
                // Add the value n8 PLUS the carry flag to A
                0 => {
                    let n8 = self.fetch_advance_pc(bus);
                    self.alu_add(n8, true);
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for AdcImm"),
            },
            Instruction::Add(operand8) => match operand8 {
                Operand8::Reg(reg) => match step {
                    0 => {
                        let val = self.regs.read8(reg);
                        self.alu_add(val, false);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Add"),
                },
                Operand8::HlInd => match step {
                    0 => {
                        // resolve the value indicated by the address in HL
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = bus.read(target_addr);
                        self.alu_add(val, false);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Add"),
                },
            },
            Instruction::AddImm => match step {
                0 => {
                    // get n8
                    let val = self.fetch_advance_pc(bus);
                    self.alu_add(val, false);
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for AddImm"),
            },
            Instruction::Cp(operand8) => match operand8 {
                Operand8::Reg(reg) => match step {
                    0 => {
                        let val = self.regs.read8(reg);
                        self.alu_cp(val);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Cp"),
                },
                Operand8::HlInd => match step {
                    0 => {
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = bus.read(target_addr);
                        self.alu_cp(val);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Cp"),
                },
            },
            Instruction::CpImm => match step {
                0 => {
                    let val = self.fetch_advance_pc(bus);
                    self.alu_cp(val);
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for CpImm"),
            },
            Instruction::Dec(operand8) => match operand8 {
                Operand8::Reg(reg8) => match step {
                    0 => {
                        let val = self.regs.read8(reg8);
                        let IncResult {
                            res,
                            flags: (z, n, h, c),
                        } = self.dec8(val);

                        self.regs.write8(reg8, res);
                        self.regs.update_flags(z, n, h, c);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Dec"),
                },
                Operand8::HlInd => match step {
                    0 => {
                        // dec the byte pointed to by HL
                        let target_addr = self.regs.read16(Reg16::HL);
                        self.temp_val = bus.read(target_addr) as u16;
                        self.state = CpuState::Executing { instr, step: 1 };
                    }
                    1 => {
                        // need to get this addr data again :sob:
                        let target_addr = self.regs.read16(Reg16::HL);

                        let val = self.temp_val;
                        let IncResult {
                            res,
                            flags: (z, n, h, c),
                        } = self.dec8(val as u8);

                        bus.write(target_addr, res);
                        self.regs.update_flags(z, n, h, c);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Dec"),
                },
            },
            Instruction::Inc(operand8) => match operand8 {
                Operand8::Reg(reg8) => match step {
                    0 => {
                        let val = self.regs.read8(reg8);
                        let IncResult {
                            res,
                            flags: (z, n, h, c),
                        } = self.inc8(val);

                        self.regs.write8(reg8, res);
                        self.regs.update_flags(z, n, h, c);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Inc"),
                },
                Operand8::HlInd => match step {
                    0 => {
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = bus.read(target_addr);
                        self.temp_val = val as u16;
                        self.state = CpuState::Executing { instr, step: 1 };
                    }
                    1 => {
                        let target_addr = self.regs.read16(Reg16::HL);

                        let val = self.temp_val;

                        let IncResult {
                            res,
                            flags: (z, n, h, c),
                        } = self.inc8(val as u8);

                        bus.write(target_addr, res);
                        self.regs.update_flags(z, n, h, c);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Inc"),
                },
            },
            Instruction::Sbc(operand8) => match operand8 {
                Operand8::Reg(reg8) => match step {
                    0 => {
                        let val = self.regs.read8(reg8);
                        self.alu_sub(val, true);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Sbc"),
                },
                Operand8::HlInd => match step {
                    0 => {
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = bus.read(target_addr);
                        self.alu_sub(val, true);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Sbc"),
                },
            },
            Instruction::SbcImm => match step {
                0 => {
                    let n8 = self.fetch_advance_pc(bus);
                    self.alu_sub(n8, true);
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for SbcImm"),
            },
            Instruction::Sub(operand8) => match operand8 {
                Operand8::Reg(reg8) => match step {
                    0 => {
                        let val = self.regs.read8(reg8);
                        self.alu_sub(val, false);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Sub"),
                },
                Operand8::HlInd => match step {
                    0 => {
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = bus.read(target_addr);
                        self.alu_sub(val, false);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Sub"),
                },
            },
            Instruction::SubImm => match step {
                0 => {
                    let n8 = self.fetch_advance_pc(bus);
                    self.alu_sub(n8, false);
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for SubImm"),
            },
            Instruction::Add16(reg16) => match step {
                0 => {
                    // add the val in reg16 to HL
                    let val = self.regs.read16(reg16);
                    let hl_val = self.regs.read16(Reg16::HL);
                    let (res, carry) = hl_val.overflowing_add(val);

                    let z = self.regs.get_flag(Flag::Z);
                    let n = false;
                    let h = (val & 0xFFF) + (hl_val & 0xFFF) > 0xFFF;
                    let c = carry;

                    self.regs.write16(Reg16::HL, res);
                    self.regs.update_flags(z, n, h, c);
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for Add16"),
            },
            Instruction::Dec16(reg16) => match step {
                0 => {
                    // decrement the value in r16 by 1
                    let val = self.regs.read16(reg16);
                    let res = val.wrapping_sub(1);

                    self.regs.write16(reg16, res);
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for Dec16"),
            }, // these inc16 and dec16 take two cycles
            Instruction::Inc16(reg16) => match step {
                0 => {
                    // increment the value in r16 by 1
                    let val = self.regs.read16(reg16);
                    let res = val.wrapping_add(1);

                    self.regs.write16(reg16, res);
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for Inc16"),
            },

            Instruction::And(operand8) => match operand8 {
                Operand8::Reg(reg8) => match step {
                    0 => {
                        let val = self.regs.read8(reg8);
                        self.alu_and(val);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for And"),
                },
                Operand8::HlInd => match step {
                    0 => {
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = bus.read(target_addr);
                        self.alu_and(val);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for And"),
                },
            },
            Instruction::AndImm => match step {
                0 => {
                    let val = self.fetch_advance_pc(bus);
                    self.alu_and(val);
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for AndImm"),
            },
            Instruction::Cpl => match step {
                // this instruction is essentially a bitwise NOT
                // complements the current value of the Reg::A
                0 => {
                    let val = self.regs.read8(Reg8::A);
                    self.regs.write8(Reg8::A, !val);
                    let z = self.regs.get_flag(Flag::Z);
                    let c = self.regs.get_flag(Flag::C);
                    self.regs.update_flags(z, true, true, c);
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for Cpl"),
            },
            Instruction::Or(operand8) => match operand8 {
                Operand8::Reg(reg8) => match step {
                    0 => {
                        let val = self.regs.read8(reg8);
                        self.alu_or(val);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Or"),
                },
                Operand8::HlInd => match step {
                    0 => {
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = bus.read(target_addr);
                        self.alu_or(val);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Or"),
                },
            },
            Instruction::OrImm => match step {
                0 => {
                    let val = self.fetch_advance_pc(bus);
                    self.alu_or(val);
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for OrImm"),
            },
            Instruction::Xor(operand8) => match operand8 {
                Operand8::Reg(reg8) => match step {
                    0 => {
                        let val = self.regs.read8(reg8);
                        self.alu_xor(val);
                        self.state = CpuState::FetchOpCode
                    }
                    _ => panic!("Invalid step for Xor"),
                },
                Operand8::HlInd => match step {
                    0 => {
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = bus.read(target_addr);
                        self.alu_xor(val);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Xor"),
                },
            },
            Instruction::XorImm => match step {
                0 => {
                    let val = self.fetch_advance_pc(bus);
                    self.alu_xor(val);
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for XorImm"),
            },
            Instruction::Bit(bit, operand8) => match operand8 {
                Operand8::Reg(reg8) => match step {
                    0 => {
                        // Test bit u3
                        // (bit var is a u8, but only will have u3 capacity)
                        // from register r8
                        let val = self.regs.read8(reg8);
                        let mask = (1 as u8) << bit;

                        // if the value remains the same after the and
                        // the bit test result was 1
                        let z = !((val & mask) == mask);
                        let n = false;
                        let h = true;
                        let c = self.regs.get_flag(Flag::C); // keep as is

                        self.regs.update_flags(z, n, h, c);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Bit"),
                },
                Operand8::HlInd => match step {
                    0 => {
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = bus.read(target_addr);
                        let mask = (1 as u8) << bit;

                        // if the value remains the same after the and
                        // the bit test result was 1
                        let z = (val & mask) == 0;
                        let n = false;
                        let h = true;
                        let c = self.regs.get_flag(Flag::C); // keep as is

                        self.regs.update_flags(z, n, h, c);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Bit"),
                },
            },
            Instruction::Res(bit, operand8) => match operand8 {
                Operand8::Reg(reg8) => match step {
                    0 => {
                        let val = self.regs.read8(reg8);
                        let mask = !((1 as u8) << bit);
                        let result = val & mask;
                        self.regs.write8(reg8, result);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Res"),
                },
                Operand8::HlInd => match step {
                    0 => {
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = bus.read(target_addr);
                        self.temp_val = val as u16;

                        self.state = CpuState::Executing { instr, step: 1 };
                    }
                    1 => {
                        let target_addr = self.regs.read16(Reg16::HL);

                        let val = self.temp_val as u8;
                        let mask = !((1 as u8) << bit);
                        let result = val & mask;

                        bus.write(target_addr, result);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Res"),
                },
            },
            Instruction::Set(bit, operand8) => match operand8 {
                Operand8::Reg(reg8) => match step {
                    0 => {
                        let val = self.regs.read8(reg8);
                        let mask = (1 as u8) << bit;
                        let result = val | mask;
                        self.regs.write8(reg8, result);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Set"),
                },
                Operand8::HlInd => match step {
                    0 => {
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = bus.read(target_addr);
                        self.temp_val = val as u16;

                        self.state = CpuState::Executing { instr, step: 1 };
                    }
                    1 => {
                        let val = self.temp_val as u8;
                        let target_addr = self.regs.read16(Reg16::HL);

                        let mask = (1 as u8) << bit;
                        let result = val | mask;
                        bus.write(target_addr, result);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Set"),
                },
            },
            Instruction::Rl(operand8) => match operand8 {
                Operand8::Reg(reg8) => match step {
                    0 => {
                        // rotate left, data in reg 8 through the carry bit as
                        // the rightmost bit
                        let r8 = self.regs.read8(reg8);
                        let carry = self.regs.get_flag(Flag::C);
                        let carry_bit = match carry {
                            true => 1,
                            false => 0,
                        };

                        let new_carry = r8 & (1 << 7);
                        // discard the MSB, and insert the carry bit.
                        let shifted_r8 = ((r8) << 1) | (carry_bit);
                        let z = shifted_r8 == 0;
                        let n = false;
                        let h = false;
                        let c = new_carry != 0;
                        self.regs.write8(reg8, shifted_r8);
                        self.regs.update_flags(z, n, h, c);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Rl"),
                },
                Operand8::HlInd => match step {
                    0 => {
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = bus.read(target_addr);
                        self.temp_val = val as u16;
                        self.state = CpuState::Executing { instr, step: 1 };
                    }
                    1 => {
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = self.temp_val as u8;
                        let new_carry = val & (1 << 7);
                        let carry = self.regs.get_flag(Flag::C);
                        let carry_bit = match carry {
                            true => 1,
                            false => 0,
                        };

                        let shifted_val = val << 1 | (carry_bit);
                        let z = shifted_val == 0;
                        let n = false;
                        let h = false;
                        let c = new_carry != 0;
                        bus.write(target_addr, shifted_val);
                        self.regs.update_flags(z, n, h, c);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Rl"),
                },
            },
            Instruction::RlA => match step {
                0 => {
                    // rotate left through the register
                    let r8 = self.regs.read8(Reg8::A);
                    let carry = self.regs.get_flag(Flag::C);
                    let carry_bit = match carry {
                        true => 1,
                        false => 0,
                    };

                    let new_carry = r8 & (1 << 7);
                    // discard the MSB, and insert the carry bit.
                    let shifted_r8 = ((r8) << 1) | (carry_bit);
                    let z = false;
                    let n = false;
                    let h = false;
                    let c = new_carry != 0;
                    self.regs.write8(Reg8::A, shifted_r8);
                    self.regs.update_flags(z, n, h, c);
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for RlA"),
            },
            Instruction::Rlc(operand8) => match operand8 {
                Operand8::Reg(reg8) => match step {
                    0 => {
                        // rotate reg8 left, but the carry itself isn't rotated
                        let r8 = self.regs.read8(reg8);
                        let msb = if r8 & (1 << 7) != 0 { 1 } else { 0 };

                        let shifted_r8 = (r8 << 1) | msb;

                        let z = shifted_r8 == 0;
                        let n = false;
                        let h = false;
                        let c = msb != 0;

                        self.regs.update_flags(z, n, h, c);
                        self.regs.write8(reg8, shifted_r8);
                        self.state = CpuState::FetchOpCode
                    }
                    _ => panic!("Invalid step for Rlc"),
                },
                Operand8::HlInd => match step {
                    0 => {
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = bus.read(target_addr);

                        self.temp_val = val as u16;
                        self.state = CpuState::Executing { instr, step: 1 };
                    }
                    1 => {
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = self.temp_val as u8;

                        let msb = if val & (1 << 7) != 0 { 1 } else { 0 };
                        let shifted_val = (val << 1) | msb;
                        let z = shifted_val == 0;
                        let n = false;
                        let h = false;
                        let c = msb != 0;

                        self.regs.update_flags(z, n, h, c);
                        bus.write(target_addr, shifted_val);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Rlc"),
                },
            },
            Instruction::RlcA => match step {
                0 => {
                    let A = self.regs.read8(Reg8::A);
                    let msb = if A & (1 << 7) != 0 { 1 } else { 0 };

                    let rotated_A = (A << 1) | msb;
                    let z = false;
                    let n = false;
                    let h = false;
                    let c = msb != 0;
                    self.regs.update_flags(z, n, h, c);
                    self.regs.write8(Reg8::A, rotated_A);
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for RlcA"),
            },
            Instruction::Rr(operand8) => match operand8 {
                Operand8::Reg(reg8) => match step {
                    0 => {
                        // right rotate through the carry
                        // carry stays on the LSB side for this
                        let r8 = self.regs.read8(reg8);
                        let carry = self.regs.get_flag(Flag::C);
                        let lsb = if (r8 & 1) == 1 { 1 } else { 0 };
                        let carry_bit = if carry { 1 } else { 0 };

                        let rotated_r8 = (r8 >> 1) | (carry_bit << 7);

                        let z = rotated_r8 == 0;
                        let n = false;
                        let h = false;
                        let c = lsb != 0;

                        self.regs.write8(reg8, rotated_r8);
                        self.regs.update_flags(z, n, h, c);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Rr"),
                },
                Operand8::HlInd => match step {
                    0 => {
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = bus.read(target_addr);
                        self.temp_val = val as u16;
                        self.state = CpuState::Executing { instr, step: 1 };
                    }
                    1 => {
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = self.temp_val as u8;
                        let carry = self.regs.get_flag(Flag::C);
                        let lsb = if (val & 1) == 1 { 1 } else { 0 };
                        let carry_bit = if carry { 1 } else { 0 };

                        let rotated_val = (val >> 1) | (carry_bit << 7);

                        let z = rotated_val == 0;
                        let n = false;
                        let h = false;
                        let c = lsb != 0;

                        bus.write(target_addr, rotated_val);
                        self.regs.update_flags(z, n, h, c);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Rr"),
                },
            },
            Instruction::RrA => match step {
                0 => {
                    let A = self.regs.read8(Reg8::A);
                    let carry = self.regs.get_flag(Flag::C);
                    let carry_bit = if carry { 1 } else { 0 };
                    let lsb = if (A & 1) == 1 { 1 } else { 0 };

                    let rotated_A = (A >> 1) | (carry_bit << 7);

                    let z = false;
                    let n = false;
                    let h = false;
                    let c = lsb != 0;

                    self.regs.write8(Reg8::A, rotated_A);
                    self.regs.update_flags(z, n, h, c);
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for RrA"),
            },
            Instruction::Rrc(operand8) => match operand8 {
                Operand8::Reg(reg8) => match step {
                    0 => {
                        // rotate the byte pushing into carry, but
                        // not rotating the carry along
                        let r8 = self.regs.read8(reg8);
                        let lsb = if (r8 & 1) == 1 { 1 } else { 0 };

                        // lsb becomes the new msb
                        let rotated_r8 = (r8 >> 1) | (lsb << 7);

                        let z = rotated_r8 == 0;
                        let n = false;
                        let h = false;
                        let c = lsb != 0;

                        self.regs.update_flags(z, n, h, c);
                        self.regs.write8(reg8, rotated_r8);
                        self.state = CpuState::FetchOpCode;
                    }

                    _ => panic!("Invalid step for Rrc"),
                },
                Operand8::HlInd => match step {
                    0 => {
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = bus.read(target_addr);
                        self.temp_val = val as u16;
                        self.state = CpuState::Executing { instr, step: 1 };
                    }
                    1 => {
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = self.temp_val as u8;
                        let lsb = if (val & 1) == 1 { 1 } else { 0 };

                        let rotated_val = (val >> 1) | (lsb << 7);

                        let z = rotated_val == 0;
                        let n = false;
                        let h = false;
                        let c = lsb != 0;

                        bus.write(target_addr, rotated_val);
                        self.regs.update_flags(z, n, h, c);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Rrc"),
                },
            },
            Instruction::RrcA => match step {
                0 => {
                    // right rotate register A, without including the carry in the
                    // rotation
                    let A = self.regs.read8(Reg8::A);
                    let lsb = if (A & 1) == 1 { 1 } else { 0 };

                    let rotated_A = (A >> 1) | (lsb << 7);

                    let z = false;
                    let n = false;
                    let h = false;
                    let c = lsb != 0;

                    self.regs.write8(Reg8::A, rotated_A);
                    self.regs.update_flags(z, n, h, c);
                    self.state = CpuState::FetchOpCode;
                }

                _ => panic!("Invalid step for RrcA"),
            },
            Instruction::Sla(operand8) => match operand8 {
                Operand8::Reg(reg8) => match step {
                    0 => {
                        // left shift reg8
                        let r8 = self.regs.read8(reg8);

                        // msb becomes the new carry
                        let msb = if r8 & (1 << 7) != 0 { 1 } else { 0 };

                        let shifted_r8 = r8 << 1;

                        let z = shifted_r8 == 0;
                        let n = false;
                        let h = false;
                        let c = msb != 0;

                        self.regs.write8(reg8, shifted_r8);
                        self.regs.update_flags(z, n, h, c);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Sla"),
                },
                Operand8::HlInd => match step {
                    0 => {
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = bus.read(target_addr);
                        self.temp_val = val as u16;
                        self.state = CpuState::Executing { instr, step: 1 };
                    }
                    1 => {
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = self.temp_val as u8;

                        let msb = if val & (1 << 7) != 0 { 1 } else { 0 };

                        let shifted_val = val << 1;

                        let z = shifted_val == 0;
                        let n = false;
                        let h = false;
                        let c = msb != 0;

                        bus.write(target_addr, shifted_val);
                        self.regs.update_flags(z, n, h, c);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Sla"),
                },
            },
            Instruction::Sra(operand8) => match operand8 {
                Operand8::Reg(reg8) => match step {
                    0 => {
                        // bit 7, the MSB must remain unchanged after the right shift
                        let r8 = self.regs.read8(reg8);
                        let lsb = if (r8 & 1) != 0 { 1 } else { 0 };
                        let msb = r8 & (1 << 7);

                        let shifted_r8 = (r8 >> 1) | msb;

                        let z = shifted_r8 == 0;
                        let n = false;
                        let h = false;
                        let c = lsb != 0;

                        self.regs.write8(reg8, shifted_r8);
                        self.regs.update_flags(z, n, h, c);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Sra"),
                },
                Operand8::HlInd => match step {
                    0 => {
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = bus.read(target_addr);
                        self.temp_val = val as u16;
                        self.state = CpuState::Executing { instr, step: 1 };
                    }
                    1 => {
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = self.temp_val as u8;
                        let lsb = if (val & 1) != 0 { 1 } else { 0 };
                        let msb = val & (1 << 7);

                        let shifted_val = (val >> 1) | msb;

                        let z = shifted_val == 0;
                        let n = false;
                        let h = false;
                        let c = lsb != 0;

                        bus.write(target_addr, shifted_val);
                        self.regs.update_flags(z, n, h, c);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Sra"),
                },
            },
            Instruction::Srl(operand8) => match operand8 {
                Operand8::Reg(reg8) => match step {
                    0 => {
                        let r8 = self.regs.read8(reg8);
                        let lsb = if (r8 & 1) != 0 { 1 } else { 0 };

                        let shifted_r8 = r8 >> 1;

                        let z = shifted_r8 == 0;
                        let n = false;
                        let h = false;
                        let c = lsb != 0;

                        self.regs.update_flags(z, n, h, c);
                        self.regs.write8(reg8, shifted_r8);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Srl"),
                },
                Operand8::HlInd => match step {
                    0 => {
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = bus.read(target_addr);
                        self.temp_val = val as u16;
                        self.state = CpuState::Executing { instr, step: 1 };
                    }
                    1 => {
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = self.temp_val as u8;

                        let lsb = if (val & 1) != 0 { 1 } else { 0 };

                        let shifted_val = val >> 1;

                        let z = shifted_val == 0;
                        let n = false;
                        let h = false;
                        let c = lsb != 0;

                        self.regs.update_flags(z, n, h, c);
                        bus.write(target_addr, shifted_val);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Srl"),
                },
            },
            Instruction::Swap(operand8) => match operand8 {
                Operand8::Reg(reg8) => match step {
                    0 => {
                        // swap the damn upper and lower bits of a byte
                        let r8 = self.regs.read8(reg8);

                        let swapped_r8 = (r8 >> 4) | (r8 << 4);

                        let z = swapped_r8 == 0;
                        let n = false;
                        let h = false;
                        let c = false;

                        self.regs.write8(reg8, swapped_r8);
                        self.regs.update_flags(z, n, h, c);
                        self.state = CpuState::FetchOpCode;
                    }
                    _ => panic!("Invalid step for Swap"),
                },
                Operand8::HlInd => match step {
                    0 => {
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = bus.read(target_addr);
                        self.temp_val = val as u16;
                        self.state = CpuState::Executing { instr, step: 1 };
                    }
                    1 => {
                        let target_addr = self.regs.read16(Reg16::HL);
                        let val = self.temp_val as u8;

                        let swapped_val = (val >> 4) | (val << 4);

                        let z = swapped_val == 0;
                        let n = false;
                        let h = false;
                        let c = false;

                        bus.write(target_addr, swapped_val);
                        self.regs.update_flags(z, n, h, c);
                        self.state = CpuState::FetchOpCode;
                    }

                    _ => panic!("Invalid step for Swap"),
                },
            },
            Instruction::Call(cond) => match step {
                0 => {
                    // first get the damn address n16
                    let low_bytes = self.fetch_advance_pc(bus);
                    self.temp_val = low_bytes as u16;
                    self.state = CpuState::Executing { instr, step: 1 };
                }
                1 => {
                    // get the high bytes, and the full address
                    let low_bytes = self.temp_val as u16;
                    let high_bytes = self.fetch_advance_pc(bus) as u16;
                    let full_addr = high_bytes << 8 | low_bytes;

                    if self.check_cond(cond) {
                        self.temp_val = full_addr;
                        self.state = CpuState::Executing { instr, step: 2 };
                    } else {
                        self.state = CpuState::FetchOpCode;
                    }
                }
                2 => {
                    // decrement SP
                    let sp = self.regs.read16(Reg16::SP);
                    let decremented_sp = sp.wrapping_sub(1);
                    self.regs.write16(Reg16::SP, decremented_sp);
                    self.state = CpuState::Executing { instr, step: 3 };
                }
                3 => {
                    //get the return address
                    //write high byte into the addr pointed to by SP
                    //decrement SP again
                    let return_addr = self.regs.read16(Reg16::PC);
                    let sp = self.regs.read16(Reg16::SP);
                    let decremented_sp = sp.wrapping_sub(1);
                    self.regs.write16(Reg16::SP, decremented_sp);

                    let high_byte = ((return_addr & 0xFF00) >> 8) as u8;
                    bus.write(sp, high_byte);
                    self.state = CpuState::Executing { instr, step: 4 };
                }
                4 => {
                    let return_addr = self.regs.read16(Reg16::PC);
                    let sp = self.regs.read16(Reg16::SP);
                    let low_byte = (return_addr & 0x00FF) as u8;
                    bus.write(sp, low_byte);

                    // set the pc to the actual addr retrieved initially
                    let call_addr = self.temp_val;
                    self.regs.write16(Reg16::PC, call_addr);
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for Call"),
            },
            Instruction::JpHl => match step {
                0 => {
                    // copy the addr from HL into PC
                    let HL = self.regs.read16(Reg16::HL);
                    self.regs.write16(Reg16::PC, HL);
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for JpHl"),
            },
            Instruction::Jp(cond) => match step {
                0 => {
                    // get the target jump addr
                    let low_byte = self.fetch_advance_pc(bus);
                    self.temp_val = low_byte as u16;
                    self.state = CpuState::Executing { instr, step: 1 };
                }
                1 => {
                    let low_byte = self.temp_val as u16;
                    let high_byte = self.fetch_advance_pc(bus) as u16;
                    let jump_addr = (low_byte & 0x00FF) | high_byte << 8;
                    if self.check_cond(cond) {
                        self.regs.write16(Reg16::PC, jump_addr);

                        self.state = CpuState::Executing { instr, step: 2 };
                    } else {
                        self.state = CpuState::FetchOpCode;
                    }
                }
                2 => {
                    // PURELY to burn cpu cycle according to the specs
                    // it's meant to consume 4 cycles, when a jump occurs
                    // only 3 when the cond fails the jump, 1 jump is implicit in
                    // reaching execute_step func, so 3 steps must happen here.
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for Jp"),
            },
            Instruction::Jr(cond) => match step {
                0 => {
                    // Jump relative
                    // this takes fewer cycles than a pure JP instruction
                    // those programmers in the 80s upto 2000s were actual gods

                    // read one byte (the offset)
                    let offset = self.fetch_advance_pc(bus);
                    let offset_signed = offset as i8;
                    if self.check_cond(cond) {
                        let new_pc = self
                            .regs
                            .read16(Reg16::PC)
                            .wrapping_add_signed(offset_signed as i16);

                        self.regs.write16(Reg16::PC, new_pc);
                        self.state = CpuState::Executing { instr, step: 1 };
                    } else {
                        self.state = CpuState::FetchOpCode;
                    }
                }
                1 => {
                    // Burning a cycle, because the specs say so.
                    self.state = CpuState::FetchOpCode;
                }
                _ => panic!("Invalid step for Jr"),
            },
            Instruction::Ret(cond) => match cond {
                Cond::NotZero | Cond::Zero | Cond::NotCarry | Cond::Carry => {
                    // The conditional variant
                    match step {
                        0 => {
                            if self.check_cond(cond) {
                                self.state = CpuState::Executing { instr, step: 1 };
                            } else {
                                self.state = CpuState::FetchOpCode;
                            }
                        }
                        1 => {
                            // pop the data off the stack
                            let sp = self.regs.read16(Reg16::SP);
                            let incremented_sp = sp.wrapping_add(1);

                            // read the data pointed to by the stack [sp]
                            let low_byte = bus.read(sp);
                            self.temp_val = low_byte as u16;
                            self.regs.write16(Reg16::SP, incremented_sp);
                            self.state = CpuState::Executing { instr, step: 2 };
                        }
                        2 => {
                            let sp = self.regs.read16(Reg16::SP);
                            let incremented_sp = sp.wrapping_add(1);

                            let low_byte = self.temp_val;
                            let high_byte = bus.read(sp) as u16;
                            let full_addr = high_byte << 8 | low_byte;

                            self.temp_val = full_addr;
                            self.regs.write16(Reg16::SP, incremented_sp);
                            self.state = CpuState::Executing { instr, step: 3 };
                        }
                        3 => {
                            let full_addr = self.temp_val;

                            self.regs.write16(Reg16::PC, full_addr);
                            self.state = CpuState::FetchOpCode;
                        }
                        _ => panic!("Invalid Step for Ret (conditional)"),
                    }
                }
                Cond::Always => {
                    // unconditional variant
                    // (consumes 4 cycles, 3 mapped here 1 implicit)
                    match step {
                        0 => {
                            // pop the data off the stack
                            let sp = self.regs.read16(Reg16::SP);
                            let incremented_sp = sp.wrapping_add(1);

                            // read the data pointed to by the stack [sp]
                            let low_byte = bus.read(sp);
                            self.temp_val = low_byte as u16;
                            self.regs.write16(Reg16::SP, incremented_sp);
                            self.state = CpuState::Executing { instr, step: 1 };
                        }
                        1 => {
                            let sp = self.regs.read16(Reg16::SP);
                            let incremented_sp = sp.wrapping_add(1);

                            let low_byte = self.temp_val;
                            let high_byte = bus.read(sp) as u16;
                            let full_addr = high_byte << 8 | low_byte;

                            self.temp_val = full_addr;
                            self.regs.write16(Reg16::SP, incremented_sp);
                            self.state = CpuState::Executing { instr, step: 2 };
                        }
                        2 => {
                            let full_addr = self.temp_val;

                            self.regs.write16(Reg16::PC, full_addr);
                            self.state = CpuState::FetchOpCode;
                        }
                        _ => panic!("Invalid Step for Ret (unconditional)"),
                    }
                }
            },
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

            // This is meant to alter the state of the tick in someway
            Instruction::PrefixCb => todo!(),
        }
    }

    fn tick(&mut self, bus: &mut bus::Bus) {
        // THIS function is meant to advance the cpu by EXACTLY one M cycle
        // I picked a cycle accurate (kinda) emulation, and it's taking its
        // toll on me :sob:

        match self.state {
            CpuState::FetchOpCode => {
                let opcode = self.fetch_advance_pc(bus);
                let instr = Instruction::from_byte(opcode);

                if matches!(instr, Instruction::PrefixCb) {
                    // the instruction fetched needs special handling
                    // it's the extended mode
                    self.state = CpuState::FetchCbOpCode;
                } else if instr.is_single_cycle() {
                    // if the instruction is single cycle
                    // like moving data from one cpu reg to another reg
                    // it can be done "instantly", doesn't need the bus
                    // or any other tertiary component. So, execute that here
                    // without needing ANOTHER tick to switch to execution state.
                    self.execute_step(instr, 0, bus);

                    // kinda unnecessary to set the state to this,
                    // since it never left that state, but for the sake of
                    // mathematical completeness, i must.
                    self.state = CpuState::FetchOpCode;
                } else {
                    // this is basically signaling a transition,
                    // let the next tick trigger the actual
                    // thing the instruction is meant to do
                    self.state = CpuState::Executing { instr, step: 0 };
                }
            }
            CpuState::FetchCbOpCode => {
                // need to do 2 fetches for a cb instr. This is the second
                // one, to really map it to a unique instruction
                let cb_opcode = self.fetch_advance_pc(bus);
                let instr = Instruction::from_cb_byte(cb_opcode);

                if let Instruction::Bit(_, Operand8::HlInd)
                | Instruction::Res(_, Operand8::HlInd)
                | Instruction::Set(_, Operand8::HlInd)
                | Instruction::Rlc(Operand8::HlInd)
                | Instruction::Rrc(Operand8::HlInd)
                | Instruction::Rl(Operand8::HlInd)
                | Instruction::Rr(Operand8::HlInd)
                | Instruction::Sla(Operand8::HlInd)
                | Instruction::Sra(Operand8::HlInd)
                | Instruction::Srl(Operand8::HlInd)
                | Instruction::Swap(Operand8::HlInd) = instr
                {
                    self.state = CpuState::Executing { instr, step: 0 };
                } else {
                    self.execute_step(instr, 0, bus);
                    self.state = CpuState::FetchOpCode;
                }
            }
            CpuState::Executing { instr, step } => self.execute_step(instr, step, bus),
            CpuState::Halted => {
                // DO NOTHING, ofc
                // Until an interrupt does some work
            }
        }
    }

    fn decode(&mut self, opcode: u8, bus: &bus::Bus) {}
}
