use std::collections::HashSet;
use std::collections::VecDeque;

use egui::Key;

pub struct ApuIoRegisters{
    pub ram: [u8; 0x20],
    pub inpur_reg:HashSet<egui::Key>, // 当前的输入状态，随时发生变化
    pub current_input: VecDeque<u8>, // 0x4016 顺序是A, B, Select, Start, Up, Down, Left, Right
    pub input_enable: bool,
    pub input_history: u8, //用于debug
}


impl ApuIoRegisters {
    pub fn new() -> Self {
        ApuIoRegisters {
            ram: [0; 0x20],
            current_input: VecDeque::new(),
            input_enable: false,
            input_history: 0,
            inpur_reg: HashSet::new(),
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        let ram_addr = addr & 0x1F;
        let mut data = self.ram[ram_addr as usize];
        match ram_addr {
            0x16 => {
                if self.input_enable == true {
                    // print!("当前输入{:?}",self.current_input);
                    data = self.current_input.pop_front().unwrap_or(0);
                    // println!("当前输出：{:?}",data);
                }
            },
            _ => {}
        }
        self.input_history = data;
        data
    }

    pub fn read_debug (&self, addr: u16) -> u8 {
        self.input_history
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        let ram_addr = addr & 0x1F;
        self.ram[ram_addr as usize]= data;
        match ram_addr {
            0x16 => {
                if data & 0x01 == 0x01 {
                    // println!("重置输入");
                    self.input_enable = false;
                    self.current_input = VecDeque::new();
                }else if data & 0x01 == 0 {
                    self.input_enable = true;
                    if self.inpur_reg.is_empty(){
                        self.current_input = VecDeque::new();
                    }else {
                        let mut current_input = VecDeque::from([0; 8].to_vec());
                        for key in &self.inpur_reg {
                            match key {
                                Key::J => current_input[0] = 1,
                                Key::K => current_input[1] = 1,
                                Key::Space => current_input[2] = 1,
                                Key::Enter => current_input[3] = 1,
                                Key::W => current_input[4] = 1,
                                Key::S => current_input[5] = 1,
                                Key::A => current_input[6] = 1,
                                Key::D => current_input[7] = 1,
                                _ => {}
                            }
                        }
                        self.current_input =  current_input;
                    }
                    // print!("接收到输入{:?}",self.current_input);
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