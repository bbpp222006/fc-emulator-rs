//包含一个用于反汇编 6502 指令的 Disassembler 结构体及其相关方法。这些方法包括反汇编整个程序（disassemble）和反汇编单个指令（disassemble_instruction）等。

// src/cpu/disassembler.rs
use std::fmt::Write;

use crate::cpu::instructions::{Instruction, Opcode};
use crate::cpu::addressing_modes::AddressingMode;

use crate::memory::Memory;
use crate::cpu::registers::{Registers, StatusFlags};
use crate::cpu::Cpu;

/// 反汇编器结构体，用于存储与反汇编过程相关的状态
// Disassembler 结构体，包含对 Cpu 的引用以及反汇编代码的起始和结束地址
pub struct Disassembler<'a> {
    cpu: &'a Cpu,
    start_addr: u16,
    end_addr: u16,
}


impl<'a> Disassembler<'a>{
    /// 创建一个新的反汇编器实例
    // 构造函数，创建一个新的 Disassembler 实例
    pub fn new(cpu: &'a Cpu, start_addr: u16, end_addr: u16) -> Self {
        Disassembler {
            cpu,
            start_addr,
            end_addr,
        }
    }
    
    // 反汇编当前结果
    pub fn get_current_log(&self) -> String{
        format!(
            "{: <48}A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} PPU:{:3}, {:2} CYC:{}",
            self.disassemble_instruction(self.cpu.registers.pc),
            self.cpu.registers.a,
            self.cpu.registers.x,
            self.cpu.registers.y,
            self.cpu.registers.p,
            self.cpu.registers.sp,
            self.cpu.ppu_scanline,
            self.cpu.ppu_cycle,
            self.cpu.cpu_cycle,
        )
    }

    /// 反汇编指定地址处的指令，返回反汇编结果
    pub fn disassemble_instruction(&self, address: u16) -> String {
        let opcode = self.cpu.memory.read(address);
        let (instruction, addressing_mode) = Opcode::decode(opcode);

        // 开始的地址
        let mut output = format!("{:04X}  ", address); 
        let operand_size = addressing_mode.operand_size();

        // opcode 和 根据寻址模式的接下来几位内存
        output.push_str(&format!("{:02X} ", opcode));
        for i in 1..=operand_size {
            output.push_str(&format!("{:02X} ", self.cpu.memory.read(address + i)));
        }
        output = format!("{: <16}", output);

        // 具体指令
        output.push_str(&format!("{:?} ", instruction));

        // 指令运行细节
        use crate::cpu::instructions::Instruction::*;
        match instruction {
            //所有的流程指令
            JMP|JSR|BEQ|BNE|BCS|BCC|BMI|BPL|BVS|BVC|RTS|RTI|BRK => {
                match addressing_mode {
                    AddressingMode::Absolute => {
                        let operand = self.cpu.memory.read_u16(address + 1);
                        output.push_str(&format!(" ${:04X}", operand));
                    }
                    AddressingMode::Indirect => {
                        let operand_address_address = self.cpu.memory.read_u16(address+1) as u16;
                        let low_byte = self.cpu.memory.read(operand_address_address);
                        let high_byte = if operand_address_address & 0x00FF == 0x00FF {
                            self.cpu.memory.read(operand_address_address & 0xFF00)
                        } else {
                            self.cpu.memory.read(operand_address_address + 1)
                        };
                        let operand_address = (high_byte as u16) << 8 | low_byte as u16;
                        let operand = self.cpu.memory.read(address);
                        output.push_str(&format!("(${:04X}) = {:04X}", operand_address_address, operand_address));
                    }
                    AddressingMode::Relative => {
                        let init_offset = self.cpu.memory.read(address+1);
                        let offset = init_offset as i8; // 读取当前地址的值作为偏移量（有符号数）
                        let should_branch = self.cpu.registers.get_flag(StatusFlags::Zero); // 检查 'Z' 标志位的状态
                        let mut operand_address = 0;
                        if should_branch {
                            operand_address = ((address as i32) + 1 + (offset as i32)) as u16;
                        } else {
                            operand_address = (address as i32 + 1 - (offset as i32)) as u16;
                        }
                        let operand = self.cpu.memory.read(operand_address);
                        output.push_str(&format!("${:04X}", operand_address));
                    }
                    _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", instruction, addressing_mode),
                }
            }
            // 所有非跳转指令
            _ => {
                match addressing_mode {
                    AddressingMode::Implied => (),
                    AddressingMode::Immediate => {
                        let operand = self.cpu.memory.read(address+1);
                        output.push_str(&format!("#${:02X}", operand));
                    }
                    AddressingMode::ZeroPage => {
                        let operand_address = self.cpu.memory.read(address+1) as u16;
                        output.push_str(&format!("${:02X} = {:02X}", operand_address, self.cpu.memory.read(operand_address)));
                    }
                    AddressingMode::ZeroPageX => {
                        let operand_address = self.cpu.memory.read(address+1) as u16;
                        output.push_str(&format!("${:02X},X @ {:02X} = {:02X}", operand_address, (operand_address + self.cpu.registers.x as u16) & 0xFF, self.cpu.memory.read((operand_address + self.cpu.registers.x as u16) & 0xFF)));
                    }
                    AddressingMode::ZeroPageY => {
                        let operand_address = self.cpu.memory.read(address+1) as u16;
                        output.push_str(&format!("${:02X},Y @ {:02X} = {:02X}", operand_address, (operand_address + self.cpu.registers.y as u16) & 0xFF, self.cpu.memory.read((operand_address + self.cpu.registers.y as u16) & 0xFF)));
                    }
                    AddressingMode::Absolute => {
                        let operand = self.cpu.memory.read_u16(address+1);
                        output.push_str(&format!("${:04X} = {:02X}", operand, self.cpu.memory.read(operand)));
                    }
                    AddressingMode::AbsoluteX => {
                        let operand = self.cpu.memory.read_u16(address+1);
                        output.push_str(&format!("${:04X},X @ {:04X} = {:02X}", operand, operand + self.cpu.registers.x as u16, self.cpu.memory.read(operand + self.cpu.registers.x as u16)));
                    }
                    AddressingMode::AbsoluteY => {
                        let operand = self.cpu.memory.read_u16(address+1);
                        output.push_str(&format!("${:04X},Y @ {:04X} = {:02X}", operand, operand + self.cpu.registers.y as u16, self.cpu.memory.read(operand + self.cpu.registers.y as u16)));
                    }
                    AddressingMode::Indirect => {
                        let operand_address_address = self.cpu.memory.read_u16(address+1) as u16;
                        let low_byte = self.cpu.memory.read(operand_address_address);
                        let high_byte = if operand_address_address & 0x00FF == 0x00FF {
                            self.cpu.memory.read(operand_address_address & 0xFF00)
                        } else {
                            self.cpu.memory.read(operand_address_address + 1)
                        };
                        let operand_address = (high_byte as u16) << 8 | low_byte as u16;
                        let operand = self.cpu.memory.read(operand_address);
                        output.push_str(&format!("(${:04X}) = {:04X}", operand_address_address, operand_address));
                    }
                    AddressingMode::IndirectX => {
                        let base_address = self.cpu.memory.read(address+1) as u16;
                        let operand_address = self.cpu.memory.read_u16(base_address + self.cpu.registers.x as u16);
                        let operand = self.cpu.memory.read(operand_address);
                        output.push_str(&format!("(${:02X},X) @ {:02X} = {:04X} = {:02X}", base_address, (base_address + self.cpu.registers.x as u16) & 0xFF, operand_address, operand));
                    }
                    AddressingMode::IndirectY => {
                        let base_address_address = self.cpu.memory.read(address+1) as u16;
                        let base_address = self.cpu.memory.read_u16(base_address_address);
                        let operand_address = base_address + self.cpu.registers.y as u16;
                        let operand = self.cpu.memory.read(operand_address);
                        output.push_str(&format!("(${:02X}),Y = {:04X} @ {:04X} = {:02X}", base_address_address, base_address, operand_address, operand));
                    }
                    _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", instruction, addressing_mode),
                }
            }
        }        
        output 
    }


}


