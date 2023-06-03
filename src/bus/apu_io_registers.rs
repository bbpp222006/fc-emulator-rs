use std::collections::HashSet;
use std::collections::VecDeque;

use crossbeam::channel::{Receiver, bounded};
use egui::Key;

pub struct ApuIoRegisters{
    ram: [u8; 0x20],
    pub input_stream: Receiver<HashSet<Key>>,
    current_input: VecDeque<u8> // 0x4016 顺序是A, B, Select, Start, Up, Down, Left, Right
}


impl ApuIoRegisters {
    pub fn new(input_stream: Receiver<HashSet<Key>>) -> Self {
        ApuIoRegisters {
            ram: [0; 0x20],
            input_stream,
            current_input: VecDeque::new(),
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        let ram_addr = addr & 0x1F;
        let mut data = self.ram[ram_addr as usize];
        // println!("read {:04X}", addr);
        match ram_addr {
            0x16 => {
                if self.current_input.is_empty() {
                    let input = self.input_stream.try_recv();
                    println!("get input {:?}", input);
                    match input {
                        Ok(input) => {
                            self.current_input = VecDeque::from([0; 8].to_vec());
                            for key in input {
                                match key {
                                    Key::J => self.current_input[0] = 1,
                                    Key::K => self.current_input[1] = 1,
                                    Key::Space => self.current_input[2] = 1,
                                    Key::Enter => self.current_input[3] = 1,
                                    Key::W => self.current_input[4] = 1,
                                    Key::S => self.current_input[5] = 1,
                                    Key::A => self.current_input[6] = 1,
                                    Key::D => self.current_input[7] = 1,
                                    _ => {}
                                }
                            }
                        },
                        Err(_) => {}
                    }
                }
                data = self.current_input.pop_front().unwrap_or(0);
            },
            _ => {}
        }
        data
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        let ram_addr = addr & 0x1F;
        self.ram[ram_addr as usize]= data;
        match addr {
            0x16 => {
                if data & 0x01 == 0x01 {
                    self.current_input = VecDeque::new();
                }
            },
            _ => {}
        }
    }

    pub fn reset(&mut self) {
        self.ram = [0; 0x20];
        self.current_input =  VecDeque::new();
    }
}