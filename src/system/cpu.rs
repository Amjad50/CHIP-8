use super::display::Display;
use super::memory::Memory;
use super::sound::Sound;
use rand; // used for the RND instruction only.
use std::cell::RefCell;
use std::fs::File;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

pub struct CPU {
    V: [u8; 16],             // 16 8-bit Vx register
    I: u16,                  // I register
    DT: u8,                  // Delay timer
    ST: u8,                  // Sound timer
    PC: u16,                 // Program counter
    SP: u8,                  // Stack pointer
    stack: [u16; 16],        // Internal stack of 16 16-bit values
    memory: RefCell<Memory>, // Memory component
    display: Display,        // Display component
    beep_sound: Sound,

    wait_for_keypress_x: i8, // used to indicate the waiting for keypress for instruction Fx0A - LD Vx, K
    last_update: SystemTime,
}

impl CPU {
    pub fn new() -> CPU {
        let cpu = CPU {
            V: [0u8; 16],
            I: 0,
            DT: 0,
            ST: 0,
            PC: 0x200, // start at 0x200 address
            SP: 0,
            stack: [0; 16],
            memory: RefCell::new(Memory::new()),
            display: Display::new(64, 32),
            wait_for_keypress_x: -1,
            last_update: SystemTime::now(),
            beep_sound: Sound::new(300),    // frequency of the sin wave
        };
        cpu.setup_keyboard();
        cpu
    }

    pub fn setup_keyboard(&self) {
        self.display.setup_keyboard();
    }

    pub fn run_display_application(self) {
        let cpu_rc = Rc::new(RefCell::new(self));
        let c_cpu = cpu_rc.clone();

        const FPS: u32 = 1000;

        cpu_rc.borrow().display.run_in_loop(1000 / FPS, move || {
            let mut cpu = c_cpu.borrow_mut();

            // cpu waiting for key press
            if cpu.wait_for_keypress_x > -1 {
                // get the keyboard press layout
                let keyboard = cpu.display.get_keyboard_data_copy();
                // if any key is being pressed, wait no more and assign
                // the value of they key to the register Vx
                match keyboard.iter().position(|&x| x) {
                    Some(key) => {
                        let x = cpu.wait_for_keypress_x as usize;
                        cpu.V[x] = key as u8;
                        cpu.wait_for_keypress_x = -1;
                    }
                    None => {}
                };
                
                return;
            }

            let instruction = (cpu.memory.borrow().get(cpu.PC) as u16) << 8
                | (cpu.memory.borrow().get(cpu.PC + 1) as u16);
            cpu.run_instruction(instruction);
            cpu.PC += 2;

            match cpu.last_update.elapsed() {
                Ok(duration) => {
                    if duration > Duration::from_millis(1000 / 60) {
                        if cpu.ST > 0 {
                            cpu.ST -= 1;

                            cpu.play_beep();
                        } else {
                            cpu.stop_beep();
                        }

                        if cpu.DT > 0 {
                            cpu.DT -= 1;
                        }

                        cpu.last_update = SystemTime::now();
                    }
                }
                Err(e) => {
                    println!("Error occurred in SystemTime::elapsed {}", e);
                }
            }
        });

        // run the application
        Display::run_application();
    }

    pub fn read_file(&mut self, file: &mut File) {
        self.memory.borrow_mut().read_file(file);
    }

    pub fn play_beep(&mut self) {
        if self.beep_sound.is_paused() {
            self.beep_sound.play();
        }
    }

    pub fn stop_beep(&mut self) {
        if !self.beep_sound.is_paused() {
            self.beep_sound.stop();
        }
    }

    pub fn run_instruction(&mut self, instruction: u16) {
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
                        self.PC = return_address - 2;
                    }
                    _ => {
                        // SYS addr
                        self.PC = address - 2;
                    }
                }
            }
            1 => {
                // JMP addr
                self.PC = address - 2;
            }
            2 => {
                // CALL addr
                self.stack[self.SP as usize] = self.PC + 2;
                self.SP += 1;
                self.PC = address - 2;
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
                self.V[x as usize] = (self.V[x as usize] as u16 + kk as u16) as u8;
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
                        self.V[x as usize] =
                            (self.V[x as usize] as i16 - self.V[y as usize] as i16) as u8;
                    }
                    6 => {
                        // SHR Vx {, Vy}
                        // TODO: not sure if this can have empty y value and what does that mean
                        self.V[0xF] = if self.V[x as usize] & 1 != 0 { 1 } else { 0 };
                        self.V[x as usize] >>= 1;
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
                        self.V[x as usize] <<= 1;
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
                self.PC = address + self.V[0] as u16 - 2;
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
                for i in 0..nibbles[3] {
                    let row = self.memory.borrow().get(self.I + i as u16);

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
                match nibbles[2] << 4 | nibbles[3] {
                    0x9E => {
                        // SKP Vx
                        if self.display.get_keyboard_data()[self.V[x as usize] as usize] {
                            self.PC += 2;
                        }
                    }
                    0xA1 => {
                        // SKNP Vx
                        if !self.display.get_keyboard_data()[self.V[x as usize] as usize] {
                            self.PC += 2;
                        }
                    }
                    _ => {}
                }
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
                        self.wait_for_keypress_x = x as i8;
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
                        self.I = self.memory.borrow().get_sprite_location(self.V[x as usize]);
                    }
                    0x33 => {
                        // LD B, Vx
                        let value = self.V[x as usize];
                        let mut memory = self.memory.borrow_mut();
                        memory.store(self.I, value / 100);
                        memory.store(self.I + 1, (value % 100) / 10);
                        memory.store(self.I + 2, value % 10);
                    }
                    0x55 => {
                        // LD [I], Vx
                        for i in 0..x + 1 {
                            self.memory.borrow_mut().store(self.I, self.V[i as usize]);
                            self.I += 1;
                        }
                    }
                    0x65 => {
                        // LD Vx, [I]
                        for i in 0..x + 1 {
                            self.V[i as usize] = self.memory.borrow().get(self.I);
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
