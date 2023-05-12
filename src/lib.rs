#![allow(unused_imports, unused_variables, dead_code)]


// 引入我们的各个模块
mod cpu;
mod memory;
mod emulator;
mod utils;

// 将我们希望暴露给其他使用此库的项目的类型、结构和函数重新导出
// 这使得它们可以在外部项目中使用
pub use cpu::Cpu;
pub use memory::{Memory,RomHeader};
pub use emulator::Emulator;
// pub use utils::disassembler::Disassembler;

// 我们还可以在这里定义一些库级别的公共函数或常量
// 例如，我们可以定义一个用于处理库中的错误的统一错误类型
pub type NesResult<T> = Result<T, NesError>;

#[derive(Debug)]
pub enum NesError {
    // 在此处定义库中可能遇到的各种错误
    IoError(std::io::Error),
    InvalidRom,
    // ...
}

impl From<std::io::Error> for NesError {
    fn from(error: std::io::Error) -> Self {
        NesError::IoError(error)
    }
}
