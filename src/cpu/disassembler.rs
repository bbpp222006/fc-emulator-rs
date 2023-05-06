//包含一个用于反汇编 6502 指令的 Disassembler 结构体及其相关方法。这些方法包括反汇编整个程序（disassemble）和反汇编单个指令（disassemble_instruction）等。

// src/cpu/disassembler.rs
use std::fmt::Write;

use crate::cpu::instructions::{Instruction, Opcode};
use crate::cpu::addressing_modes::AddressingMode;

use crate::memory::Memory;
use crate::cpu::registers::Registers;


/// 反汇编器结构体，用于存储与反汇编过程相关的状态
pub struct Disassembler {
    pub memory: Memory, // 存储程序内存的快照
    pub start_address: u16, // 反汇编的开始地址
    pub end_address: u16, // 反汇编的结束地址
}

impl Disassembler {
    /// 创建一个新的反汇编器实例
    pub fn new(memory: Memory, start_address: u16, end_address: u16) -> Self {
        Disassembler {
            memory,
            start_address,
            end_address,
        }
    }

    /// 反汇编整个内存区域，并将结果输出到控制台
    pub fn disassemble(&self) {
        let mut address = self.start_address;

        while address <= self.end_address {
            // 反汇编单条指令
            let (instruction, next_address) = self.disassemble_instruction(address);

            // 打印反汇编结果
            println!("{:04X}: {}", address, instruction);

            // 更新当前地址
            address = next_address;
        }
    }

    /// 反汇编指定地址处的指令，返回反汇编结果和下一条指令的地址
    pub fn disassemble_instruction(&self, address: u16) -> (String, u16) {
        // 获取操作码并解码为指令和寻址模式
        let opcode = self.memory.read(address);
        let (instruction, addressing_mode) = Opcode::decode(opcode);

        // 根据寻址模式获取操作数
        let (operand, next_address) = addressing_mode.get_operand(&self.memory, &Registers::new(), address+1);

        // 将指令和操作数格式化为易读的汇编语言表示
        let disassembly = format_instruction(instruction,addressing_mode, operand);

        (disassembly, next_address)
    }
}

/// 格式化指令和操作数为易读的汇编语言表示
fn format_instruction(instruction: Instruction, addressing_mode: AddressingMode, operand: Option<u16>) -> String {
    let mut result = String::new();
    write!(&mut result, "{:?}", instruction).unwrap();

    match addressing_mode {
        AddressingMode::Implied | AddressingMode::Accumulator => (),
        AddressingMode::Immediate => {
            if let Some(value) = operand {
                write!(&mut result, " #${:02X}", value).unwrap();
            }
        }
        AddressingMode::Absolute => {
            if let Some(value) = operand {
                write!(&mut result, " ${:04X}", value).unwrap();
            }
        }
        AddressingMode::AbsoluteX => {
            if let Some(value) = operand {
                write!(&mut result, " ${:04X},X", value).unwrap();
            }
        }
        AddressingMode::AbsoluteY => {
            if let Some(value) = operand {
                write!(&mut result, " ${:04X},Y", value).unwrap();
            }
        }
        AddressingMode::ZeroPage => {
            if let Some(value) = operand {
                write!(&mut result, " ${:02X}", value).unwrap();
            }
        }
        AddressingMode::ZeroPageX => {
            if let Some(value) = operand {
                write!(&mut result, " ${:02X},X", value).unwrap();
            }
        }
        AddressingMode::ZeroPageY => {
            if let Some(value) = operand {
                write!(&mut result, " ${:02X},Y", value).unwrap();
            }
        }
        AddressingMode::Indirect => {
            if let Some(value) = operand {
                write!(&mut result, " (${:04X})", value).unwrap();
            }
        }
        AddressingMode::IndirectX => {
            if let Some(value) = operand {
                write!(&mut result, " (${:02X},X)", value).unwrap();
            }
        }
        AddressingMode::IndirectY => {
            if let Some(value) = operand {
                write!(&mut result, " (${:02X}),Y", value).unwrap();
            }
        }
        AddressingMode::Relative => {
            if let Some(value) = operand {
                let signed_value = value as i8;
                write!(&mut result, " ${:04X}", (signed_value as u16).wrapping_add(2)).unwrap();
            }
        }
    }
    result
}

