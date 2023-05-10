use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::cpu::{Cpu};
use crate::memory::Memory;
use crate::Disassembler;

pub struct Emulator {
    pub cpu: Cpu,
}

impl Emulator {
    pub fn new() -> Self {
        let memory = Memory::default();
        let cpu = Cpu::new(memory);
        Emulator { 
            cpu,
        }
    }

    pub fn load_rom(&mut self, path: &str) {
        let mut file = File::open(Path::new(path)).expect("无法打开 ROM 文件");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).expect("无法读取 ROM 文件");
        self.cpu.memory.load_rom(buffer);
    }

    pub fn step(&mut self) {
        self.cpu.step();
    }

    pub fn get_log(&self) -> String {
        self.cpu.get_current_log()
    }


}
