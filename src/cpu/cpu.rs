//包含主要的 Cpu 结构体，以及与模拟 6502 CPU 相关的所有方法。这些方法可能包括初始化 CPU（new）、执行指令（step）、读取和写入内存（read 和 write）等。
// src/cpu/cpu.rs

use crate::cpu::instructions::{Instruction, Opcode};
use crate::cpu::addressing_modes::AddressingMode;
use crate::memory::Memory;
use crate::cpu::registers::Registers;

/// 6502 CPU 的结构体
pub struct Cpu {
    pub registers: Registers, // CPU 寄存器
    pub memory: Memory,     // 内存访问接口
    pub interrupt: Interrupt, // 中断类型
}

pub enum Interrupt {
    None,
    NMI,
    IRQ,
    Reset,
}

impl Cpu {
    /// 创建一个新的 CPU 实例
    pub fn new(memory: Memory) -> Self {
        Cpu {
            registers: Registers::new(),
            memory,
            interrupt: Interrupt::None,
        }
    }

    /// 执行一条指令
    pub fn step(&mut self) {
        // 获取操作码
        let opcode = self.memory.read(self.registers.pc);

        // 解码操作码为指令和寻址模式
        let (instruction, addressing_mode) = Opcode::decode(opcode);

        // 根据寻址模式获取操作数
        let (operand, next_address) = addressing_mode.get_operand(&self.memory, &self.registers, self.registers.pc+1);

        // 执行指令
        self.execute(instruction, addressing_mode, operand);

        // 更新程序计数器
        self.registers.pc = next_address;
    }

    /// 执行指令
    fn execute(&mut self, instruction: Instruction, addressing_mode: AddressingMode, operand: Option<u16>) {
        match instruction {
            Instruction::LDA => self.lda(operand.unwrap()),
            Instruction::LDX => self.ldx(operand.unwrap()),
            Instruction::LDY => self.ldy(operand.unwrap()),
            // ... 处理其他指令
            _ => unimplemented!(), // 如果尚未实现的指令，触发未实现错误
        }
    }

    /// LDA 指令实现
    fn lda(&mut self, operand: u16) {
        // TODO: 实现 LDA 指令的逻辑
    }

    /// LDX 指令实现
    fn ldx(&mut self, operand: u16) {
        // TODO: 实现 LDX 指令的逻辑
    }

    /// LDY 指令实现
    fn ldy(&mut self, operand: u16) {
        // TODO: 实现 LDY 指令的逻辑
    }

    // ... 实现其他指令
}
