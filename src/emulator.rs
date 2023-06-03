use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::thread;

use crossbeam::channel::{bounded, select, Receiver, Sender};

use crate::bus::{start_bus_thread, RWMessage, RWResult};
use crate::cpu::start_cpu_thread;
use crate::ppu::start_ppu_thread;
use crate::utils::{Frame, GlobalSignal, Palettes};

use crate::window::MyApp;

pub struct Emulator {
    pub pip_cpu2bus: (Sender<RWMessage>, Receiver<RWMessage>),
    pub pip_bus2cpu: (Sender<RWResult>, Receiver<RWResult>),
    pub pip_ppu2bus: (Sender<RWMessage>, Receiver<RWMessage>),
    pub pip_bus2ppu: (Sender<RWResult>, Receiver<RWResult>),
    pub pip_apu2bus: (Sender<RWMessage>, Receiver<RWMessage>),
    pub pip_bus2apu: (Sender<RWResult>, Receiver<RWResult>),
    pub pip_global_signal: PipGlobalSignal,
    pub pip_rom: (Sender<Vec<u8>>, Receiver<Vec<u8>>),
    pub pip_log: (Sender<String>, Receiver<String>),
    pub pip_ppu_frame: (Sender<Frame>, Receiver<Frame>),
    pub palettes: Palettes,
    frame_cache: Frame,
    // pub window: Window,
}

pub struct PipGlobalSignal{
    cpu: (Sender<GlobalSignal>, Receiver<GlobalSignal>),
    ppu: (Sender<GlobalSignal>, Receiver<GlobalSignal>),
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
        Emulator {
            pip_cpu2bus,
            pip_bus2cpu,
            pip_ppu2bus,
            pip_bus2ppu,
            pip_apu2bus,
            pip_bus2apu,
            pip_global_signal: PipGlobalSignal{
                cpu: bounded(1),
                ppu: bounded(1),
            },
            pip_rom,
            pip_log,
            pip_ppu_frame,
            palettes: Palettes::new(),
            frame_cache: Frame::new(),
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
            self.pip_global_signal.cpu.1.clone(),
            self.pip_log.0.clone(),
        );

        start_ppu_thread(
            self.pip_ppu2bus.0.clone(),
            self.pip_bus2ppu.1.clone(),
            self.pip_ppu_frame.0.clone(),
            self.pip_global_signal.ppu.1.clone(),
            self.pip_log.0.clone(),
        );
    }

    fn send_global_signal(&self, signal: GlobalSignal) {
        self.pip_global_signal.cpu.0.clone().send(signal).unwrap();
        self.pip_global_signal.ppu.0.clone().send(signal).unwrap();
    }

    pub fn load_rom(&self, path: &str) {
        let mut file = File::open(Path::new(path)).expect("无法打开 ROM 文件");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).expect("无法读取 ROM 文件");
        self.pip_rom.0.clone().send(buffer).unwrap();
        self.send_global_signal(GlobalSignal::Reset);
    }

    pub fn step(&self) {
        self.send_global_signal(GlobalSignal::Step);
    }

    pub fn clock(&self) {
        self.send_global_signal(GlobalSignal::Clock);
    }

    pub fn get_log(&self) -> String {
        self.send_global_signal(GlobalSignal::GetLog);
        self.pip_log.1.clone().recv().unwrap()
    }
}
