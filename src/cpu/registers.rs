//包含一个结构体，用于表示 6502 CPU 的寄存器。这些寄存器包括累加器（A）、X 索引寄存器（X）、Y 索引寄存器（Y）、程序计数器（PC）和状态寄存器（P）。此文件还可以包含与寄存器相关的方法，例如设置和清除状态标志等。
// src/cpu/registers.rs

/// 6502 CPU 寄存器结构体
pub struct Registers {
    pub a: u8,        // 累加器
    pub x: u8,        // X 寄存器
    pub y: u8,        // Y 寄存器
    pub pc: u16,      // 程序计数器
    pub sp: u8,       // 栈指针
    pub p: u8,   // 状态寄存器
}

#[derive(Copy, Clone)]
pub enum StatusFlags {
    Carry = 1 << 0,
    Zero = 1 << 1,
    InterruptDisable = 1 << 2,
    DecimalMode = 1 << 3,
    BreakCommand = 1 << 4,
    Unused = 1 << 5,
    Overflow = 1 << 6,
    Negative = 1 << 7,
}

impl std::default::Default for Registers {
    fn default() -> Self {
        Registers {
            a: 0,
            x: 0,
            y: 0,
            pc: 0xC000, // debug
            sp: 0xFD, // debug
            p: 0x24,
        }
    }
}

impl Registers {
    /// 设置状态寄存器的某个标志位
    pub fn set_flag(&mut self, flag: StatusFlags, value: bool) {
        let flag = flag as u8;
        if value {
            self.p |= flag;
        } else {
            self.p &= !flag;
        }
    }

    /// 获取状态寄存器的某个标志位
    pub fn get_flag(&self, flag: StatusFlags) -> bool {
        let flag = flag as u8;
        (self.p & flag) != 0
    }
}
