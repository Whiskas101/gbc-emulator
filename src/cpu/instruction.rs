use super::registers::{Reg8, Reg16};

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

    // The prefix for 0xCB
    PrefixCb,
}

impl Instruction {
    pub fn from_byte(opcode: u8) -> Self {
        include!(concat!(env!("OUT_DIR"), "/decode_unprefixed.rs"))
    }
    pub fn from_cb_byte(opcode: u8) -> Self {
        include!(concat!(env!("OUT_DIR"), "/decode_cbprefixed.rs"))
    }

    pub fn is_single_cycle(&self) -> bool {
        todo!()
    }
}
