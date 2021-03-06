use std::fs::File;
use std::io::{Read};

pub struct Memory {
    memory: [u8; 0x1000], // 4095 byte memory
                          // user programs should only use memory from 0x200
}

impl Memory {
    pub fn new() -> Memory {
        let mut mem = [0; 0x1000];

        // put the sprites of the normal letters in lower memory
        mem[0.. 0x10 * 5].clone_from_slice(&[
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ]);

        Memory { memory: mem }
    }

    pub fn read_file(&mut self, file: &mut File) {
        file.read(&mut self.memory[0x200..]).unwrap();
    }

    pub fn store(&mut self, addr: u16, value: u8) {
        assert_eq!(addr < 0x1000, true);
        self.memory[addr as usize] = value;
    }

    pub fn get(&self, addr: u16) -> u8 {
        assert_eq!(addr < 0x1000, true);
        self.memory[addr as usize]
    }

    pub fn get_memory(&self) -> &[u8] {
        &self.memory
    }

    pub fn get_sprite_location(&self, value: u8) -> u16 {
        assert_eq!(value <= 0xF, true);
        value as u16 * 5
    }
}
