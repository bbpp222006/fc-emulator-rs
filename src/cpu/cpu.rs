//包含主要的 Cpu 结构体，以及与模拟 6502 CPU 相关的所有方法。这些方法可能包括初始化 CPU（new）、执行指令（step）、读取和写入内存（read 和 write）等。
// src/cpu/cpu.rs

use crate::cpu::instructions::{Instruction, Opcode};
use crate::cpu::addressing_modes::AddressingMode;
use crate::memory::Memory;
use crate::cpu::registers::{Registers,StatusFlags};

use std::fmt::Write;


/// 6502 CPU 的结构体
pub struct Cpu {
    pub registers: Registers, // CPU 寄存器
    pub memory: Memory,     // 内存访问接口
    pub interrupt: Interrupt, // 中断类型
    pub ppu_scanline :u64,
    pub ppu_cycle:u64,
    pub cpu_cycle: u64,
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
            ppu_scanline : 0,
            ppu_cycle: 0,
            cpu_cycle: 7,
        }
    }


    /// 执行一条指令
    pub fn step(&mut self) {

        // 获取、记录操作码
        let opcode = self.memory.read(self.registers.pc);
   
        // 解码操作码为指令和寻址模式
        let (instruction, addressing_mode) = Opcode::decode(opcode);

        self.execute(instruction, addressing_mode);   

    }

        // 反汇编当前结果
    pub fn get_current_log(&self) -> String{
        format!(
            "{: <48}A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} PPU:{:3}, {:2} CYC:{}",
            self.disassemble_instruction(self.registers.pc),
            self.registers.a,
            self.registers.x,
            self.registers.y,
            self.registers.p,
            self.registers.sp,
            self.ppu_scanline,
            self.ppu_cycle,
            self.cpu_cycle,
        )
    }

    /// 反汇编指定地址处的指令，返回反汇编结果
    pub fn disassemble_instruction(&self, address: u16) -> String {
        let opcode = self.memory.read(address);
        let (instruction, addressing_mode) = Opcode::decode(opcode);

        // 开始的地址
        let mut output = format!("{:04X}  ", address); 
        let operand_size = addressing_mode.operand_size();

        // opcode 和 根据寻址模式的接下来几位内存
        output.push_str(&format!("{:02X} ", opcode));
        for i in 1..=operand_size-1 {
            output.push_str(&format!("{:02X} ", self.memory.read(address + i)));
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
                    AddressingMode::Implied => (),
                    AddressingMode::Absolute => {
                        let operand = self.memory.read_u16(address + 1);
                        output.push_str(&format!("${:04X}", operand));
                    }
                    AddressingMode::Indirect => {
                        let operand_address_address = self.memory.read_u16(address+1) as u16;
                        let low_byte = self.memory.read(operand_address_address);
                        let high_byte = if operand_address_address & 0x00FF == 0x00FF {
                            self.memory.read(operand_address_address & 0xFF00)
                        } else {
                            self.memory.read(operand_address_address + 1)
                        };
                        let operand_address = (high_byte as u16) << 8 | low_byte as u16;
                        output.push_str(&format!("(${:04X}) = {:04X}", operand_address_address, operand_address));
                    }
                    AddressingMode::Relative => {
                        let init_offset = self.memory.read(address+1);
                        let offset = init_offset as i8; // 读取当前地址的值作为偏移量（有符号数）
                        let operand_address = ((address+1) as i32 + 1 + (offset as i32)) as u16;
                        output.push_str(&format!("${:04X}", operand_address));
                    }
                    _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", instruction, addressing_mode),
                }
            }
            // 所有非跳转指令
            _ => {
                match addressing_mode {
                    AddressingMode::Implied => (),
                    AddressingMode::Accumulator => output.push_str("A"),
                    AddressingMode::Immediate => {
                        let operand = self.memory.read(address+1);
                        output.push_str(&format!("#${:02X}", operand));
                    }
                    AddressingMode::ZeroPage => {
                        let operand_address = self.memory.read(address+1) as u16;
                        output.push_str(&format!("${:02X} = {:02X}", operand_address, self.memory.read(operand_address)));
                    }
                    AddressingMode::ZeroPageX => {
                        let operand_address = self.memory.read(address+1) as u16;
                        output.push_str(&format!("${:02X},X @ {:02X} = {:02X}", operand_address, (operand_address + self.registers.x as u16) & 0xFF, self.memory.read((operand_address + self.registers.x as u16) & 0xFF)));
                    }
                    AddressingMode::ZeroPageY => {
                        let operand_address = self.memory.read(address+1) as u16;
                        output.push_str(&format!("${:02X},Y @ {:02X} = {:02X}", operand_address, (operand_address + self.registers.y as u16) & 0xFF, self.memory.read((operand_address + self.registers.y as u16) & 0xFF)));
                    }
                    AddressingMode::Absolute => {
                        let operand = self.memory.read_u16(address+1);
                        output.push_str(&format!("${:04X} = {:02X}", operand, self.memory.read(operand)));
                    }
                    AddressingMode::AbsoluteX => {
                        let operand = self.memory.read_u16(address+1);
                        output.push_str(&format!("${:04X},X @ {:04X} = {:02X}", operand, operand + self.registers.x as u16, self.memory.read(operand + self.registers.x as u16)));
                    }
                    AddressingMode::AbsoluteY => {
                        let base_address = self.memory.read_u16(address+1);
                        let operand_address = base_address.wrapping_add(self.registers.y as u16);
                        let operand = self.memory.read(operand_address);
                        output.push_str(&format!("${:04X},Y @ {:04X} = {:02X}", base_address, operand_address, operand));
                    }
                    AddressingMode::Indirect => {
                        let operand_address_address = self.memory.read_u16(address+1) as u16;
                        let low_byte = self.memory.read(operand_address_address);
                        let high_byte = if operand_address_address & 0x00FF == 0x00FF {
                            self.memory.read(operand_address_address & 0xFF00)
                        } else {
                            self.memory.read(operand_address_address + 1)
                        };
                        let operand_address = (high_byte as u16) << 8 | low_byte as u16;
                        let operand = self.memory.read(operand_address);
                        output.push_str(&format!("(${:04X}) = {:04X}", operand_address_address, operand_address));
                    }
                    AddressingMode::IndirectX => {
                        let base_address = self.memory.read(address+1);
                        let operand_address = self.memory.read_u16_z(base_address.wrapping_add(self.registers.x));
                        let operand = self.memory.read(operand_address);
                        output.push_str(&format!("(${:02X},X) @ {:02X} = {:04X} = {:02X}", base_address, (base_address as u16 + self.registers.x as u16) & 0xFF, operand_address, operand));
                    }
                    AddressingMode::IndirectY => {
                        let base_address_address = self.memory.read(address+1);
                        let base_address = self.memory.read_u16_z(base_address_address);
                        let operand_address = base_address.wrapping_add(self.registers.y as u16) ;
                        let operand = self.memory.read(operand_address);
                        output.push_str(&format!("(${:02X}),Y = {:04X} @ {:04X} = {:02X}", base_address_address, base_address, operand_address, operand));
                    }
                    _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", instruction, addressing_mode),
                }
            }
        }        
        output 
    }


    /// 执行指令
    fn execute(&mut self, instruction: Instruction, addressing_mode: AddressingMode) {
        // 根据寻址模式获取操作数所在的地址
        let (operand_address,page_crossed) = addressing_mode.get_operand(&self.memory, &self.registers, self.registers.pc + 1);

        match instruction {
            Instruction::JMP => self.jmp(addressing_mode,operand_address),
            Instruction::LDX => self.ldx(addressing_mode,operand_address),
            Instruction::STX => self.stx(addressing_mode,operand_address),
            Instruction::JSR => self.jsr(addressing_mode,operand_address),
            Instruction::NOP => self.nop(),
            Instruction::SEC => self.sec(),
            Instruction::BCS => self.bcs(addressing_mode,operand_address,page_crossed),
            Instruction::CLC => self.clc(),
            Instruction::BCC => self.bcc(operand_address,page_crossed),
            Instruction::LDA => self.lda(addressing_mode,operand_address,page_crossed),
            Instruction::BEQ => self.beq(addressing_mode,operand_address,page_crossed),
            Instruction::BNE => self.bne(addressing_mode,operand_address,page_crossed),
            Instruction::STA => self.sta(addressing_mode,operand_address,page_crossed),
            Instruction::BIT => self.bit(addressing_mode,operand_address,page_crossed),
            Instruction::BVS => self.bvs(addressing_mode,operand_address,page_crossed),
            Instruction::BVC => self.bvc(addressing_mode,operand_address,page_crossed),
            Instruction::BPL => self.bpl(addressing_mode,operand_address,page_crossed),
            Instruction::RTS => self.rts(addressing_mode),
            Instruction::SEI => self.sei(addressing_mode),
            Instruction::SED => self.sed(addressing_mode),
            Instruction::PHP => self.php(addressing_mode),
            Instruction::PLA => self.pla(addressing_mode),
            Instruction::AND => self.and(addressing_mode,operand_address,page_crossed),
            Instruction::CMP => self.cmp(addressing_mode,operand_address,page_crossed),
            Instruction::CLD => self.cld(addressing_mode),
            Instruction::PHA => self.pha(addressing_mode),
            Instruction::PLP => self.plp(addressing_mode),
            Instruction::BMI => self.bmi(addressing_mode,operand_address,page_crossed),
            Instruction::ORA => self.ora(addressing_mode,operand_address,page_crossed),
            Instruction::CLV => self.clv(addressing_mode),
            Instruction::EOR => self.eor(addressing_mode,operand_address,page_crossed),
            Instruction::ADC => self.adc(addressing_mode,operand_address,page_crossed),
            Instruction::LDY => self.ldy(addressing_mode,operand_address,page_crossed),
            Instruction::CPY => self.cpy(addressing_mode,operand_address,page_crossed),
            Instruction::CPX => self.cpx(addressing_mode,operand_address,page_crossed),
            Instruction::SBC => self.sbc(addressing_mode,operand_address,page_crossed),
            Instruction::INY => self.iny(addressing_mode),
            Instruction::INX => self.inx(addressing_mode),
            Instruction::DEY => self.dey(addressing_mode),
            Instruction::DEX => self.dex(addressing_mode),
            Instruction::TAY => self.tay(addressing_mode),
            Instruction::TAX => self.tax(addressing_mode),
            Instruction::TYA => self.tya(addressing_mode),
            Instruction::TXA => self.txa(addressing_mode),
            Instruction::TSX => self.tsx(addressing_mode),
            Instruction::TXS => self.txs(addressing_mode),
            Instruction::RTI => self.rti(addressing_mode),
            Instruction::LSR => self.lsr(addressing_mode,operand_address),
            Instruction::ASL => self.asl(addressing_mode,operand_address),
            Instruction::ROR => self.ror(addressing_mode,operand_address),
            Instruction::ROL => self.rol(addressing_mode,operand_address),
            Instruction::STY => self.sty(addressing_mode,operand_address),
            Instruction::INC => self.inc(addressing_mode,operand_address),
            Instruction::DEC => self.dec(addressing_mode,operand_address),
            // ... 处理其他指令
            _ => unimplemented!(), // 如果尚未实现的指令，触发未实现错误
        }
    }

    fn stack_push(&mut self, value: u8) {

        // 堆栈基地址是 0x0100
        const STACK_BASE: u16 = 0x0100;
    
        // 计算堆栈当前位置的地址，将堆栈指针（SP）加上基地址
        let stack_address = 0x0100 + self.registers.sp as u16;
    
        // 将值写入当前堆栈位置
        self.memory.write(stack_address, value);
    
        // 更新堆栈指针（SP），将其减 1，指向下一个可用位置
        self.registers.sp = self.registers.sp.wrapping_sub(1);
    }
    
    fn stack_pop(&mut self) -> u8 {

        // 堆栈基地址是 0x0100
        const STACK_BASE: u16 = 0x0100;

        // 堆栈指针（SP）加 1
        self.registers.sp = self.registers.sp.wrapping_add(1);
    
        // 堆栈的基地址为 0x0100
        let stack_address = STACK_BASE + self.registers.sp as u16;
    
        // 从内存中读取位于 stack_address 的值
        let value = self.memory.read(stack_address);
    
        // 返回弹出的值
        value
    }

    fn check_zsflag(&mut self, register: u8) {
        // 检查零标志（Zero flag）
        if register == 0 {
            self.registers.set_flag(StatusFlags::Zero, true);
        } else {
            self.registers.set_flag(StatusFlags::Zero, false);
        }
    
        // 检查负数标志（Sign 或 Negative flag）
        if register & 0x80 != 0 {
            self.registers.set_flag(StatusFlags::Negative, true);
        } else {
            self.registers.set_flag(StatusFlags::Negative, false);
        }
    }

    fn cld(&mut self,addressing_mode: AddressingMode) {
        // 清除十进制模式标志（Decimal flag）
        self.registers.set_flag(StatusFlags::DecimalMode, false);
        self.registers.pc += addressing_mode.operand_size();
        self.cpu_cycle += 2;
    }

    fn cmp(&mut self,addressing_mode: AddressingMode,operand_address: u16,page_crossed: bool) {
        // 从内存中读取操作数
        let operand = self.memory.read(operand_address);
    
        // 将操作数与寄存器 A 进行比较
        let result = self.registers.a.wrapping_sub(operand);
    
        // 检查零标志（Zero flag）和负数标志（Sign 或 Negative flag）
        self.check_zsflag(result);
    
        // 检查进位标志（Carry flag）
        if self.registers.a >= operand {
            self.registers.set_flag(StatusFlags::Carry, true);
        } else {
            self.registers.set_flag(StatusFlags::Carry, false);
        }
    
        // 增加所需的 CPU 周期
        match addressing_mode {
            AddressingMode::Immediate => self.cpu_cycle += 2,
            AddressingMode::ZeroPage => self.cpu_cycle += 3,
            AddressingMode::ZeroPageX|AddressingMode::Absolute|AddressingMode::AbsoluteX|AddressingMode::AbsoluteY=>self.cpu_cycle += 4,
            AddressingMode::IndirectX=>self.cpu_cycle += 6,
            AddressingMode::IndirectY=>self.cpu_cycle += 5,
            _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::CMP, addressing_mode),
        }
        if page_crossed{
            self.cpu_cycle += 1;
        }
        self.registers.pc+=addressing_mode.operand_size();
    }

    fn and(&mut self,addressing_mode: AddressingMode,operand_address: u16,page_crossed: bool) {
        // 从内存中读取操作数
        let operand = self.memory.read(operand_address);
    
        // 将操作数与寄存器 A 进行 AND 运算
        self.registers.a &= operand;
    
        // 检查零标志（Zero flag）和负数标志（Sign 或 Negative flag）
        self.check_zsflag(self.registers.a);
        
        // 增加所需的 CPU 周期
        match addressing_mode {
            AddressingMode::Immediate => self.cpu_cycle += 2,
            AddressingMode::ZeroPage => self.cpu_cycle += 3,
            AddressingMode::ZeroPageX|AddressingMode::Absolute|AddressingMode::AbsoluteX|AddressingMode::AbsoluteY=>self.cpu_cycle += 4,
            AddressingMode::IndirectX=>self.cpu_cycle += 6,
            AddressingMode::IndirectY=>self.cpu_cycle += 5,
            _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::AND, addressing_mode),
        }
        // 如果跨页了，增加一个周期
        if page_crossed {
            self.cpu_cycle += 1;
        }
    
        self.registers.pc += addressing_mode.operand_size();
    }


    fn pla(&mut self,addressing_mode: AddressingMode) {
        // 从堆栈中弹出值
        let value = self.stack_pop();
    
        // 将值写入寄存器 A
        self.registers.a = value;
    
        self.check_zsflag(self.registers.a);
    
        // 增加所需的 CPU 周期
        self.cpu_cycle += 4;
    
        self.registers.pc += addressing_mode.operand_size();
    }

    fn php(&mut self,addressing_mode: AddressingMode) {
        // 将状态寄存器的值压入堆栈
        self.stack_push(self.registers.p|StatusFlags::BreakCommand as u8|StatusFlags::Unused as u8);
    
        // 增加所需的 CPU 周期
        self.cpu_cycle += 3;
    
        self.registers.pc += addressing_mode.operand_size();
    }

    fn sed(&mut self,addressing_mode: AddressingMode) {
        // 设置状态寄存器的 D 标志位
        self.registers.set_flag(StatusFlags::DecimalMode, true);

        // 增加所需的 CPU 周期
        self.cpu_cycle += 2;

        self.registers.pc += addressing_mode.operand_size();
    }

    fn sei(&mut self,addressing_mode: AddressingMode) {
        // 设置状态寄存器的 I 标志位
        self.registers.set_flag(StatusFlags::InterruptDisable, true);

        // 增加所需的 CPU 周期
        self.cpu_cycle += 2;

        self.registers.pc += addressing_mode.operand_size();
    }

    fn rts(&mut self, addressing_mode: AddressingMode) {
        // 从堆栈中弹出返回地址的低字节（低 8 位）
        let low_byte = self.stack_pop() as u16;

        // 从堆栈中弹出返回地址的高字节（高 8 位）
        let high_byte = self.stack_pop() as u16;

        // 将返回地址的高字节和低字节组合成完整的 16 位地址
        let return_address = (high_byte << 8) | low_byte;

        // 将程序计数器（PC）设置为返回地址 + 1，以返回调用 JSR 指令之前的指令
        self.registers.pc = return_address.wrapping_add(1);

        // 增加所需的 CPU 周期
        self.cpu_cycle += 6;
    }

    fn bpl(&mut self,addressing_mode: AddressingMode,operand_address:u16,page_crossed:bool) {
        self.cpu_cycle += 2;
        if !self.registers.get_flag(StatusFlags::Negative) {
            self.cpu_cycle += 1;
            self.registers.pc = operand_address;
            if page_crossed {
                self.cpu_cycle += 1;
            }
        } else {
            self.registers.pc += addressing_mode.operand_size();
        }
    }

    fn bvc(&mut self,addressing_mode: AddressingMode,operand_address:u16,page_crossed:bool) {
        self.cpu_cycle += 2;
        if !self.registers.get_flag(StatusFlags::Overflow) {
            self.cpu_cycle += 1;
            self.registers.pc = operand_address;
            if page_crossed {
                self.cpu_cycle += 1;
            }
        } else {
            self.registers.pc += addressing_mode.operand_size();
        }
    }

    fn bvs(&mut self,addressing_mode: AddressingMode,operand_address:u16,page_crossed:bool) {
        self.cpu_cycle += 2;
        if self.registers.get_flag(StatusFlags::Overflow) {
            self.cpu_cycle += 1;
            self.registers.pc = operand_address;
            if page_crossed {
                self.cpu_cycle += 1;
            }
        } else {
            self.registers.pc += addressing_mode.operand_size();
        }
    }

    fn bit(&mut self,addressing_mode: AddressingMode,operand_address:u16,page_crossed:bool) {
        let operand = self.memory.read(operand_address);
        let value = self.registers.a & operand;
        self.registers.set_flag(StatusFlags::Zero, value==0);
        self.registers.set_flag(StatusFlags::Overflow, operand & 0x40 != 0);
        self.registers.set_flag(StatusFlags::Negative, operand & 0x80 != 0);
        self.registers.pc += addressing_mode.operand_size();
        match addressing_mode {
            AddressingMode::ZeroPage => {
                self.cpu_cycle += 3;
            }
            AddressingMode::Absolute => {
                self.cpu_cycle += 4;
            }
            _ => {
                panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::BIT, addressing_mode);
            }
        }
    }

    fn sta(&mut self,addressing_mode: AddressingMode,operand_address:u16,page_crossed:bool) {
        let value = self.registers.a;
        self.memory.write(operand_address, value);
        self.registers.pc += addressing_mode.operand_size();
        match addressing_mode {
            AddressingMode::ZeroPage => {
                self.cpu_cycle += 3;
            }
            AddressingMode::ZeroPageX|AddressingMode::Absolute => {
                self.cpu_cycle += 4;
            }
            AddressingMode::AbsoluteX|AddressingMode::AbsoluteY => {
                self.cpu_cycle += 5;
            }
            AddressingMode::IndirectX|AddressingMode::IndirectY => {
                self.cpu_cycle += 6;
            }
            _ => {
                panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::STA, addressing_mode);
            }
        }
            
    }
    
    fn bne(&mut self,addressing_mode: AddressingMode,operand_address:u16,page_crossed:bool) {
        if !self.registers.get_flag(StatusFlags::Zero) {
            self.registers.pc = operand_address;
            self.cpu_cycle += 1;
            if page_crossed {
                self.cpu_cycle += 1;
            }
        } else {
            self.registers.pc += 2;
        }
        self.cpu_cycle += 2;
    }

    // BEQ 指令实现
    fn beq(&mut self,addressing_mode: AddressingMode,operand_address:u16,page_crossed:bool) {
        if self.registers.get_flag(StatusFlags::Zero) {
            self.registers.pc = operand_address;
            self.cpu_cycle += 1;
            if page_crossed {
                self.cpu_cycle += 1;
            }
        } else {
            self.registers.pc += 2;
        }
        self.cpu_cycle += 2;
    }

    // LDA 指令实现
    fn lda(&mut self,addressing_mode: AddressingMode,operand_address:u16,page_crossed:bool) {
        let operand = self.memory.read(operand_address);
        self.registers.pc+=addressing_mode.operand_size();
        self.registers.a = operand;
        match addressing_mode {
            AddressingMode::Immediate => {
                self.cpu_cycle += 2;
            },
            AddressingMode::ZeroPage => {
                self.cpu_cycle += 3;
            },
            AddressingMode::ZeroPageX|AddressingMode::Absolute|AddressingMode::AbsoluteX|AddressingMode::AbsoluteY => {
                self.cpu_cycle += 4;
                if page_crossed {
                    self.cpu_cycle += 1;
                }
            },
            AddressingMode::IndirectX => {
                self.cpu_cycle += 6;
            },
            AddressingMode::IndirectY => {
                self.cpu_cycle += 5;
                if page_crossed {
                    self.cpu_cycle += 1;
                }
            },
            _ => unimplemented!(),
        }
        
        self.check_zsflag(self.registers.a)
        }

    // BCC 指令实现
    fn bcc(&mut self,operand_address:u16,page_crossed:bool) {
        // 如果进位标志（Carry）为 0，则跳转到指定地址
            if !self.registers.get_flag(StatusFlags::Carry) {
                self.registers.pc = operand_address;
                self.cpu_cycle += 1;
                //在页面边界交叉时 +1s
                if page_crossed {
                    self.cpu_cycle += 1;
                }
            } else {
                self.registers.pc += 2;
            }
            self.cpu_cycle += 2;
        }

    // CLC 指令实现
    fn clc(&mut self) {
        self.registers.set_flag(StatusFlags::Carry, false);
        self.registers.pc += 1;
        self.cpu_cycle += 2;
    }

    // BCS 指令实现
    fn bcs(&mut self,addressing_mode: AddressingMode,operand_address:u16,page_crossed:bool) {
        // 如果进位标志（Carry）为 1，则跳转到指定地址
            if self.registers.get_flag(StatusFlags::Carry) {
                self.registers.pc = operand_address;
                self.cpu_cycle += 1;
                //在页面边界交叉时 +1s
                if page_crossed {
                    self.cpu_cycle += 1;
                }
            } else {
                self.registers.pc += 2;
            }
            self.cpu_cycle += 2;
        }
        
    /// SEC 指令实现
    fn sec(&mut self) {
        self.registers.set_flag(StatusFlags::Carry, true);
        self.registers.pc += 1;
        self.cpu_cycle += 2;
    }

    /// NOP
    fn nop(&mut self) {
        self.registers.pc += 1;
        self.cpu_cycle += 2;
    }
    /// JSR 指令实现
    fn jsr(&mut self,addressing_mode: AddressingMode,operand_address:u16) {
        match addressing_mode {
            AddressingMode::Absolute => {
                // JSR 操作的目标地址是绝对地址
                let target_address = operand_address;

                // 获取当前程序计数器（PC）值，并将其减 1，以指向 JSR 指令后的下一条指令
                let return_address = self.registers.pc.wrapping_add(2);

                // 将 return_address 的高字节（高 8 位）和低字节（低 8 位）分开
                let high_byte = ((return_address >> 8) & 0xFF) as u8;
                let low_byte = (return_address & 0xFF) as u8;

                // 将 return_address 的高字节和低字节推入堆栈
                self.stack_push(high_byte);
                self.stack_push(low_byte);

                // 将程序计数器（PC）设置为目标地址，以开始执行子程序
                self.registers.pc = target_address;

                self.cpu_cycle += 6;
            },
            _ => unimplemented!(),
        }
    }

    /// STX 指令实现
    fn stx(&mut self,addressing_mode: AddressingMode,operand_address:u16) {
        let operand = self.memory.read(operand_address);
        match addressing_mode {
            AddressingMode::ZeroPage => {
                self.memory.write(operand_address, self.registers.x);
                self.registers.pc += 2;
                self.cpu_cycle += 3;
            },
            AddressingMode::ZeroPageY => {
                self.memory.write(operand_address, self.registers.x);
                self.registers.pc += 2;
                self.cpu_cycle += 4;
            },
            AddressingMode::Absolute => {
                self.memory.write(operand_address, self.registers.x);
                self.registers.pc += 3;
                self.cpu_cycle += 4;
            },
            _ => unimplemented!(),
        }
    }

    /// LDX 指令实现
    fn ldx(&mut self,addressing_mode: AddressingMode,operand_address:u16) {
        let operand = self.memory.read(operand_address);
        match addressing_mode {
            AddressingMode::Immediate => {
                self.registers.x = operand;
                self.registers.pc += 2;
                self.cpu_cycle += 2;
            },
            AddressingMode::ZeroPage => {
                self.registers.x = operand;
                self.registers.pc += 2;
                self.cpu_cycle += 3;
            },
            AddressingMode::ZeroPageY => {
                self.registers.x = operand;
                self.registers.pc += 2;
                self.cpu_cycle += 4;
            },
            AddressingMode::Absolute => {
            self.registers.x = operand;
            self.registers.pc += 3;
            self.cpu_cycle += 4;
            },
            AddressingMode::AbsoluteY => {
                self.registers.x = operand;
                self.registers.pc += 3;
                //在页面边界交叉时 +1s
                if operand_address & 0xFF00 != (operand_address + self.registers.y as u16) & 0xFF00 {
                    self.cpu_cycle += 1;
                }
                self.cpu_cycle += 4;
            },
            _ => unimplemented!(),
        }
        // 更新零（Zero）标志
        self.registers.set_flag(StatusFlags::Zero, self.registers.x == 0);
        // 更新负（Negative）标志
        self.registers.set_flag(StatusFlags::Negative, self.registers.x & 0x80 != 0);
    }


    /// JMP 指令实现
    fn jmp(&mut self, addressing_mode: AddressingMode,operand_address: u16) {
        self.registers.pc = operand_address;
        match addressing_mode {
            AddressingMode::Absolute => {
                self.cpu_cycle += 3;
            },
            AddressingMode::Indirect => {
                self.cpu_cycle += 5;
            },
            _ => unimplemented!(),
            
        }
    }

    fn pha (&mut self,addressing_mode:AddressingMode) {
        self.stack_push(self.registers.a);
        self.registers.pc += addressing_mode.operand_size();
        self.cpu_cycle += 3;
    }

    fn plp (&mut self,addressing_mode:AddressingMode) {
        self.registers.p = self.stack_pop();
        self.registers.set_flag(StatusFlags::BreakCommand, false);
        self.registers.set_flag(StatusFlags::Unused, true);
        self.registers.pc += addressing_mode.operand_size();
        self.cpu_cycle += 4;
    }

    fn bmi(&mut self,addressing_mode:AddressingMode,operand_address:u16,page_crossed: bool) {
        if self.registers.get_flag(StatusFlags::Negative) {
            self.registers.pc = operand_address;
            self.cpu_cycle += 1;   
            if page_crossed {
                self.cpu_cycle += 1;
            }
        } else {
            self.registers.pc += addressing_mode.operand_size();
        }
        self.cpu_cycle += 2;
    }

    fn ora (&mut self,addressing_mode:AddressingMode,operand_address:u16,page_crossed: bool) {
        let operand = self.memory.read(operand_address);
        self.registers.a |= operand;
        self.registers.pc += addressing_mode.operand_size();
        self.check_zsflag(self.registers.a);
        match addressing_mode {
            AddressingMode::Immediate => {
                self.cpu_cycle += 2;
            },
            AddressingMode::ZeroPage => {
                self.cpu_cycle += 3;
            },
            AddressingMode::ZeroPageX|AddressingMode::Absolute => {
                self.cpu_cycle += 4;
            },
            AddressingMode::AbsoluteX|AddressingMode::AbsoluteY => {
                self.cpu_cycle += 4;
                if page_crossed {
                    self.cpu_cycle += 1;
                }
            },
            AddressingMode::IndirectX => {
                self.cpu_cycle += 6;
            },
            AddressingMode::IndirectY => {
                self.cpu_cycle += 5;
            },
            _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::ORA, addressing_mode),
        }
    }

    fn clv(&mut self,addressing_mode:AddressingMode) {
        self.registers.set_flag(StatusFlags::Overflow, false);
        self.registers.pc += addressing_mode.operand_size();
        self.cpu_cycle += 2;
    }

    fn eor(&mut self,addressing_mode:AddressingMode,operand_address:u16,page_crossed: bool) {
        let operand = self.memory.read(operand_address);
        self.registers.a ^= operand;
        self.registers.pc += addressing_mode.operand_size();
        self.check_zsflag(self.registers.a);
        match addressing_mode {
            AddressingMode::Immediate => {
                self.cpu_cycle += 2;
            },
            AddressingMode::ZeroPage => {
                self.cpu_cycle += 3;
            },
            AddressingMode::ZeroPageX|AddressingMode::Absolute => {
                self.cpu_cycle += 4;
            },
            AddressingMode::AbsoluteX|AddressingMode::AbsoluteY => {
                self.cpu_cycle += 4;
                if page_crossed {
                    self.cpu_cycle += 1;
                }
            },
            AddressingMode::IndirectX => {
                self.cpu_cycle += 6;
            },
            AddressingMode::IndirectY => {
                self.cpu_cycle += 5;
                if page_crossed {
                    self.cpu_cycle += 1;
                }
            },
            _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::EOR, addressing_mode),
        }
    }

    fn adc(&mut self, addressing_mode: AddressingMode, operand_address: u16, page_crossed: bool) {
        // 根据寻址模式获取操作数值
        let operand = self.memory.read(operand_address);
    
        // 获取累加器的当前值
        let a = self.registers.a;
    
        // 获取当前进位标志（Carry）的值
        let carry = self.registers.get_flag(StatusFlags::Carry) as u8;
    
        // 计算 ADC 操作的结果
        let result = a.wrapping_add(operand).wrapping_add(carry);
    
        // 更新标志寄存器
        self.registers.set_flag(StatusFlags::Carry, (a as u16 + operand as u16 + carry as u16) > 0xFF);
        self.registers.set_flag(
            StatusFlags::Overflow,
            (((a ^ result) & (operand ^ result)) & 0x80) != 0
        );
        self.check_zsflag(result as u8);
    
        // 将计算结果存入累加器
        self.registers.a = result as u8;
    
        // 根据寻址模式和是否跨页来更新 CPU 周期
        self.cpu_cycle += match addressing_mode {
            AddressingMode::Immediate => 2,
            AddressingMode::ZeroPage => 3,
            AddressingMode::ZeroPageX => 4,
            AddressingMode::Absolute => 4,
            AddressingMode::AbsoluteX => {
                if page_crossed {
                    5
                } else {
                    4
                }
            }
            AddressingMode::AbsoluteY => {
                if page_crossed {
                    5
                } else {
                    4
                }
            }
            AddressingMode::IndirectX => 6,
            AddressingMode::IndirectY => {
                if page_crossed {
                    6
                } else {
                    5
                }
            }
            _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::ADC, addressing_mode),
        };
        self.registers.pc+=addressing_mode.operand_size();
    }
    
    fn ldy(&mut self, addressing_mode: AddressingMode, operand_address: u16, page_crossed: bool) {
        // 根据寻址模式获取操作数值
        let operand = self.memory.read(operand_address);
    
        // 将操作数存入 Y 寄存器
        self.registers.y = operand;

        // 更新标志寄存器
        self.check_zsflag(self.registers.y);
    
        // 根据寻址模式和是否跨页来更新 CPU 周期
        self.cpu_cycle += match addressing_mode {
            AddressingMode::Immediate => 2,
            AddressingMode::ZeroPage => 3,
            AddressingMode::ZeroPageX => 4,
            AddressingMode::Absolute => 4,
            AddressingMode::AbsoluteX => {
                if page_crossed {
                    5
                } else {
                    4
                }
            }
            _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::LDY, addressing_mode),
        };
        self.registers.pc+=addressing_mode.operand_size();
    }

    fn cpy(&mut self, addressing_mode: AddressingMode, operand_address: u16, page_crossed: bool) {
        // 根据寻址模式获取操作数值
        let operand = self.memory.read(operand_address);
    
        // 获取 Y 寄存器的当前值
        let y = self.registers.y;
    
        // 计算 CPY 操作的结果
        let result = y.wrapping_sub(operand);
    
        // 更新标志寄存器
        self.registers.set_flag(StatusFlags::Carry, y >= operand);
        self.check_zsflag(result);
    
        // 根据寻址模式和是否跨页来更新 CPU 周期
        self.cpu_cycle += match addressing_mode {
            AddressingMode::Immediate => 2,
            AddressingMode::ZeroPage => 3,
            AddressingMode::Absolute => 4,
            _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::CPY, addressing_mode),
        };
        self.registers.pc+=addressing_mode.operand_size();
    }

    fn cpx(&mut self, addressing_mode: AddressingMode, operand_address: u16, page_crossed: bool) {
        // 根据寻址模式获取操作数值
        let operand = self.memory.read(operand_address);
    
        // 获取 X 寄存器的当前值
        let x = self.registers.x;
    
        // 计算 CPX 操作的结果
        let result = x.wrapping_sub(operand);
    
        // 更新标志寄存器
        self.registers.set_flag(StatusFlags::Carry, x >= operand);
        self.check_zsflag(result);
    
        // 根据寻址模式和是否跨页来更新 CPU 周期
        self.cpu_cycle += match addressing_mode {
            AddressingMode::Immediate => 2,
            AddressingMode::ZeroPage => 3,
            AddressingMode::Absolute => 4,
            _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::CPX, addressing_mode),
        };
        self.registers.pc+=addressing_mode.operand_size();
    }

    fn sbc(&mut self, addressing_mode: AddressingMode, operand_address: u16, page_crossed: bool) {
        let acc = self.registers.a;
        let operand = self.memory.read(operand_address);

        // 获取借位标志
        let borrow = if self.registers.get_flag(StatusFlags::Carry) { 0 } else { 1 };

        // 执行减法操作
        let result = acc.wrapping_sub(operand).wrapping_sub(borrow);

        // 更新状态寄存器
        self.registers.set_flag(StatusFlags::Carry, (acc as i16 - operand as i16 - borrow as i16) >= 0);
        self.registers.set_flag(StatusFlags::Zero, result == 0);
        self.registers.set_flag(StatusFlags::Negative, result & 0x80 != 0);
        self.registers.set_flag(StatusFlags::Overflow, (((acc ^ operand) & (acc ^ result)) & 0x80) != 0);

        // 更新累加器寄存器
        self.registers.a = result;
    
        // 根据寻址模式和是否跨页来更新 CPU 周期
        self.cpu_cycle += match addressing_mode {
            AddressingMode::Immediate => 2,
            AddressingMode::ZeroPage => 3,
            AddressingMode::ZeroPageX => 4,
            AddressingMode::Absolute => 4,
            AddressingMode::AbsoluteX => {
                if page_crossed {
                    5
                } else {
                    4
                }
            }
            AddressingMode::AbsoluteY => {
                if page_crossed {
                    5
                } else {
                    4
                }
            }
            AddressingMode::IndirectX => 6,
            AddressingMode::IndirectY => {
                if page_crossed {
                    6
                } else {
                    5
                }
            }
            _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::ADC, addressing_mode),
        };
        self.registers.pc+=addressing_mode.operand_size();
    }

    fn iny(&mut self, addressing_mode: AddressingMode) {
        // 更新 Y 寄存器
        self.registers.y =self.registers.y.wrapping_add(1);
    
        // 更新标志寄存器
        self.check_zsflag(self.registers.y);
    
        // 根据寻址模式和是否跨页来更新 CPU 周期
        self.cpu_cycle += match addressing_mode {
            AddressingMode::Implied => 2,
            _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::INY, addressing_mode),
        };
        self.registers.pc+=addressing_mode.operand_size();
    }

    fn inx(&mut self, addressing_mode: AddressingMode) {
        // 更新 X 寄存器
        self.registers.x = self.registers.x.wrapping_add(1);
    
        // 更新标志寄存器
        self.check_zsflag(self.registers.x);
    
        // 根据寻址模式和是否跨页来更新 CPU 周期
        self.cpu_cycle += match addressing_mode {
            AddressingMode::Implied => 2,
            _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::INX, addressing_mode),
        };
        self.registers.pc+=addressing_mode.operand_size();
    }

    fn dey(&mut self, addressing_mode: AddressingMode) {
        // 更新 Y 寄存器
        self.registers.y= self.registers.y.wrapping_sub(1);
    
        // 更新标志寄存器
        self.check_zsflag(self.registers.y);
    
        // 根据寻址模式和是否跨页来更新 CPU 周期
        self.cpu_cycle += match addressing_mode {
            AddressingMode::Implied => 2,
            _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::DEY, addressing_mode),
        };
        self.registers.pc+=addressing_mode.operand_size();
    }

    fn dex(&mut self, addressing_mode: AddressingMode) {
        // 更新 X 寄存器
        self.registers.x = self.registers.x.wrapping_sub(1);
    
        // 更新标志寄存器
        self.check_zsflag(self.registers.x);
    
        // 根据寻址模式和是否跨页来更新 CPU 周期
        self.cpu_cycle += match addressing_mode {
            AddressingMode::Implied => 2,
            _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::DEX, addressing_mode),
        };
        self.registers.pc+=addressing_mode.operand_size();
    }

    fn tay (&mut self, addressing_mode: AddressingMode) {
        // 更新 Y 寄存器
        self.registers.y = self.registers.a;
    
        // 更新标志寄存器
        self.check_zsflag(self.registers.y);
    
        // 根据寻址模式和是否跨页来更新 CPU 周期
        self.cpu_cycle += match addressing_mode {
            AddressingMode::Implied => 2,
            _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::TAY, addressing_mode),
        };
        self.registers.pc+=addressing_mode.operand_size();
    }

    fn tax (&mut self, addressing_mode: AddressingMode) {
        // 更新 X 寄存器
        self.registers.x = self.registers.a;
    
        // 更新标志寄存器
        self.check_zsflag(self.registers.x);
    
        // 根据寻址模式和是否跨页来更新 CPU 周期
        self.cpu_cycle += match addressing_mode {
            AddressingMode::Implied => 2,
            _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::TAX, addressing_mode),
        };
        self.registers.pc+=addressing_mode.operand_size();
    }

    fn tya (&mut self, addressing_mode: AddressingMode) {
        // 更新 A 寄存器
        self.registers.a = self.registers.y;
    
        // 更新标志寄存器
        self.check_zsflag(self.registers.a);
    
        // 根据寻址模式和是否跨页来更新 CPU 周期
        self.cpu_cycle += match addressing_mode {
            AddressingMode::Implied => 2,
            _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::TYA, addressing_mode),
        };
        self.registers.pc+=addressing_mode.operand_size();
    }

    fn txa (&mut self, addressing_mode: AddressingMode) {
        // 更新 A 寄存器
        self.registers.a = self.registers.x;
    
        // 更新标志寄存器
        self.check_zsflag(self.registers.a);
    
        // 根据寻址模式和是否跨页来更新 CPU 周期
        self.cpu_cycle += match addressing_mode {
            AddressingMode::Implied => 2,
            _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::TXA, addressing_mode),
        };
        self.registers.pc+=addressing_mode.operand_size();
    }

    fn tsx (&mut self, addressing_mode: AddressingMode) {
        // 更新 X 寄存器
        self.registers.x = self.registers.sp;
    
        // 更新标志寄存器
        self.check_zsflag(self.registers.x);
    
        // 根据寻址模式和是否跨页来更新 CPU 周期
        self.cpu_cycle += match addressing_mode {
            AddressingMode::Implied => 2,
            _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::TSX, addressing_mode),
        };
        self.registers.pc+=addressing_mode.operand_size();
    }

    fn txs (&mut self, addressing_mode: AddressingMode) {
        // 更新 SP 寄存器
        self.registers.sp = self.registers.x;
    
        // 根据寻址模式和是否跨页来更新 CPU 周期
        self.cpu_cycle += match addressing_mode {
            AddressingMode::Implied => 2,
            _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::TXS, addressing_mode),
        };
        self.registers.pc+=addressing_mode.operand_size();
    }

    fn rti (&mut self, addressing_mode: AddressingMode) {
        // 从栈中弹出标志寄存器
        let status = self.stack_pop();
        // 恢复处理器状态寄存器，注意：B(4)和U(5)标志位不会被恢复
        self.registers.p = status ;
        self.registers.set_flag(StatusFlags::BreakCommand, false);
        self.registers.set_flag(StatusFlags::Unused, true);
    
        // 从栈中弹出程序计数器的低字节和高字节
        let low_byte = self.stack_pop() as u16;
        let high_byte = self.stack_pop() as u16;
    
        // 恢复程序计数器
        self.registers.pc = (high_byte << 8) | low_byte;
    
        // 更新 CPU 周期
        self.cpu_cycle += 6;
    }

    fn lsr(&mut self, addressing_mode: AddressingMode,operand_address: u16){
        self.registers.pc += addressing_mode.operand_size();
        match addressing_mode {
            AddressingMode::Accumulator =>{
                let mut value = self.registers.a;
                self.registers.set_flag(StatusFlags::Carry, value & 0x01 == 0x01);
                value >>= 1;
                self.check_zsflag(value);
                self.registers.a = value;
                self.cpu_cycle += 2;
            }
            AddressingMode::ZeroPage|AddressingMode::ZeroPageX|AddressingMode::Absolute|AddressingMode::AbsoluteX =>{
                let mut value = self.memory.read(operand_address);
                self.registers.set_flag(StatusFlags::Carry, value & 0x01 == 0x01);
                value >>= 1;
                self.check_zsflag(value);
                self.memory.write(operand_address, value);
                self.cpu_cycle += match addressing_mode {
                    AddressingMode::ZeroPage=>5,
                    AddressingMode::ZeroPageX|AddressingMode::Absolute=>6,
                    AddressingMode::AbsoluteX=>7,
                    _=>panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::LSR, addressing_mode),
                }
            }
            _=>panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::LSR, addressing_mode),
        }
    }

    fn asl (&mut self, addressing_mode: AddressingMode,operand_address: u16) {
        self.registers.pc += addressing_mode.operand_size();
        match addressing_mode {
            AddressingMode::Accumulator =>{
                let mut value = self.registers.a;
                self.registers.set_flag(StatusFlags::Carry, value & 0x80 == 0x80);
                value <<= 1;
                self.check_zsflag(value);
                self.registers.a = value;
                self.cpu_cycle += 2;
            }
            AddressingMode::ZeroPage|AddressingMode::ZeroPageX|AddressingMode::Absolute|AddressingMode::AbsoluteX =>{
                let mut value = self.memory.read(operand_address);
                self.registers.set_flag(StatusFlags::Carry, value & 0x80 == 0x80);
                value <<= 1;
                self.check_zsflag(value);
                self.memory.write(operand_address, value);
                self.cpu_cycle += match addressing_mode {
                    AddressingMode::ZeroPage=>5,
                    AddressingMode::ZeroPageX|AddressingMode::Absolute=>6,
                    AddressingMode::AbsoluteX=>7,
                    _=>panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::ASL, addressing_mode),
                }
            }
            _=>panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::ASL, addressing_mode),
        }
    }

    fn ror(&mut self, addressing_mode: AddressingMode, operand_address: u16) {
        let operand = match addressing_mode {
            AddressingMode::Accumulator => self.registers.a,
            _ => self.memory.read(operand_address),
        };
    
        // 将操作数最低位旋转到C标志位
        let carry = operand & 1;
    
        // 向右旋转操作数
        let result = (operand >> 1) | (self.registers.get_flag(StatusFlags::Carry) as u8) << 7;
    
        match addressing_mode {
            AddressingMode::Accumulator => self.registers.a = result,
            _ => self.memory.write(operand_address, result),
        };
    
        // 更新 C 和 Z 标志位
        self.registers.set_flag(StatusFlags::Carry, carry != 0);
        self.check_zsflag(result);
    
        // 更新 CPU 周期
        self.cpu_cycle += match addressing_mode {
            AddressingMode::Accumulator => 2,
            AddressingMode::ZeroPage => 5,
            AddressingMode::ZeroPageX => 6,
            AddressingMode::Absolute => 6,
            AddressingMode::AbsoluteX => 7,
            _ => unreachable!(),
        };
        self.registers.pc+=addressing_mode.operand_size();
    }

    fn rol(&mut self, addressing_mode: AddressingMode, operand_address: u16) {
        let operand = match addressing_mode {
            AddressingMode::Accumulator => self.registers.a,
            _ => self.memory.read(operand_address),
        };
    
        // 将操作数最高位旋转到C标志位
        let carry = operand & 0x80;
    
        // 向左旋转操作数
        let result = (operand << 1) | (self.registers.get_flag(StatusFlags::Carry) as u8);
    
        match addressing_mode {
            AddressingMode::Accumulator => self.registers.a = result,
            _ => self.memory.write(operand_address, result),
        };
    
        // 更新 C 和 Z 标志位
        self.registers.set_flag(StatusFlags::Carry, carry != 0);
        self.check_zsflag(result);
    
        // 更新 CPU 周期
        self.cpu_cycle += match addressing_mode {
            AddressingMode::Accumulator => 2,
            AddressingMode::ZeroPage => 5,
            AddressingMode::ZeroPageX => 6,
            AddressingMode::Absolute => 6,
            AddressingMode::AbsoluteX => 7,
            _ => unreachable!(),
        };

        self.registers.pc+=addressing_mode.operand_size();
    }
    
    fn sty(&mut self, addressing_mode: AddressingMode, operand_address: u16){
        self.memory.write(operand_address, self.registers.y);
        self.cpu_cycle += match addressing_mode {
            AddressingMode::ZeroPage => 3,
            AddressingMode::ZeroPageX => 4,
            AddressingMode::Absolute => 4,
            _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::STY, addressing_mode),
        };
        self.registers.pc += addressing_mode.operand_size();
    }

    fn inc(&mut self, addressing_mode: AddressingMode, operand_address: u16){
        let operand = self.memory.read(operand_address);
        let result = operand.wrapping_add(1);
        self.check_zsflag(result);
        self.memory.write(operand_address, result);
        self.cpu_cycle += match addressing_mode {
            AddressingMode::ZeroPage => 5,
            AddressingMode::ZeroPageX => 6,
            AddressingMode::Absolute => 6,
            AddressingMode::AbsoluteX => 7,
            _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::INC, addressing_mode),
        };
        self.registers.pc += addressing_mode.operand_size();
    }

    fn dec(&mut self, addressing_mode: AddressingMode, operand_address: u16){
        let operand = self.memory.read(operand_address);
        let result = operand.wrapping_sub(1);
        self.check_zsflag(result);
        self.memory.write(operand_address, result);
        self.cpu_cycle += match addressing_mode {
            AddressingMode::ZeroPage => 5,
            AddressingMode::ZeroPageX => 6,
            AddressingMode::Absolute => 6,
            AddressingMode::AbsoluteX => 7,
            _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::DEC, addressing_mode),
        };
        self.registers.pc += addressing_mode.operand_size();
    }

    // ... 实现其他指令
}

 