//包含一个枚举类型，用于表示所有可能的寻址模式，以及与寻址模式相关的逻辑。例如，可以实现一个函数（decode），用于根据操作码确定相应的寻址模式，以及一个函数（get_operand_address），用于根据寻址模式计算操作数的地址。
// src/cpu/addressing_modes.rs

use crate::memory::Memory;
use crate::cpu::registers::{Registers,StatusFlags};
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



 //页面交叉判断
 fn check_page_boundary_crossed(addr1: u16, addr2: u16) -> bool {
    (addr1 & 0xFF00) != (addr2 & 0xFF00)
}

impl AddressingMode {
    /// 返回(操作数的地址，页面是否交叉)
    pub fn get_operand_address(
        &self,
        memory: &Memory,
        registers: &Registers,
        address: u16,
    )-> (u16,bool) {
        let mut operand_address = 0;
        let mut page_crossed = false;
        match *self {
            AddressingMode::Implied|AddressingMode::Accumulator => (),
            AddressingMode::Immediate => {
                operand_address = address;
            }
            AddressingMode::Absolute => {
                operand_address = memory.read_u16(address);
            }
            AddressingMode::AbsoluteX => {
                let base_address = memory.read_u16(address);
                operand_address = base_address + registers.x as u16;
                // 页面交叉判断
                if check_page_boundary_crossed(base_address ,operand_address){
                    page_crossed = true;
                }
            }
            AddressingMode::AbsoluteY => {
                let base_address = memory.read_u16(address);
                operand_address = base_address.wrapping_add(registers.y as u16);
                // 页面交叉判断
                if check_page_boundary_crossed(base_address ,operand_address){
                    page_crossed = true;
                }
            }
            AddressingMode::ZeroPage => {
                operand_address = memory.read(address) as u16;
            }
            AddressingMode::ZeroPageX => {
                let base_address = memory.read(address) as u16;
                operand_address = (base_address + registers.x as u16)& 0x00FF;
            }
            AddressingMode::ZeroPageY => {
                let base_address = memory.read(address) as u16;
                operand_address = (base_address + registers.y as u16)& 0x00FF;
            }
            AddressingMode::Indirect => {
                let operand_address_address = memory.read_u16(address);
                let low_byte = memory.read(operand_address_address);
                let high_byte = if operand_address_address & 0x00FF == 0x00FF {
                    memory.read(operand_address_address & 0xFF00)
                } else {
                    memory.read(operand_address_address + 1)
                };
                operand_address = (high_byte as u16) << 8 | low_byte as u16;
            }
            AddressingMode::IndirectX => {
                let base_address = memory.read(address);
                operand_address = memory.read_u16_z(base_address.wrapping_add(registers.x));
            }
            AddressingMode::IndirectY => {
                let base_address_address = memory.read(address);
                let base_address = memory.read_u16_z(base_address_address);
                operand_address = base_address.wrapping_add(registers.y as u16) ;
                // 页面交叉判断
                if check_page_boundary_crossed(base_address ,operand_address){
                    page_crossed = true;
                }
            }
            AddressingMode::Relative => {
                let init_offset = memory.read(address);
                let offset = init_offset as i8; // 读取当前地址的值作为偏移量（有符号数）
                operand_address = (address as i32 + 1 + (offset as i32)) as u16;
                // 页面交叉判断
                if check_page_boundary_crossed(address+1 ,operand_address){
                    page_crossed = true;
                }
            }
        };
        (operand_address,page_crossed)
    }

   
}