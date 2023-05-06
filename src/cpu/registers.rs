//包含一个结构体，用于表示 6502 CPU 的寄存器。这些寄存器包括累加器（A）、X 索引寄存器（X）、Y 索引寄存器（Y）、程序计数器（PC）和状态寄存器（P）。此文件还可以包含与寄存器相关的方法，例如设置和清除状态标志等。
// src/cpu/registers.rs

/// 6502 CPU 寄存器结构体
pub struct Registers {
    pub a: u8,        // 累加器
    pub x: u8,        // X 寄存器
    pub y: u8,        // Y 寄存器
    pub pc: u16,      // 程序计数器
    pub sp: u8,       // 栈指针
    pub status: u8,   // 状态寄存器
}

impl Registers {
    /// 创建一个新的 Registers 实例
    pub fn new() -> Self {
        Registers {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            sp: 0,
            status: 0,
        }
    }

    /// 设置状态寄存器的某个标志位
    pub fn set_flag(&mut self, flag: u8, value: bool) {
        if value {
            self.status |= flag;
        } else {
            self.status &= !flag;
        }
    }

    /// 获取状态寄存器的某个标志位
    pub fn get_flag(&self, flag: u8) -> bool {
        (self.status & flag) != 0
    }
}
