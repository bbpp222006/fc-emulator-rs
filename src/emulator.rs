use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::thread;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use crossbeam::channel::{bounded, select, Receiver, Sender};
use egui::Key;

use crate::bus::{ RWMessage, RWResult,Bus};
use crate::cpu::{Cpu};
use crate::ppu::{Ppu};
use crate::utils::{Frame, GlobalSignal, Palettes};

use crate::window::MyApp;

pub struct Emulator {
    pub pip_cpu2bus: (Sender<RWMessage>, Receiver<RWMessage>),
    pub pip_bus2cpu: (Sender<RWResult>, Receiver<RWResult>),
    pub pip_ppu2bus: (Sender<RWMessage>, Receiver<RWMessage>),
    pub pip_bus2ppu: (Sender<RWResult>, Receiver<RWResult>),
    pub pip_apu2bus: (Sender<RWMessage>, Receiver<RWMessage>),
    pub pip_bus2apu: (Sender<RWResult>, Receiver<RWResult>),
    pub pip_rom: (Sender<Vec<u8>>, Receiver<Vec<u8>>),
    pub pip_log: (Sender<String>, Receiver<String>),
    pub pip_ppu_frame: (Sender<Frame>, Receiver<Frame>),
    pub palettes: Palettes,
    pub pip_input_stream: (Sender<HashSet<egui::Key>>, Receiver<HashSet<egui::Key>>),
    // pub window: Window,
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub bus: Rc<RefCell<Bus>>,
    log: String,
}


impl Emulator {
    pub fn new() -> Self {
        // 初始化通信管道
        let pip_cpu2bus = bounded(1);
        let pip_bus2cpu = bounded(1);
        let pip_ppu2bus = bounded(1);
        let pip_bus2ppu = bounded(1);
        let pip_apu2bus = bounded(1);
        let pip_bus2apu = bounded(1);
        let pip_rom = bounded(1);
        let pip_log = bounded(1);
        let pip_ppu_frame = bounded(1);
        let pip_input_stream: (Sender<HashSet<Key>>, Receiver<HashSet<Key>>) = bounded(1);
        let bus: Rc<RefCell<Bus>>  = Rc::new(RefCell::new(Bus::new(pip_input_stream.1.clone())));
        let cpu = Cpu::new(Rc::clone(&bus));
        let ppu = Ppu::new(Rc::clone(&bus),pip_ppu_frame.0.clone());
        Emulator {
            pip_cpu2bus,
            pip_bus2cpu,
            pip_ppu2bus,
            pip_bus2ppu,
            pip_apu2bus,
            pip_bus2apu,
            pip_rom,
            pip_log,
            pip_ppu_frame,
            palettes: Palettes::new(),
            pip_input_stream,
            cpu,
            ppu,
            bus,
            log: String::new(),
        }
    }

    pub fn load_rom(&mut self, path: &str) {
        let mut file = File::open(Path::new(path)).expect("无法打开 ROM 文件");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).expect("无法读取 ROM 文件");
        self.bus.borrow_mut().load_rom(buffer);
        self.reset();
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.ppu.reset();
        self.bus.borrow_mut().reset();
    }

    pub fn hard_reset(&mut self) {
        self.cpu.hard_reset();
        self.ppu.hard_reset();
        self.bus.borrow_mut().hard_reset();
    }

    

    pub fn refresh_input(&mut self,new_input:HashSet<egui::Key>) {
        self.bus.borrow_mut().refresh_input(new_input);
    }

    pub fn cpu_clock(&mut self) {
        if self.cpu.cpu_cycle_wait == 0 {
            self.cpu.step();
        } else {
            self.cpu.cpu_cycle_wait -= 1;
        }
        for _ in 0..3 {
            self.ppu.step();
        }
    }

    pub fn cpu_step(&mut self) {
        while self.cpu.cpu_cycle_wait != 0 {
            for _ in 0..3 {
                self.ppu.step();
            }
            self.cpu.cpu_cycle_wait -= 1;
        }
        self.cpu.step();
        while self.cpu.cpu_cycle_wait != 0 {
            for _ in 0..3 {
                self.ppu.step();
            }
            self.cpu.cpu_cycle_wait -= 1;
        }
    }
    
    pub fn get_log(&self) -> String {
        self.cpu.get_current_log()
    }
}
