use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

// 你的模拟器的引用
use crate::emulator::Emulator;



pub async fn run_test() {
    let rom_path = "rom/nestest.nes";
    let mut emulator =Emulator::new();
    emulator.load_rom(rom_path);
 

    let mut current_num = 0;
     // 主循环
    loop {

        // 获取新的帧并将其绘制到窗口上
        let frame = emulator.get_frame();
        println!("frame:{:?}",frame.data);

    }

    println!("Test passed!");
}
