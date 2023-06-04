use std::collections::HashSet;
use std::collections::VecDeque;

use egui::Key;

pub struct ApuIoRegisters{
    ram: [u8; 0x20],
    pub current_input: VecDeque<u8>, // 0x4016 顺序是A, B, Select, Start, Up, Down, Left, Right
    pub input_enable: bool,
}


impl ApuIoRegisters {
    pub fn new() -> Self {
        ApuIoRegisters {
            ram: [0; 0x20],
            current_input: VecDeque::new(),
            input_enable: false,
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        let ram_addr = addr & 0x1F;
        let mut data = self.ram[ram_addr as usize];
        // println!("read {:04X}, current input {:?}", addr,self.current_input);
        match ram_addr {
            0x16 => {
                if self.input_enable == true {
                    data = self.current_input.pop_front().unwrap_or(0);
                }
            },
            _ => {}
        }
        data
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        let ram_addr = addr & 0x1F;
        self.ram[ram_addr as usize]= data;
        match ram_addr {
            0x16 => {
                if data & 0x01 == 0x01 {
                    self.input_enable = false;
                }else if data & 0x01 == 0 {
                    self.input_enable = true;
                }
            },
            _ => {}
        }
    }

    pub fn reset(&mut self) {
        self.ram = [0; 0x20];
        self.current_input =  VecDeque::new();
        self.input_enable = false;
    }
}