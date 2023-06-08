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
    let mut emulator = Emulator::new();
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(340.0, 261.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Show an image with eframe/egui",
        options,
        Box::new(move|cc| Box::new(MyApp::new(cc,emulator))),
    ).unwrap();
    // a.join().unwrap();
}
