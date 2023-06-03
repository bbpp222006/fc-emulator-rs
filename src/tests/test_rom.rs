use std::f32::consts::E;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use regex::Regex;

// 模拟器的引用
// 注意：运行此测试时，需要将src\cpu\cpu.rs中的reset函数中的self.registers.pc = self.read_u16(0xFFFC);注释掉，从特定地址直接运行
use crate::emulator::Emulator;

fn compare_logs(emulator_log: &str, expected_log: &str) -> bool {
    let re = Regex::new(r"(?P<cyc>CYC:\d+)").unwrap();
    let emulator_log_line = re.captures(emulator_log).unwrap();
    let expected_log_line = re.captures(expected_log).unwrap();
    (emulator_log[..74] == expected_log[..74])&&(emulator_log_line["cyc"] == expected_log_line["cyc"])
}

pub fn run_test() {
    let rom_path = "rom/nestest.nes";
    let log_path = "rom/nestest.log";

    let emulator =Emulator::new();
    emulator.start();
    emulator.load_rom(rom_path);
 
    let log_file = File::open(&Path::new(log_path)).expect("Unable to open log file");

    let mut current_num = 0;
    for expected_log_line in io::BufReader::new(log_file).lines() {
        let expected_log_line = expected_log_line.unwrap();
        println!("{}", expected_log_line);

        let emulator_log_line = emulator.get_log(); // 获取模拟器的日志
        // println!("获取日志结束");
        println!("{}", emulator_log_line);

        assert!(
            compare_logs(&emulator_log_line, &expected_log_line),
            "Emulator log: {}\nExpected log: {}",
            emulator_log_line,
            expected_log_line
        );
        
        emulator.step(); // 在此处运行模拟器的单步执行功能
        current_num+=1;
        // println!("after 0x0400:{:02X}",emulator.cpu.memory.ram[0x0081]);

        println!("{:*<48}", current_num);
    }

    println!("Test passed!");
}
