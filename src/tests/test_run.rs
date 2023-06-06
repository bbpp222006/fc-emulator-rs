use crossbeam::channel::bounded;
use crossbeam::select;
use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::thread;

// 你的模拟器的引用
use crate::emulator::Emulator;
use crate::window::MyApp;

pub fn run_test() {
    let rom_path = "rom/cpu_dummy_reads.nes";
    let mut emulator = Emulator::new();
    let pip_ppu_frameout = emulator.pip_ppu_frame.1.clone();
    let pip_input_stream_in = emulator.pip_input_stream.0.clone();
    emulator.load_rom(rom_path);
    let a = thread::spawn(move || loop {
        emulator.clock(); // 在此处运行模拟器的单步执行功能
    });

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(340.0, 261.0)),
        ..Default::default()
    };

    
    eframe::run_native(
        "Show an image with eframe/egui",
        options,
        Box::new(move|cc| Box::new(MyApp::new(pip_ppu_frameout,pip_input_stream_in))),
    ).unwrap();
    // a.join().unwrap();
}
