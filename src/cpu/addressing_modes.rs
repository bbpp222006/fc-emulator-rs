//包含一个枚举类型，用于表示所有可能的寻址模式，以及与寻址模式相关的逻辑。例如，可以实现一个函数（decode），用于根据操作码确定相应的寻址模式，以及一个函数（get_operand_address），用于根据寻址模式计算操作数的地址。
// src/cpu/addressing_modes.rs
pub use std::fmt::Write;


/// 6502 CPU 的寻址模式枚举
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AddressingMode {
    Implied, // 隐含寻址
    Accumulator, // 累加器寻址
    Immediate, // 立即寻址
    Absolute,// 绝对寻址
    AbsoluteX,// 绝对寻址 X
    AbsoluteY,// 绝对寻址 Y
    ZeroPage,// 零页寻址
    ZeroPageX,// 零页寻址 X
    ZeroPageY,// 零页寻址 Y
    Indirect,// 间接寻址
    IndirectX,// 间接寻址 X
    IndirectY,// 间接寻址 Y
    Relative,// 相对寻址
}
