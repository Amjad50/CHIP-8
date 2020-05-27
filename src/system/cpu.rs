use super::display::Display;
use super::memory::Memory;
use rand;
use std::fs::File; // used for the RND instruction only.

pub struct CPU {
    V: [u8; 16],      // 16 8-bit Vx register
    I: u16,           // I register
    DT: u8,           // Delay timer
    ST: u8,           // Sound timer
    PC: u16,          // Program counter
    SP: u8,           // Stack pointer
    stack: [u16; 16], // Internal stack of 16 16-bit values
    memory: Memory,   // Memory component
    display: Display, // Display component
}

impl CPU {
    pub fn new() -> CPU {
        return CPU {
            V: [0u8; 16],
            I: 0,
            DT: 0,
            ST: 0,
            PC: 0,
            SP: 0,
            stack: [0; 16],
            memory: Memory::new(),
            display: Display::new(64, 32),
        };
    }

    // FiXME: right now, this must run at the end, because ownership is moved
    pub fn run_display_application(self) {
        self.display.run();
    }

    pub fn read_file(&mut self, file: &mut File) {
        self.memory.read_file(file);
    }

    pub fn run_instruction(&mut self, instruction: u16) {
        // nipples will have the values of the instruction as
        // each four bytes of the instruction starting from the left as 0
        //
        // example: instruction = 0xfa12, nipples = [0xf, 0xa, 0x1, 0x2]
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
                        for i in 0..self.display.get_height() {
                            for j in 0..self.display.get_width() {
                                self.display.draw_pixel(j, i, false);
                            }
                        }
                        self.display.redraw();
                    }
                    0xEE => {
                        // RET
                        self.SP -= 1;
                        let return_address = self.stack[self.SP as usize];
                        self.PC = return_address;
                    }
                    _ => {
                        // SYS addr
                        self.PC = address;
                    }
                }
            }
            1 => {
                // JMP addr
                self.PC = address;
            }
            2 => {
                // CALL addr
                self.stack[self.ST as usize] = self.PC;
                self.ST += 1;
                self.PC = address;
            }
            3 => {
                // SE Vx, byte
                if self.V[x as usize] == kk {
                    self.PC += 2;
                }
            }
            4 => {
                // SNE Vx, byte
                if self.V[x as usize] != kk {
                    self.PC += 2;
                }
            }
            5 => {
                // SE Vx, Vy
                if nibbles[3] == 0 {
                    if self.V[x as usize] == self.V[y as usize] {
                        self.PC += 2;
                    }
                }
            }
            6 => {
                // LD Vx, byte
                self.V[x as usize] = kk;
            }
            7 => {
                // ADD Vx, byte
                self.V[x as usize] += kk;
            }
            8 => {
                // LD, OR, AND, XOR, ADD, SUB, SHR, SUBN, SHL
                match nibbles[3] {
                    0 => {
                        // LD Vx, Vy
                        self.V[x as usize] = self.V[y as usize];
                    }
                    1 => {
                        // OR Vx, Vy
                        self.V[x as usize] |= self.V[y as usize];
                    }
                    2 => {
                        // AND Vx, Vy
                        self.V[x as usize] &= self.V[y as usize];
                    }
                    3 => {
                        // XOR Vx, Vy
                        self.V[x as usize] ^= self.V[y as usize];
                    }
                    4 => {
                        // ADD Vx, Vy
                        let result = (self.V[x as usize] as u16) + (self.V[y as usize] as u16);
                        // store the result in Vx
                        self.V[x as usize] = result as u8;
                        // store the carry in Vf
                        self.V[0xF] = if result > 255 { 1 } else { 0 };
                    }
                    5 => {
                        // SUB Vx, Vy
                        // store NOT BORROW
                        self.V[0xF] = if self.V[x as usize] > self.V[y as usize] {
                            1
                        } else {
                            0
                        };
                        self.V[x as usize] -= self.V[y as usize];
                    }
                    6 => {
                        // SHR Vx {, Vy}
                        // TODO: not sure if this can have empty y value and what does that mean
                        self.V[0xF] = if self.V[x as usize] & 1 != 0 { 1 } else { 0 };
                        self.V[x as usize] >>= self.V[y as usize];
                    }
                    7 => {
                        // SUBN Vx, Vy
                        // store NOT BORROW
                        self.V[0xF] = if self.V[y as usize] > self.V[x as usize] {
                            1
                        } else {
                            0
                        };
                        self.V[x as usize] = self.V[y as usize] - self.V[x as usize];
                    }
                    0xE => {
                        // SHL Vx {, Vy}
                        // TODO: not sure if this can have empty y value and what does that mean
                        self.V[0xF] = if self.V[x as usize] & 0x80 != 0 { 1 } else { 0 };
                        self.V[x as usize] <<= self.V[y as usize];
                    }
                    _ => {
                        // invalid instruction
                    }
                }
            }
            9 => {
                // SNE Vx, Vy
                if nibbles[3] == 0 {
                    if self.V[x as usize] != self.V[y as usize] {
                        self.PC += 2;
                    }
                }
            }
            0xA => {
                // LD I, addr
                self.I = address;
            }
            0xB => {
                // JP V0, addr
                self.PC = address + self.V[0] as u16;
            }
            0xC => {
                // RND Vx, byte
                let random = rand::random::<u8>();
                self.V[x as usize] = random & kk;
            }
            0xD => {
                // DRW Vx, Vy, nibble
                let mut cur_row = self.V[y as usize];
                let mut cur_col = self.V[x as usize];
                let mut collision = false;
                for _ in 0..nibbles[3] {
                    let row = self.memory.get(self.I);
                    self.I += 1;

                    // wrap around
                    if cur_row >= (self.display.get_height() as u8) {
                        cur_row = 0;
                    }
                    for j in 0..8 {
                        // wrap around
                        if cur_col >= (self.display.get_width() as u8) {
                            cur_col = 0;
                        }

                        // XOR and check for colliding pixels
                        collision |= self.display.xor_pixel(
                            cur_col as u16,
                            cur_row as u16,
                            row & (1 << (8 - 1 - j)) != 0,
                        );
                        cur_col += 1;
                    }
                    cur_col = self.V[x as usize];
                    cur_row += 1;
                }
                self.display.redraw();
                self.V[0xF] = collision as u8;
            }
            0xE => {
                // SKP, SKNP
                panic!("SKP, SKNP instruction not implemented, need keyboard");
            }
            0xF => {
                // LD, ADD
                match nibbles[2] << 4 | nibbles[3] {
                    0x07 => {
                        // LD Vx, DT
                        self.V[x as usize] = self.DT;
                    }
                    0x0A => {
                        // LD Vx, K
                        panic!("LD Vx, K instruction not implemented, need keyboard");
                    }
                    0x15 => {
                        // LD DT, Vx
                        self.DT = self.V[x as usize];
                    }
                    0x18 => {
                        // LD ST, Vx
                        self.ST = self.V[x as usize];
                    }
                    0x1E => {
                        // ADD I, Vx
                        self.I += self.V[x as usize] as u16;
                    }
                    0x29 => {
                        // LD F, Vx
                        self.I = self.memory.get_sprite_location(self.V[x as usize]);
                    }
                    0x33 => {
                        // LD B, Vx
                        let value = self.V[x as usize];
                        self.memory.store(self.I, value / 100);
                        self.memory.store(self.I + 1, (value % 100) / 10);
                        self.memory.store(self.I + 2, value % 10);
                    }
                    0x55 => {
                        // LD [I], Vx
                        for i in 0..x + 1 {
                            self.memory.store(self.I, self.V[i as usize]);
                            self.I += 1;
                        }
                    }
                    0x65 => {
                        // LD Vx, [I]
                        for i in 0..x + 1 {
                            self.V[i as usize] = self.memory.get(self.I);
                            self.I += 1;
                        }
                    }
                    _ => {
                        // invalid instruction
                    }
                }
            }
            _ => {}
        }
    }
}
