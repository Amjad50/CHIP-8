pub struct Instruction {
    pub bytes: u16,
    pub address: u16,
    pub opcode: String,
}

fn generate_instruction_string(instruction: u16) -> String {
    const INVALID_INSTRUCTION: &str = "??";
    // nibbles will have the values of the instruction as
    // each four bytes of the instruction starting from the left as 0
    //
    // example: instruction = 0xfa12, nibbles = [0xf, 0xa, 0x1, 0x2]
    let mut nibbles = [0u8; 4];
    for i in 0..4 {
        let offset = (3 - i) * 4;
        nibbles[i] = ((instruction & (0xf << offset)) >> offset) as u8;
    }

    // the lowest 12-bit value
    let address = (nibbles[1] as u16) << 8 | (nibbles[2] as u16) << 4 | (nibbles[3] as u16);
    // lowest 4-bit value in the high byte
    let x = nibbles[1];
    // highest 4-bit value in the low byte
    let y = nibbles[2];
    // the lowest 8-bit value
    let kk = nibbles[2] << 4 | nibbles[3];

    match nibbles[0] {
        0 => {
            // CLS, RET, SYS
            match nibbles[2] << 4 | nibbles[3] {
                0xE0 => {
                    // CLS
                    "CLS".to_string()
                }
                0xEE => {
                    // RET
                    "RET".to_string()
                }
                _ => {
                    // SYS addr
                    format!("SYS 0x{:03X}", address)
                }
            }
        }
        1 => {
            // JMP addr
            format!("JMP 0x{:03X}", address)
        }
        2 => {
            // CALL addr
            format!("CALL 0x{:03X}", address)
        }
        3 => {
            // SE Vx, byte
            format!("SE V{:01X}, 0x{:02X}", x, kk)
        }
        4 => {
            // SNE Vx, byte
            format!("SNE V{:01X}, 0x{:02X}", x, kk)
        }
        5 => {
            // SE Vx, Vy
            format!("SE V{:01X}, V{:01X}", x, y)
        }
        6 => {
            // LD Vx, byte
            format!("LD V{:01X}, 0x{:02X}", x, kk)
        }
        7 => {
            // ADD Vx, byte
            format!("ADD V{:01X}, 0x{:02X}", x, kk)
        }
        8 => {
            // LD, OR, AND, XOR, ADD, SUB, SHR, SUBN, SHL
            match nibbles[3] {
                0 => {
                    // LD Vx, Vy
                    format!("LD V{:01X}, V{:01X}", x, y)
                }
                1 => {
                    // OR Vx, Vy
                    format!("OR V{:01X}, V{:01X}", x, y)
                }
                2 => {
                    // AND Vx, Vy
                    format!("AND V{:01X}, V{:01X}", x, y)
                }
                3 => {
                    // XOR Vx, Vy
                    format!("XOR V{:01X}, V{:01X}", x, y)
                }
                4 => {
                    // ADD Vx, Vy
                    format!("ADD V{:01X}, V{:01X}", x, y)
                }
                5 => {
                    // SUB Vx, Vy
                    format!("SUB V{:01X}, V{:01X}", x, y)
                }
                6 => {
                    // SHR Vx {, Vy}
                    format!("SHR V{:01X}", x)
                }
                7 => {
                    // SUBN Vx, Vy
                    format!("SUBN V{:01X}, V{:01X}", x, y)
                }
                0xE => {
                    // SHL Vx {, Vy}
                    format!("SHL V{:01X}", x)
                }
                _ => {
                    // invalid instruction
                    INVALID_INSTRUCTION.to_string()
                }
            }
        }
        9 => {
            // SNE Vx, Vy
            format!("SNE V{:01X}, V{:01X}", x, y)
        }
        0xA => {
            // LD I, addr
            format!("LD I, 0x{:03X}", address)
        }
        0xB => {
            // JP V0, addr
            format!("JP V0, 0x{:03X}", address)
        }
        0xC => {
            // RND Vx, byte
            format!("RND V{:01X}, 0x{:02X}", x, kk)
        }
        0xD => {
            // DRW Vx, Vy, nibble
            format!("DRW V{:01X}, V{:01X}, 0x{:01x}", x, y, nibbles[3])
        }
        0xE => {
            // SKP, SKNP
            match nibbles[2] << 4 | nibbles[3] {
                0x9E => {
                    // SKP Vx
                    format!("SKP V{:01X}", x)
                }
                0xA1 => {
                    // SKNP Vx
                    format!("SKNP V{:01X}", x)
                }
                _ => INVALID_INSTRUCTION.to_string(),
            }
        }
        0xF => {
            // LD, ADD
            match nibbles[2] << 4 | nibbles[3] {
                0x07 => {
                    // LD Vx, DT
                    format!("LD V{:01X}, DT", x)
                }
                0x0A => {
                    // LD Vx, K
                    format!("LD V{:01X}, K", x)
                }
                0x15 => {
                    // LD DT, Vx
                    format!("LD DT, V{:01X}", x)
                }
                0x18 => {
                    // LD ST, Vx
                    format!("LD ST, V{:01X}", x)
                }
                0x1E => {
                    // ADD I, Vx
                    format!("ADD I, V{:01X}", x)
                }
                0x29 => {
                    // LD F, Vx
                    format!("LD F, V{:01X}", x)
                }
                0x33 => {
                    // LD B, Vx
                    format!("LD B, V{:01X}", x)
                }
                0x55 => {
                    // LD [I], Vx
                    format!("LD [I], V{:01X}", x)
                }
                0x65 => {
                    // LD Vx, [I]
                    format!("LD V{:01X}, [I]", x)
                }
                _ => {
                    // invalid instruction
                    INVALID_INSTRUCTION.to_string()
                }
            }
        }
        _ => INVALID_INSTRUCTION.to_string(),
    }
}

pub fn disassemble(instructions: &[u8], offset: u16) -> Vec<Instruction> {
    let mut result = Vec::<Instruction>::with_capacity(instructions.len() / 2);

    for i in (0..instructions.len()).step_by(2) {
        let instruction = ((instructions[i] as u16) << 8) | (instructions[i + 1] as u16);

        result.push(Instruction {
            address: i as u16 + offset,
            bytes: instruction,
            opcode: generate_instruction_string(instruction),
        });
    }

    result
}
