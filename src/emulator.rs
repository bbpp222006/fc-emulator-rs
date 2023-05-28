use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::thread;

use crossbeam::channel::{bounded, select, Receiver, Sender};

use crate::cpu::start_cpu_thread;
use crate::bus::{RWMessage,RWResult,start_bus_thread};
use crate::utils::Frame;

use crate::utils::GlobalSignal;
use crate::window::MyApp;

pub struct Emulator {
    pub pip_cpu2bus:(Sender<RWMessage>, Receiver<RWMessage>),
    pub pip_bus2cpu:  (Sender<RWResult>, Receiver<RWResult>),
    pub pip_ppu2bus: (Sender<RWMessage>, Receiver<RWMessage>),
    pub pip_bus2ppu: (Sender<RWResult>, Receiver<RWResult>),
    pub pip_apu2bus: (Sender<RWMessage>, Receiver<RWMessage>),
    pub pip_bus2apu: (Sender<RWResult>, Receiver<RWResult>),
    pub pip_global_signal: (Sender<GlobalSignal>, Receiver<GlobalSignal>),
    pub pip_rom: (Sender<Vec<u8>>, Receiver<Vec<u8>>),
    pub pip_log: (Sender<String>, Receiver<String>),
    pub pip_ppu_frame: (Sender<Frame>, Receiver<Frame>),
    // pub window: Window,
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
        let pip_global_signal = bounded(1);
        let pip_rom = bounded(1);
        let pip_log = bounded(1);
        let pip_ppu_frame = bounded(1);
        Emulator {
            pip_cpu2bus,
            pip_bus2cpu,
            pip_ppu2bus,
            pip_bus2ppu,
            pip_apu2bus,
            pip_bus2apu,
            pip_global_signal,
            pip_rom,
            pip_log,
            pip_ppu_frame,
        }
    }


    pub fn start(&self) {
        // 将通信队列连接起来
        start_bus_thread(
            self.pip_bus2cpu.0.clone(),
            self.pip_cpu2bus.1.clone(),
            self.pip_bus2ppu.0.clone(),
            self.pip_ppu2bus.1.clone(),
            self.pip_bus2apu.0.clone(),
            self.pip_apu2bus.1.clone(),
            self.pip_rom.1.clone(),
        );
        start_cpu_thread(
            self.pip_cpu2bus.0.clone(),
            self.pip_bus2cpu.1.clone(),
            self.pip_global_signal.1.clone(),
            self.pip_log.0.clone(),
        );

        
        let options = eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(340.0, 261.0)),
            ..Default::default()
        };
        let pip_ppu_frameout = self.pip_ppu_frame.1.clone();
        eframe::run_native(
            "Show an image with eframe/egui",
            options,
            Box::new(|cc| Box::new(MyApp::new(pip_ppu_frameout))),
        )
        .unwrap(); 
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
