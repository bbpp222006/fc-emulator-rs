//包含一个枚举类型，用于表示所有可能的寻址模式，以及与寻址模式相关的逻辑。例如，可以实现一个函数（decode），用于根据操作码确定相应的寻址模式，以及一个函数（get_operand_address），用于根据寻址模式计算操作数的地址。
// src/cpu/addressing_modes.rs

use crate::memory::Memory;
use crate::cpu::registers::Registers;

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

impl AddressingMode {
    /// 获取操作数及下一条指令的地址
    pub fn get_operand(
        &self,
        memory: &Memory,
        registers: &Registers,
        address: u16,
    )-> (Option<u16>, u16) {
        match *self {
            AddressingMode::Implied => (None, address),
            AddressingMode::Accumulator => (None, address),
            AddressingMode::Immediate => (Some(memory.read(address) as u16), address + 1),
            AddressingMode::Absolute => {
                let operand_address = memory.read_u16(address);
                let operand = memory.read_u16(operand_address);
                (Some(operand), address + 2)
            }
            AddressingMode::AbsoluteX => {
                let base_address = memory.read_u16(address);
                let operand_address = base_address + registers.x as u16;
                let operand = memory.read_u16(operand_address);
                (Some(operand), address + 2)
            }
            AddressingMode::AbsoluteY => {
                let base = memory.read_u16(address);
                let operand = base + registers.y as u16;
                (Some(operand), address + 2)
            }
            AddressingMode::ZeroPage => {
                let operand = memory.read(address) as u16;
                (Some(operand), address + 1)
            }
            AddressingMode::ZeroPageX => {
                let base = memory.read(address) as u16;
                let operand = (base + registers.x as u16) & 0xFF;
                (Some(operand), address + 1)
            }
            AddressingMode::ZeroPageY => {
                let base = memory.read(address) as u16;
                let operand = (base + registers.y as u16) & 0xFF;
                (Some(operand), address + 1)
            }
            AddressingMode::Indirect => {
                let pointer = memory.read_u16(address);
                let operand = memory.read_u16(pointer);
                (Some(operand), address + 2)
            }
            AddressingMode::IndirectX => {
                let base = memory.read(address);
                let pointer = (base as u16 + registers.x as u16) & 0xFF;
                let operand = memory.read_u16(pointer);
                (Some(operand), address + 1)
            }
            AddressingMode::IndirectY => {
                let base = memory.read(address);
                let pointer = base as u16;
                let operand = memory.read_u16(pointer) + registers.y as u16;
                (Some(operand), address + 1)
            }
            AddressingMode::Relative => {
                let offset = memory.read(address) as i8;
                let operand = (address as i32 + offset as i32 + 1) as u16;
                (Some(operand), address + 1)
            }
        }
    }
}