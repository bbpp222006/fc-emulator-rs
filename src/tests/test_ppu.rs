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
    let rom_path = "rom/nestest.nes";
    let mut emulator = Emulator::new();
    let pip_ppu_frameout = emulator.pip_ppu_frame.1.clone();
    let pip_input_stream_in = emulator.pip_input_stream.0.clone();
    emulator.load_rom(rom_path);
    let a = thread::spawn(move || loop {
        emulator.clock(); // 在此处运行模拟器的单步执行功能
    });

    let mut fps = 0.0;
    let mut fps_time = std::time::Instant::now();

    let mut frame_num = 0;
    for _ in pip_ppu_frameout.iter() {
        fps = 1.0 / fps_time.elapsed().as_secs_f32();
        println!("ppu_fps: {}", fps);
        fps_time = std::time::Instant::now();
        frame_num += 1;
        if frame_num == 100 {
            break;
        }
    }
    print!("Hello World")
}
