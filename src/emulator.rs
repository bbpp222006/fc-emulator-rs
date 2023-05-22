use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::thread;

use crossbeam::channel::{bounded, select, Receiver, Sender};

use crate::cpu::{start_cpu_thread, Cpu};
use crate::memory::{start_mem_thread, Memory, RWMessage, RWResult};
use crate::utils::Frame;
use crate::ppu::ppu_impl::Ppu;

use crate::utils::GlobalSignal;

pub struct Emulator {
    pub pip_mem_out: (Sender<RWResult>, Receiver<RWResult>),
    pub pip_mem_in: (Sender<RWMessage>, Receiver<RWMessage>),
    pub pip_global_signal: (Sender<GlobalSignal>, Receiver<GlobalSignal>),
    pub pip_rom: (Sender<Vec<u8>>, Receiver<Vec<u8>>),
    pub pip_log: (Sender<String>, Receiver<String>),
    // pub window: Window,
}

impl Emulator {
    pub fn new() -> Self {
        // 初始化通信管道
        let pip_mem_out = bounded(1);
        let pip_mem_in = bounded(1);
        let pip_global_signal = bounded(1);
        let pip_rom = bounded(1);
        let pip_log = bounded(1);
        Emulator {
            pip_mem_out,
            pip_mem_in,
            pip_global_signal,
            pip_rom,
            pip_log,
        }
    }

    pub fn start(&self) {
        // 将通信队列连接起来
        start_mem_thread(
            self.pip_mem_in.1.clone(),
            self.pip_mem_out.0.clone(),
            self.pip_rom.1.clone(),
        );
        start_cpu_thread(
            self.pip_mem_in.0.clone(),
            self.pip_mem_out.1.clone(),
            self.pip_global_signal.1.clone(),
            self.pip_log.0.clone(),
        );
    }

    pub fn load_rom(&self, path: &str) {
        let mut file = File::open(Path::new(path)).expect("无法打开 ROM 文件");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).expect("无法读取 ROM 文件");
        self.pip_rom.0.clone().send(buffer).unwrap();
        self.pip_global_signal
            .0
            .clone()
            .send(GlobalSignal::Reset)
            .unwrap();
    }

    pub fn step(&self) {
        self.pip_global_signal
            .0
            .clone()
            .send(GlobalSignal::Step)
            .unwrap();
    }

    pub fn get_log(&self) -> String {
        self.pip_global_signal
            .0
            .clone()
            .send(GlobalSignal::GetLog)
            .unwrap();
        self.pip_log.1.clone().recv().unwrap()
    }

    pub fn get_frame(&self) -> Frame {
        todo!()
    }
}
