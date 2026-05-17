use serde::Deserialize;
use std::collections::BTreeMap;
use std::env;
use std::fmt::format;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Debug)]
struct OperandDef {
    name: String,
    immediate: bool,
}

#[derive(Deserialize, Debug)]
struct OpcodeDef {
    mnemonic: String,
    operands: Vec<OperandDef>,
}

#[derive(Deserialize, Debug)]
struct OpcodesJson {
    unprefixed: BTreeMap<String, OpcodeDef>,
    cbprefixed: BTreeMap<String, OpcodeDef>,
}

fn parse_reg8(name: &str) -> Option<&'static str> {
    match name {
        "A" => Some("Reg8::A"),
        "B" => Some("Reg8::B"),
        "C" => Some("Reg8::C"),
        "D" => Some("Reg8::D"),
        "E" => Some("Reg8::E"),
        "H" => Some("Reg8::H"),
        "L" => Some("Reg8::L"),
        _ => None,
    }
}

fn parse_reg16(name: &str) -> Option<&'static str> {
    match name {
        "AF" => Some("Reg16::AF"),
        "BC" => Some("Reg16::BC"),
        "DE" => Some("Reg16::DE"),
        "HL" => Some("Reg16::HL"),
        "SP" => Some("Reg16::SP"),
        _ => None,
    }
}

fn parse_cond(name: &str) -> Option<&'static str> {
    match name {
        "NZ" => Some("Cond::NotZero"),
        "Z" => Some("Cond::Zero"),
        "NC" => Some("Cond::NotCarry"),
        "C" => Some("Cond::Carry"),
        _ => None,
    }
}

fn parse_operand8(name: &str) -> Option<String> {
    if let Some(reg) = parse_reg8(name) {
        return Some(format!("Operand8::Reg({})", reg));
    }
    if name == "HL" && !name.contains("n8") {
        return Some("Operand8::HlInd".to_string());
    }
    None
}

fn map_cb_instruction(def: &OpcodeDef) -> String {
    let op =
        parse_operand8(&def.operands.last().unwrap().name).expect("Failed to parse CB operand");

    match def.mnemonic.as_str() {
        "RLC" => format!("Instruction::Rlc({})", op),
        "RRC" => format!("Instruction::Rrc({})", op),
        "RL" => format!("Instruction::Rl({})", op),
        "RR" => format!("Instruction::Rr({})", op),
        "SLA" => format!("Instruction::Sla({})", op),
        "SRA" => format!("Instruction::Sra({})", op),
        "SWAP" => format!("Instruction::Swap({})", op),
        "SRL" => format!("Instruction::Srl({})", op),
        "BIT" => format!("Instruction::Bit({}, {})", def.operands[0].name, op),
        "RES" => format!("Instruction::Res({}, {})", def.operands[0].name, op),
        "SET" => format!("Instruction::Set({}, {})", def.operands[0].name, op),
        _ => format!("TODO: Unmapped CB Instruction: {}", def.mnemonic),
    }
}

fn map_standard_instruction(def: &OpcodeDef) -> String {
    match def.mnemonic.as_str() {
        "NOP" => "Instruction::Nop".to_string(),
        "PREFIX" => "Instruction::PrefixCb".to_string(),
        "DI" => "Instruction::Di".to_string(),
        "EI" => "Instruction::Ei".to_string(),
        "HALT" => "Instruction::Halt".to_string(),
        "STOP" => "Instruction::Stop".to_string(),
        "DAA" => "Instruction::Daa".to_string(),
        "CPL" => "Instruction::Cpl".to_string(),
        "SCF" => "Instruction::Scf".to_string(),
        "CCF" => "Instruction::Ccf".to_string(),
        "RETI" => "Instruction::Reti".to_string(),

        // NON cb rotates
        "RLCA" => "Instruction::RlcA".to_string(),
        "RRCA" => "Instruction::RrcA".to_string(),
        "RLA" => "Instruction::RlA".to_string(),
        "RRA" => "Instruction::RrA".to_string(),

        // ILLEGAL op codes that are not present in the game boy cpu
        m if m.starts_with("ILLEGAL") => "Instruction::Stop".to_string(),

        "RST" => {
            let vec_str = def.operands[0]
                .name
                .replace("H", "")
                .replace("$", "")
                .replace("0x", "");

            let vec_val = u8::from_str_radix(&vec_str, 16).unwrap_or(0);
            format!("Instruction::Rst({:#04X})", vec_val)
        }

        "PUSH" | "POP" => {
            let variant = if def.mnemonic == "PUSH" {
                "Push"
            } else {
                "Pop"
            };
            if let Some(r16) = parse_reg16(&def.operands[0].name) {
                format!("Instruction::{}({})", variant, r16)
            } else {
                format!("TODO: Unmapped {} pattern", def.mnemonic)
            }
        }

        "ADD" | "ADC" | "SUB" | "SBC" | "AND" | "XOR" | "OR" | "CP" => {
            let variant_name = match def.mnemonic.as_str() {
                "ADD" => "Add",
                "ADC" => "Adc",
                "SUB" => "Sub",
                "SBC" => "Sbc",
                "AND" => "And",
                "XOR" => "Xor",
                "OR" => "Or",
                "CP" => "Cp",
                _ => unreachable!(),
            };

            if def.mnemonic == "ADD" && def.operands[0].name == "HL" {
                return format!(
                    "Instruction::Add16({})",
                    parse_reg16(&def.operands[1].name).unwrap()
                );
            }

            if def.mnemonic == "ADD" && def.operands[0].name == "SP" {
                return "Instruction::AddSpImm".to_string();
            }

            if def.operands.len() > 1 && def.operands[1].name == "n8" {
                format!("Instruction::{}Imm", variant_name)
            } else {
                let target = if def.operands.len() == 2 {
                    &def.operands[1].name
                } else {
                    &def.operands[0].name
                };
                match parse_operand8(target) {
                    Some(op) => format!("Instruction::{}({})", variant_name, op),
                    None => format!(
                        "TODO: Unmapped inst 16bit ALU pattern : {} {}",
                        def.mnemonic, target
                    ),
                }
            }
        }
        "LD" | "LDH" => {
            let op1 = &def.operands[0];
            let op2 = &def.operands[1];

            if op1.name == "a16" && !op1.immediate && op2.name == "SP" {
                return "Instruction::LdImm16Sp".to_string(); // LD [a16], SP
            }

            // 16-bit Immediates & SP Memory Load
            if op2.name == "n16" || op2.name == "a16" || op2.name == "d16" {
                if let Some(r16) = parse_reg16(&op1.name) {
                    return format!("Instruction::Ld16Imm({})", r16); // LD r16, n16
                }
            }

            // 8-bit Immediates
            if op2.name == "n8" || op2.name == "d8" {
                if op1.name == "HL" && !op1.immediate {
                    return "Instruction::LdHlIndImm".to_string(); // LD [HL], n8
                }
                if let Some(r1) = parse_reg8(&op1.name) {
                    return format!("Instruction::LdImm({})", r1); // LD r8, n8
                }
            }

            //  8 bit regs to other 8 bit regs
            if let (Some(r1), Some(r2)) = (parse_reg8(&op1.name), parse_reg8(&op2.name)) {
                return format!("Instruction::Ld({}, {})", r1, r2); // LD r8, r8
            }

            // standard [HL] indirection
            if op1.name == "HL" && !op1.immediate {
                if let Some(r2) = parse_reg8(&op2.name) {
                    return format!("Instruction::LdHlInd({})", r2);
                }
            }
            if op2.name == "HL" && !op2.immediate {
                if let Some(r1) = parse_reg8(&op1.name) {
                    return format!("Instruction::LdRegHlInd({})", r1);
                }
            }

            // HLI / HLD specific (JSON HANDLES THESE NAMES DIFFERENTLY SOMETIMES)
            if !op1.immediate && (op1.name == "HL+" || op1.name == "hli") {
                return "Instruction::LdHliA".to_string();
            }
            if !op1.immediate && (op1.name == "HL-" || op1.name == "hld") {
                return "Instruction::LdHldA".to_string();
            }
            if !op2.immediate && (op2.name == "HL+" || op2.name == "hli") {
                return "Instruction::LdAhli".to_string();
            }
            if !op2.immediate && (op2.name == "HL-" || op2.name == "hld") {
                return "Instruction::LdAHld".to_string();
            }

            // 16-bit Indirection specific to Accumulator (A)
            if op1.name == "A" && !op2.immediate {
                if let Some(r16) = parse_reg16(&op2.name) {
                    return format!("Instruction::LdAReg16Ind({})", r16);
                }
                if op2.name == "a16" {
                    return "Instruction::LdAImm16Ind".to_string();
                }
                if op2.name == "C" {
                    return "Instruction::LdhACInd".to_string();
                }
                if op2.name == "a8" || def.mnemonic == "LDH" {
                    return "Instruction::LdhAImm8Ind".to_string();
                }
            }
            if op2.name == "A" && !op1.immediate {
                if let Some(r16) = parse_reg16(&op1.name) {
                    return format!("Instruction::LdReg16IndA({})", r16);
                }
                if op1.name == "a16" {
                    return "Instruction::LdImmIndA".to_string();
                }
                if op1.name == "C" {
                    return "Instruction::LdhCIndA".to_string();
                }
                if op1.name == "a8" || def.mnemonic == "LDH" {
                    return "Instruction::LdhImm8IndA".to_string();
                }
            }

            // SP and HL edge cases
            if op1.name == "SP" && op2.name == "HL" {
                return "Instruction::LdSpHl".to_string();
            }
            if op1.name == "HL" && op2.name.starts_with("SP") {
                return "Instruction::LdHlSpImm".to_string();
            }

            format!("TODO: Unmapped LD pattern: {} -> {}", op1.name, op2.name)
        }

        "INC" | "DEC" => {
            let variant = if def.mnemonic == "INC" { "Inc" } else { "Dec" };
            let target = &def.operands[0].name;

            if let Some(r16) = parse_reg16(target) {
                format!("Instruction::{}16({})", variant, r16)
            } else if let Some(op8) = parse_operand8(target) {
                format!("Instruction::{}({})", variant, op8)
            } else {
                format!("TODO: Unmapped {} pattern: {}", def.mnemonic, target)
            }
        }

        "JP" | "JR" | "CALL" | "RET" => {
            let variant = match def.mnemonic.as_str() {
                "JP" => "Jp",
                "JR" => "Jr",
                "CALL" => "Call",
                "RET" => "Ret",
                _ => unreachable!(),
            };

            if def.operands.is_empty() || (def.mnemonic == "RET" && def.operands.is_empty()) {
                // Unconditional return
                format!("Instruction::{}(Cond::Always)", variant)
            } else if let Some(cond) = parse_cond(&def.operands[0].name) {
                // for conditional ones
                format!("Instruction::{}({})", variant, cond)
            } else {
                if def.mnemonic == "JP"
                    && def.operands[0].name == "HL"
                    && !def.operands[0].immediate
                {
                    "Instruction::JpHl".to_string()
                } else {
                    format!("Instruction::{}(Cond::Always)", variant)
                }
            }
        }

        _ => format!("TODO: Unmapped Instruction: {}", def.mnemonic),
    }
}

fn generate_table(table: &BTreeMap<String, OpcodeDef>, dest: std::path::PathBuf, is_cb: bool) {
    let mut code = String::new();
    code.push_str("match opcode {\n");

    for (hex_str, def) in table {
        let rust_variant = if is_cb {
            map_cb_instruction(def)
        } else {
            map_standard_instruction(def)
        };

        if rust_variant.starts_with("TODO") {
            code.push_str(&format!(
                "    // {} => {}, // {}\n",
                hex_str, rust_variant, def.mnemonic
            ));
        } else {
            code.push_str(&format!("   {} => {}, \n", hex_str, rust_variant));
        }
    }

    code.push_str("     _ => panic!(\"Unknown Opcode: {:#04X}\", opcode),\n");
    code.push_str("}\n");

    fs::write(&dest, code).unwrap();
}

fn main() {
    // No idea how this black magic works, but apparently mere printing
    // commands works to create a watcher
    println!("cargo:rerun-if-changed=data/Opcodes.json");
    let json_data = fs::read_to_string("data/Opcodes.json").expect("Failed to read Opcodes.json!");

    let opcodes: OpcodesJson = serde_json::from_str(&json_data).expect("Failed to parse json");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    generate_table(
        &opcodes.unprefixed,
        Path::new(&out_dir).join("decode_unprefixed.rs"),
        false,
    );

    generate_table(
        &opcodes.cbprefixed,
        Path::new(&out_dir).join("decode_cbprefixed.rs"),
        true,
    );
}
