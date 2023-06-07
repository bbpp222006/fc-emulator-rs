//包含主要的 Cpu 结构体，以及与模拟 6502 CPU 相关的所有方法。这些方法可能包括初始化 CPU（new）、执行指令（step）、读取和写入内存（read 和 write）等。
// src/cpu/cpu.rs

use crate::cpu::instructions::Instruction;
use crate::cpu::opcodes::decode_opcode;
use crate::cpu::addressing_modes::AddressingMode;
use crate::cpu::registers::{Registers,StatusFlags};
use crate::bus::{RWMessage,RWResult,RWType, Bus};
use crate::utils::GlobalSignal;

use std::cell::RefCell;
use std::rc::Rc;
use std::{thread, vec};
use std::sync::{Arc, Mutex};
use crossbeam::channel::{bounded, select, Receiver, Sender};

// use super::cpu_ram::CpuRam;
use super::instructions::InstructionInfo;


/// 6502 CPU 的结构体
pub struct Cpu {
    pub registers: Registers, // CPU 寄存器
    interrupt: Interrupt, // 中断类型
    pub cpu_cycle: u64, // CPU 周期
    pub instruction_info: InstructionInfo, // 当前指令信息
    pub cpu_cycle_wait: u64,
    // cpu_ram: CpuRam, // CPU 内存
    // pub channels: CpuChannels,
    log: String,
    bus: Rc<RefCell<Bus>>,
}

pub struct CpuChannels {
    cpu2mem_in: Sender<RWMessage>,
    mem2cpu_out: Receiver<RWResult>,
}

pub struct Interrupt {
    nmi: (bool,bool), // (当前nmi状态,上一次nmi状态)
    irq: bool,
    reset: bool,
}

impl Default for Interrupt {
    fn default() -> Self {
        Interrupt {
            nmi: (false,false),
            irq: false,
            reset: false,
        }
    }
}

// pub fn start_cpu_thread(cpu2mem_in:Sender<RWMessage>,mem2cpu_out:Receiver<RWResult>,global_signal_out:Receiver<GlobalSignal>,pip_log_in:Sender<String>) {
//     let mut cpu = Cpu::new(cpu2mem_in,mem2cpu_out);
//     thread::spawn(move || {
//         loop {
//             let global_signal_out = global_signal_out.recv().unwrap();
//             match global_signal_out {
//                 GlobalSignal::Clock => {
//                     // println!("接收到时钟信息");
//                     if cpu.cpu_cycle_wait == 0 {
//                         // println!("cpu开始执行指令");
//                         cpu.step();
//                         // let log = &cpu.log;
//                         // println!("{}",log);
//                         // println!("cpu开始执行完成");
//                     } else {
//                         // println!("cpu等待中，等待周期数：{}",cpu.cpu_cycle_wait);
//                         cpu.cpu_cycle_wait -= 1;
//                     }
//                 },
//                 GlobalSignal::Reset => {
//                     // println!("接收到复位信息，cpu开始执行复位");
//                     cpu.reset();
//                     // println!("cpu复位结束");
//                 },
//                 GlobalSignal::GetLog => {
//                     // println!("接收到获取日志信息，cpu开始执行获取日志");
//                     let log = cpu.get_current_log();
//                     pip_log_in.send(log).unwrap();
//                 },
//                 GlobalSignal::Step => {
//                     // println!("接收到cpu强制执行信息，cpu开始执行指令");
//                     cpu.step();
//                     // println!("cpu开始执行完成");
//                 },
//             }
            
//         }
//     });
// }



impl Cpu {
    pub fn new(bus:Rc<RefCell<Bus>>) -> Self {
        Cpu {
            registers: Registers::default(),
            interrupt: Interrupt::default(),
            cpu_cycle: 0,
            instruction_info: InstructionInfo::default(),
            cpu_cycle_wait: 0,
            // cpu_ram: CpuRam::new(),
            // channels: CpuChannels{
            //     cpu2mem_in,
            //     mem2cpu_out,
            // },
            log: String::new(),
            bus,
        }
    }
    
    //https://www.nesdev.org/wiki/CPU_power_up_state ,待优化    
    pub fn reset(&mut self) {
        self.registers.sp = 0xFD; 
        self.registers.pc = self.read_u16(0xFFFC); // 从内存中读取复位向量
        self.registers.set_flag(StatusFlags::InterruptDisable, true);
        self.cpu_cycle=7;
        self.instruction_info=InstructionInfo::default();
        self.log=String::new();
        // self.cpu_ram.reset();
    }

    fn read(&self, address: u16) -> u8 {
        let read_result = self.bus.borrow_mut().cpu_read(address);
        read_result
    }

    fn write(&mut self, address: u16, data: u8) {
        self.bus.borrow_mut().cpu_write(address, data);
        if address == 0x4014 {
            self.cpu_cycle+= if self.cpu_cycle&1==1{514}else{513};
        }
    }

    fn read_interrupt_status(&mut self) {
        let interrupt_status = self.bus.borrow_mut().interrupt_status;
        self.interrupt.irq = interrupt_status & 1 == 1;
        self.interrupt.nmi.0 = interrupt_status & 2 == 2;
        self.interrupt.reset = interrupt_status & 4 == 4;
    }
    
    fn write_interrupt_status(&mut self) {
        let mut interrupt_status = 0;
        interrupt_status|=if self.interrupt.irq {1} else {0};
        interrupt_status|=if self.interrupt.nmi.0 {2} else {0};
        interrupt_status|=if self.interrupt.reset {4} else {0};
        self.bus.borrow_mut().interrupt_status = interrupt_status;
    }


    fn read_u16(&self, address: u16) -> u16 {
        let lo = self.read(address) as u16;
        let hi = self.read(address + 1) as u16;
        (hi << 8) | lo
    }

    fn read_u16_z(&self, address:u8 ) -> u16 {
        let low_byte = self.read(address as u16) as u16;
        let high_byte = self.read(address.wrapping_add(1) as u16) as u16;
        (high_byte << 8) | low_byte
    }


    /// 执行一条指令
    pub fn step(&mut self) {
        self.read_interrupt_status();
        // 如果有复位信号，执行复位
        if self.interrupt.reset {
            self.reset();
            // println!("reset");
            self.interrupt.reset = false;
        }
        // 执行nmi，边缘触发
        if (self.interrupt.nmi.0==true) && (self.interrupt.nmi.1==false)  {
            self.nmi();
            // println!("nmi");
        }
        // 如果有irq信号，执行irq
        if self.interrupt.irq && !self.registers.get_flag(StatusFlags::InterruptDisable) {
            self.irq();
            // println!("irq");
            self.interrupt.irq = false;
        }
        self.interrupt.nmi.1 = self.interrupt.nmi.0;
        // self.write_interrupt_status();
        // 解码操作码为指令和寻址模式
        let opcode = self.read(self.registers.pc);
        self.instruction_info = decode_opcode(opcode);

        let current_cyc = self.cpu_cycle;
        self.execute();  
        self.cpu_cycle_wait = self.cpu_cycle-current_cyc; 
    }

        // 反汇编当前结果
    pub fn get_current_log(&self) -> String{
        format!(
            "{: <48}A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
            self.disassemble_instruction(self.registers.pc),
            self.registers.a,
            self.registers.x,
            self.registers.y,
            self.registers.p,
            self.registers.sp,
        )
    } 

    // 静态反汇编用
    fn read_debug(&self, address: u16) -> u8 {
        let read_result = self.bus.borrow_mut().cpu_read_debug(address);
        read_result
    }

    fn read_u16_z_debug(&self, address:u8 ) -> u16 {
        let low_byte = self.read_debug(address as u16) as u16;
        let high_byte = self.read_debug(address.wrapping_add(1) as u16) as u16;
        (high_byte << 8) | low_byte
    }

    pub fn disassemble_instruction_short(&self) -> String {
        use crate::cpu::instructions::Instruction::*;
        let address = self.registers.pc;
        let opcode = self.read_debug(address);
        let instruction_info = decode_opcode(opcode);

        // 开始的地址
        let mut output = "".to_string(); 
        let operand_size = instruction_info.operand_size;

        // opcode 和 根据寻址模式的接下来几位内存
        let mut tmp=vec![];
        for i in 1..=operand_size-1 {
            let code = self.read_debug(address + i as u16);
            tmp.push(code);
        }
        //将tmp向量合并成u16
        let tmp =tmp.iter().rev().fold(0, |acc, &x| (acc << 8) | x as u16);

        // 如果opcode是拓展指令，则在前面加*
        if instruction_info.unofficial {
            output.push_str(&"*");
        } else {
            output.push_str(&" ");
        }

        // 具体指令
        match  instruction_info.instruction {
            DOP|TOP=>{
                output.push_str(&format!("NOP "));
            }
            ISC=>{
                output.push_str(&format!("ISB "));
            }
            _=>{
                output.push_str(&format!("{:?} ", instruction_info.instruction));
            }
        }
        

        // 指令运行细节
       
        match instruction_info.instruction {
            //所有的流程指令
            JMP|JSR|BEQ|BNE|BCS|BCC|BMI|BPL|BVS|BVC|RTS|RTI|BRK => {
                match instruction_info.addressing_mode {
                    AddressingMode::Implied => (),
                    AddressingMode::Absolute => {
                        let operand = tmp;
                        output.push_str(&format!("${:04X}", operand));
                    }
                    AddressingMode::Indirect => {
                        let operand_address_address = tmp;
                        let low_byte = self.read_debug(operand_address_address);
                        let high_byte = if operand_address_address & 0x00FF == 0x00FF {
                            self.read_debug(operand_address_address & 0xFF00)
                        } else {
                            self.read_debug(operand_address_address + 1)
                        };
                        let operand_address = (high_byte as u16) << 8 | low_byte as u16;
                        output.push_str(&format!("(${:04X}) = {:04X}", operand_address_address, operand_address));
                    }
                    AddressingMode::Relative => {
                        let init_offset = tmp as u8;
                        let offset = init_offset as i8; // 读取当前地址的值作为偏移量（有符号数）
                        let operand_address = ((address+1) as i32 + 1 + (offset as i32)) as u16;
                        output.push_str(&format!("${:04X}", operand_address));
                    }
                    _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", instruction_info.instruction, instruction_info.addressing_mode),
                }
            }
            // 所有非跳转指令
            _ => {
                match instruction_info.addressing_mode {
                    AddressingMode::Implied => (),
                    AddressingMode::Accumulator => output.push_str("A"),
                    AddressingMode::Immediate => {
                        let operand = tmp;
                        output.push_str(&format!("#${:02X}", operand));
                    }
                    AddressingMode::ZeroPage => {
                        let operand_address = tmp;
                        output.push_str(&format!("${:02X} = {:02X}", operand_address, self.read_debug(operand_address)));
                    }
                    AddressingMode::ZeroPageX => {
                        let operand_address = tmp;
                        output.push_str(&format!("${:02X},X @ {:02X} = {:02X}", operand_address, (operand_address + self.registers.x as u16) & 0xFF, self.read_debug((operand_address + self.registers.x as u16) & 0xFF)));
                    }
                    AddressingMode::ZeroPageY => {
                        let operand_address = tmp;
                        output.push_str(&format!("${:02X},Y @ {:02X} = {:02X}", operand_address, (operand_address + self.registers.y as u16) & 0xFF, self.read_debug((operand_address + self.registers.y as u16) & 0xFF)));
                    }
                    AddressingMode::Absolute => {
                        let operand = tmp;
                        output.push_str(&format!("${:04X} = {:02X}", operand, self.read_debug(operand)));
                    }
                    AddressingMode::AbsoluteX => {
                        let operand = tmp;
                        output.push_str(&format!("${:04X},X @ {:04X} = {:02X}", operand, operand + self.registers.x as u16, self.read_debug(operand + self.registers.x as u16)));
                    }
                    AddressingMode::AbsoluteY => {
                        let base_address = tmp;
                        let operand_address = base_address.wrapping_add(self.registers.y as u16);
                        let operand = self.read_debug(operand_address);
                        output.push_str(&format!("${:04X},Y @ {:04X} = {:02X}", base_address, operand_address, operand));
                    }
                    AddressingMode::Indirect => {
                        let operand_address_address = tmp as u16;
                        let low_byte = self.read_debug(operand_address_address);
                        let high_byte = if operand_address_address & 0x00FF == 0x00FF {
                            self.read_debug(operand_address_address & 0xFF00)
                        } else {
                            self.read_debug(operand_address_address + 1)
                        };
                        let operand_address = (high_byte as u16) << 8 | low_byte as u16;
                        output.push_str(&format!("(${:04X}) = {:04X}", operand_address_address, operand_address));
                    }
                    AddressingMode::IndirectX => {
                        let base_address = tmp as u8;
                        let operand_address = self.read_u16_z_debug(base_address.wrapping_add(self.registers.x));
                        let operand = self.read_debug(operand_address);
                        output.push_str(&format!("(${:02X},X) @ {:02X} = {:04X} = {:02X}", base_address, (base_address as u16 + self.registers.x as u16) & 0xFF, operand_address, operand));
                    }
                    AddressingMode::IndirectY => {
                        let base_address_address = tmp as u8;
                        let base_address = self.read_u16_z_debug(base_address_address);
                        let operand_address = base_address.wrapping_add(self.registers.y as u16) ;
                        let operand = self.read_debug(operand_address);
                        output.push_str(&format!("(${:02X}),Y = {:04X} @ {:04X} = {:02X}", base_address_address, base_address, operand_address, operand));
                    }
                    _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", instruction_info.instruction, instruction_info.addressing_mode),
                }
            }
        }        
        output 
    }

    /// 反汇编指定地址处的指令，返回反汇编结果
    pub fn disassemble_instruction(&self, address: u16) -> String {
        use crate::cpu::instructions::Instruction::*;

        let opcode = self.read_debug(address);
        let instruction_info = decode_opcode(opcode);

        // 开始的地址
        let mut output = format!("{:04X}  ", address); 
        let operand_size = instruction_info.operand_size;

        // opcode 和 根据寻址模式的接下来几位内存
        output.push_str(&format!("{:02X} ", opcode));
        let mut tmp=vec![];
        for i in 1..=operand_size-1 {
            let code = self.read_debug(address + i as u16);
            tmp.push(code);
            output.push_str(&format!("{:02X} ", code));
        }
        //将tmp向量合并成u16
        let tmp =tmp.iter().rev().fold(0, |acc, &x| (acc << 8) | x as u16);

        output = format!("{: <15}", output);

        // 如果opcode是拓展指令，则在前面加*
        if instruction_info.unofficial {
            output.push_str(&"*");
        } else {
            output.push_str(&" ");
        }

        // 具体指令
        match  instruction_info.instruction {
            DOP|TOP=>{
                output.push_str(&format!("NOP "));
            }
            ISC=>{
                output.push_str(&format!("ISB "));
            }
            _=>{
                output.push_str(&format!("{:?} ", instruction_info.instruction));
            }
        }
        

        // 指令运行细节
       
        match instruction_info.instruction {
            //所有的流程指令
            JMP|JSR|BEQ|BNE|BCS|BCC|BMI|BPL|BVS|BVC|RTS|RTI|BRK => {
                match instruction_info.addressing_mode {
                    AddressingMode::Implied => (),
                    AddressingMode::Absolute => {
                        let operand = tmp;
                        output.push_str(&format!("${:04X}", operand));
                    }
                    AddressingMode::Indirect => {
                        let operand_address_address = tmp;
                        let low_byte = self.read_debug(operand_address_address);
                        let high_byte = if operand_address_address & 0x00FF == 0x00FF {
                            self.read_debug(operand_address_address & 0xFF00)
                        } else {
                            self.read_debug(operand_address_address + 1)
                        };
                        let operand_address = (high_byte as u16) << 8 | low_byte as u16;
                        output.push_str(&format!("(${:04X}) = {:04X}", operand_address_address, operand_address));
                    }
                    AddressingMode::Relative => {
                        let init_offset = tmp as u8;
                        let offset = init_offset as i8; // 读取当前地址的值作为偏移量（有符号数）
                        let operand_address = ((address+1) as i32 + 1 + (offset as i32)) as u16;
                        output.push_str(&format!("${:04X}", operand_address));
                    }
                    _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", instruction_info.instruction, instruction_info.addressing_mode),
                }
            }
            // 所有非跳转指令
            _ => {
                match instruction_info.addressing_mode {
                    AddressingMode::Implied => (),
                    AddressingMode::Accumulator => output.push_str("A"),
                    AddressingMode::Immediate => {
                        let operand = tmp;
                        output.push_str(&format!("#${:02X}", operand));
                    }
                    AddressingMode::ZeroPage => {
                        let operand_address = tmp;
                        output.push_str(&format!("${:02X} = {:02X}", operand_address, self.read_debug(operand_address)));
                    }
                    AddressingMode::ZeroPageX => {
                        let operand_address = tmp;
                        output.push_str(&format!("${:02X},X @ {:02X} = {:02X}", operand_address, (operand_address + self.registers.x as u16) & 0xFF, self.read_debug((operand_address + self.registers.x as u16) & 0xFF)));
                    }
                    AddressingMode::ZeroPageY => {
                        let operand_address = tmp;
                        output.push_str(&format!("${:02X},Y @ {:02X} = {:02X}", operand_address, (operand_address + self.registers.y as u16) & 0xFF, self.read_debug((operand_address + self.registers.y as u16) & 0xFF)));
                    }
                    AddressingMode::Absolute => {
                        let operand = tmp;
                        output.push_str(&format!("${:04X} = {:02X}", operand, self.read_debug(operand)));
                    }
                    AddressingMode::AbsoluteX => {
                        let operand = tmp;
                        output.push_str(&format!("${:04X},X @ {:04X} = {:02X}", operand, operand + self.registers.x as u16, self.read_debug(operand + self.registers.x as u16)));
                    }
                    AddressingMode::AbsoluteY => {
                        let base_address = tmp;
                        let operand_address = base_address.wrapping_add(self.registers.y as u16);
                        let operand = self.read_debug(operand_address);
                        output.push_str(&format!("${:04X},Y @ {:04X} = {:02X}", base_address, operand_address, operand));
                    }
                    AddressingMode::Indirect => {
                        let operand_address_address = tmp as u16;
                        let low_byte = self.read_debug(operand_address_address);
                        let high_byte = if operand_address_address & 0x00FF == 0x00FF {
                            self.read_debug(operand_address_address & 0xFF00)
                        } else {
                            self.read_debug(operand_address_address + 1)
                        };
                        let operand_address = (high_byte as u16) << 8 | low_byte as u16;
                        output.push_str(&format!("(${:04X}) = {:04X}", operand_address_address, operand_address));
                    }
                    AddressingMode::IndirectX => {
                        let base_address = tmp as u8;
                        let operand_address = self.read_u16_z_debug(base_address.wrapping_add(self.registers.x));
                        let operand = self.read_debug(operand_address);
                        output.push_str(&format!("(${:02X},X) @ {:02X} = {:04X} = {:02X}", base_address, (base_address as u16 + self.registers.x as u16) & 0xFF, operand_address, operand));
                    }
                    AddressingMode::IndirectY => {
                        let base_address_address = tmp as u8;
                        let base_address = self.read_u16_z_debug(base_address_address);
                        let operand_address = base_address.wrapping_add(self.registers.y as u16) ;
                        let operand = self.read_debug(operand_address);
                        output.push_str(&format!("(${:02X}),Y = {:04X} @ {:04X} = {:02X}", base_address_address, base_address, operand_address, operand));
                    }
                    _ => panic!("当前指令:{:?} 不存在寻址模式{:?}", instruction_info.instruction, instruction_info.addressing_mode),
                }
            }
        }        
        output 
    }


    pub fn get_operand_address(&self)-> (u16,bool) {
        fn check_page_boundary_crossed(addr1: u16, addr2: u16) -> bool {
            (addr1 & 0xFF00) != (addr2 & 0xFF00)
        }
        let address = self.registers.pc+1;
        let mut operand_address = 0;
        let mut page_crossed = false;
        match self.instruction_info.addressing_mode {
            AddressingMode::Implied|AddressingMode::Accumulator => (),
            AddressingMode::Immediate => {
                operand_address = address;
            }
            AddressingMode::Absolute => {
                operand_address = self.read_u16(address);
            }
            AddressingMode::AbsoluteX => {
                let base_address = self.read_u16(address);
                operand_address = base_address + self.registers.x as u16;
                // 页面交叉判断
                if check_page_boundary_crossed(base_address ,operand_address){
                    page_crossed = true;
                }
            }
            AddressingMode::AbsoluteY => {
                let base_address = self.read_u16(address);
                operand_address = base_address.wrapping_add(self.registers.y as u16);
                // 页面交叉判断
                if check_page_boundary_crossed(base_address ,operand_address){
                    page_crossed = true;
                }
            }
            AddressingMode::ZeroPage => {
                operand_address = self.read(address) as u16;
            }
            AddressingMode::ZeroPageX => {
                let base_address = self.read(address) as u16;
                operand_address = (base_address + self.registers.x as u16)& 0x00FF;
            }
            AddressingMode::ZeroPageY => {
                let base_address = self.read(address) as u16;
                operand_address = (base_address + self.registers.y as u16)& 0x00FF;
            }
            AddressingMode::Indirect => {
                let operand_address_address = self.read_u16(address);
                let low_byte = self.read(operand_address_address);
                let high_byte = if operand_address_address & 0x00FF == 0x00FF {
                    self.read(operand_address_address & 0xFF00)
                } else {
                    self.read(operand_address_address + 1)
                };
                operand_address = (high_byte as u16) << 8 | low_byte as u16;
            }
            AddressingMode::IndirectX => {
                let base_address = self.read(address);
                operand_address = self.read_u16_z(base_address.wrapping_add(self.registers.x));
            }
            AddressingMode::IndirectY => {
                let base_address_address = self.read(address);
                let base_address = self.read_u16_z(base_address_address);
                operand_address = base_address.wrapping_add(self.registers.y as u16) ;
                // 页面交叉判断
                if check_page_boundary_crossed(base_address ,operand_address){
                    page_crossed = true;
                }
            }
            AddressingMode::Relative => {
                let init_offset = self.read(address);
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

    /// 执行指令
    fn execute(&mut self) {
        // 根据寻址模式获取操作数所在的地址
        match self.instruction_info.instruction {
            Instruction::JMP => self.jmp(),
            Instruction::LDX => self.ldx(),
            Instruction::STX => self.stx(),
            Instruction::JSR => self.jsr(),
            Instruction::NOP => self.nop(),
            Instruction::SEC => self.sec(),
            Instruction::BCS => self.bcs(),
            Instruction::CLC => self.clc(),
            Instruction::BCC => self.bcc(),
            Instruction::LDA => self.lda(),
            Instruction::BEQ => self.beq(),
            Instruction::BNE => self.bne(),
            Instruction::STA => self.sta(),
            Instruction::BIT => self.bit(),
            Instruction::BVS => self.bvs(),
            Instruction::BVC => self.bvc(),
            Instruction::BPL => self.bpl(),
            Instruction::RTS => self.rts(),
            Instruction::SEI => self.sei(),
            Instruction::SED => self.sed(),
            Instruction::PHP => self.php(),
            Instruction::PLA => self.pla(),
            Instruction::AND => self.and(),
            Instruction::CMP => self.cmp(),
            Instruction::CLD => self.cld(),
            Instruction::PHA => self.pha(),
            Instruction::PLP => self.plp(),
            Instruction::BMI => self.bmi(),
            Instruction::ORA => self.ora(),
            Instruction::CLV => self.clv(),
            Instruction::EOR => self.eor(),
            Instruction::ADC => self.adc(),
            Instruction::LDY => self.ldy(),
            Instruction::CPY => self.cpy(),
            Instruction::CPX => self.cpx(),
            Instruction::SBC => self.sbc(),
            Instruction::INY => self.iny(),
            Instruction::INX => self.inx(),
            Instruction::DEY => self.dey(),
            Instruction::DEX => self.dex(),
            Instruction::TAY => self.tay(),
            Instruction::TAX => self.tax(),
            Instruction::TYA => self.tya(),
            Instruction::TXA => self.txa(),
            Instruction::TSX => self.tsx(),
            Instruction::TXS => self.txs(),
            Instruction::RTI => self.rti(),
            Instruction::LSR => self.lsr(),
            Instruction::ASL => self.asl(),
            Instruction::ROR => self.ror(),
            Instruction::ROL => self.rol(),
            Instruction::STY => self.sty(),
            Instruction::INC => self.inc(),
            Instruction::DEC => self.dec(),
            Instruction::DOP => self.nop(),
            Instruction::TOP => self.nop(),
            Instruction::LAX => self.lax(),
            Instruction::SAX => self.sax(),
            Instruction::DCP => self.dcp(),
            Instruction::ISC => self.isc(),
            Instruction::SLO => self.slo(),
            Instruction::RLA => self.rla(),
            Instruction::SRE => self.sre(),
            Instruction::RRA => self.rra(),
            Instruction::BRK => self.brk(),
            Instruction::CLI => self.cli(),
            // ... 处理其他指令
            _ => panic!("{:?}指令暂未实现",self.instruction_info.instruction), // 如果尚未实现的指令，触发未实现错误
        }
    }

    fn stack_push(&mut self, value: u8) {

        // 堆栈基地址是 0x0100
        const STACK_BASE: u16 = 0x0100;
    
        // 计算堆栈当前位置的地址，将堆栈指针（SP）加上基地址
        let stack_address = 0x0100 + self.registers.sp as u16;
    
        // 将值写入当前堆栈位置
        self.write(stack_address, value);
    
        // 更新堆栈指针（SP），将其减 1，指向下一个可用位置
        self.registers.sp = self.registers.sp.wrapping_sub(1);
    }

    fn stack_push_16(&mut self, value: u16) {
        // 将 16 位值的高 8 位压入堆栈
        self.stack_push(((value >> 8) & 0xFF) as u8);
    
        // 将 16 位值的低 8 位压入堆栈
        self.stack_push((value & 0xFF) as u8);
    }
    
    fn stack_pop(&mut self) -> u8 {

        // 堆栈基地址是 0x0100
        const STACK_BASE: u16 = 0x0100;

        // 堆栈指针（SP）加 1
        self.registers.sp = self.registers.sp.wrapping_add(1);
    
        // 堆栈的基地址为 0x0100
        let stack_address = STACK_BASE + self.registers.sp as u16;
    
        // 从内存中读取位于 stack_address 的值
        let value = self.read(stack_address);
    
        // 返回弹出的值
        value
    }

    fn nmi(&mut self) {
        // 保存当前 PC 寄存器的值
        self.stack_push_16(self.registers.pc);
    
        // 保存当前状态寄存器的值
        self.stack_push(self.registers.p);
    
        // 将 PC 寄存器设置为 NMI 中断处理程序的地址
        self.registers.pc = self.read_u16(0xFFFA);
    }

    fn irq(&mut self) {
        // 保存当前 PC 寄存器的值
        self.stack_push_16(self.registers.pc);
    
        // 保存当前状态寄存器的值
        self.stack_push(self.registers.p);
    
        // 将 PC 寄存器设置为 IRQ 中断处理程序的地址
        self.registers.pc = self.read_u16(0xFFFE);
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

    fn cld(&mut self) {
        // 清除十进制模式标志（Decimal flag）
        self.registers.set_flag(StatusFlags::DecimalMode, false);
        self.registers.pc += self.instruction_info.operand_size as u16;
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
    }

    fn cmp(&mut self) {
        let (operand_address,page_crossed) = self.get_operand_address();
        // 从内存中读取操作数
        let operand = self.read(operand_address);
    
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
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        if page_crossed{
            self.cpu_cycle += 1;
        }
        self.registers.pc += self.instruction_info.operand_size as u16;
    }

    fn and(&mut self) {
        let (operand_address,page_crossed) = self.get_operand_address();
        // 从内存中读取操作数
        let operand = self.read(operand_address);
    
        // 将操作数与寄存器 A 进行 AND 运算
        self.registers.a &= operand;
    
        // 检查零标志（Zero flag）和负数标志（Sign 或 Negative flag）
        self.check_zsflag(self.registers.a);
        
        // 增加所需的 CPU 周期
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        // 如果跨页了，增加一个周期
        if page_crossed {
            self.cpu_cycle += 1;
        }
    
        self.registers.pc += self.instruction_info.operand_size as u16;
    }


    fn pla(&mut self) {
        // 从堆栈中弹出值
        let value = self.stack_pop();
    
        // 将值写入寄存器 A
        self.registers.a = value;
    
        self.check_zsflag(self.registers.a);
    
        // 增加所需的 CPU 周期
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
    
        self.registers.pc += self.instruction_info.operand_size as u16;
    }

    fn php(&mut self) {
        // 将状态寄存器的值压入堆栈
        self.stack_push(self.registers.p|StatusFlags::BreakCommand as u8|StatusFlags::Unused as u8);
    
        // 增加所需的 CPU 周期
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
    
        self.registers.pc += self.instruction_info.operand_size as u16;
    }

    fn sed(&mut self) {
        // 设置状态寄存器的 D 标志位
        self.registers.set_flag(StatusFlags::DecimalMode, true);

        // 增加所需的 CPU 周期
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
    
        self.registers.pc += self.instruction_info.operand_size as u16;
    }

    fn sei(&mut self) {
        // 设置状态寄存器的 I 标志位
        self.registers.set_flag(StatusFlags::InterruptDisable, true);

        // 增加所需的 CPU 周期
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
    
        self.registers.pc += self.instruction_info.operand_size as u16;
    }

    fn rts(&mut self) {
        // 从堆栈中弹出返回地址的低字节（低 8 位）
        let low_byte = self.stack_pop() as u16;

        // 从堆栈中弹出返回地址的高字节（高 8 位）
        let high_byte = self.stack_pop() as u16;

        // 将返回地址的高字节和低字节组合成完整的 16 位地址
        let return_address = (high_byte << 8) | low_byte;

        // 将程序计数器（PC）设置为返回地址 + 1，以返回调用 JSR 指令之前的指令
        self.registers.pc = return_address.wrapping_add(1);

        // 增加所需的 CPU 周期
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
    }

    fn bpl(&mut self) {
        let (operand_address,page_crossed) = self.get_operand_address();
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        if !self.registers.get_flag(StatusFlags::Negative) {
            self.cpu_cycle += 1;
            self.registers.pc = operand_address;
            if page_crossed {
                self.cpu_cycle += 1;
            }
        } else {
            self.registers.pc += self.instruction_info.operand_size as u16;
        }
    }

    fn bvc(&mut self) {
        let (operand_address,page_crossed) = self.get_operand_address();
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        if !self.registers.get_flag(StatusFlags::Overflow) {
            self.cpu_cycle += 1;
            self.registers.pc = operand_address;
            if page_crossed {
                self.cpu_cycle += 1;
            }
        } else {
            self.registers.pc += self.instruction_info.operand_size as u16;
        }
    }

    fn bvs(&mut self) {
        let (operand_address,page_crossed) = self.get_operand_address();
        if self.registers.get_flag(StatusFlags::Overflow) {
            self.cpu_cycle += 1;
            self.registers.pc = operand_address;
            if page_crossed {
                self.cpu_cycle += 1;
            }
        } else {
            self.registers.pc += self.instruction_info.operand_size as u16;
        }
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
    }

    fn bit(&mut self) {
        let (operand_address,page_crossed) = self.get_operand_address();
        let operand = self.read(operand_address);
        let value = self.registers.a & operand;
        self.registers.set_flag(StatusFlags::Zero, value==0);
        self.registers.set_flag(StatusFlags::Overflow, operand & 0x40 != 0);
        self.registers.set_flag(StatusFlags::Negative, operand & 0x80 != 0);
        self.registers.pc += self.instruction_info.operand_size as u16;
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
    }

    fn sta(&mut self) {
        let (operand_address,page_crossed) = self.get_operand_address();
        let value = self.registers.a;
        self.write(operand_address, value);
        self.registers.pc += self.instruction_info.operand_size as u16;
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
    }
    
    fn bne(&mut self) {
        let (operand_address,page_crossed) = self.get_operand_address();
        if !self.registers.get_flag(StatusFlags::Zero) {
            self.registers.pc = operand_address;
            self.cpu_cycle += 1;
            if page_crossed {
                self.cpu_cycle += 1;
            }
        } else {
            self.registers.pc += self.instruction_info.operand_size as u16;
        }
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
    }

    // BEQ 指令实现
    fn beq(&mut self) {
        let (operand_address,page_crossed) = self.get_operand_address();
        if self.registers.get_flag(StatusFlags::Zero) {
            self.registers.pc = operand_address;
            self.cpu_cycle += 1;
            if page_crossed {
                self.cpu_cycle += 1;
            }
        } else {
            self.registers.pc += self.instruction_info.operand_size as u16;
        }
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
    }

    // LDA 指令实现
    fn lda(&mut self) {
        let (operand_address,page_crossed) = self.get_operand_address();
        let operand = self.read(operand_address);
        self.check_zsflag(operand);
        self.registers.a = operand;
        self.registers.pc += self.instruction_info.operand_size as u16;
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        if page_crossed {
            self.cpu_cycle += 1;
        }
    }

    // BCC 指令实现
    fn bcc(&mut self) {
        let (operand_address,page_crossed) = self.get_operand_address();
        // 如果进位标志（Carry）为 0，则跳转到指定地址
            if !self.registers.get_flag(StatusFlags::Carry) {
                self.registers.pc = operand_address;
                self.cpu_cycle += 1;
                //在页面边界交叉时 +1s
                if page_crossed {
                    self.cpu_cycle += 1;
                }
            } else {
                self.registers.pc += self.instruction_info.operand_size as u16;
            }
            self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        }

    // CLC 指令实现
    fn clc(&mut self) {
        self.registers.set_flag(StatusFlags::Carry, false);
        self.registers.pc += 1;
        self.cpu_cycle += 2;
    }

    // BCS 指令实现
    fn bcs(&mut self) {
        let (operand_address,page_crossed) = self.get_operand_address();
        // 如果进位标志（Carry）为 1，则跳转到指定地址
            if self.registers.get_flag(StatusFlags::Carry) {
                self.registers.pc = operand_address;
                self.cpu_cycle += 1;
                //在页面边界交叉时 +1s
                if page_crossed {
                    self.cpu_cycle += 1;
                }
            } else {
                self.registers.pc += self.instruction_info.operand_size as u16;
            }
            self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        }
        
    /// SEC 指令实现
    fn sec(&mut self) {
        self.registers.set_flag(StatusFlags::Carry, true);
        self.registers.pc += 1;
        self.cpu_cycle += 2;
    }

    /// NOP
    fn nop(&mut self) {
        let (_,page_crossed) = self.get_operand_address();
        self.registers.pc += self.instruction_info.operand_size as u16;
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        if page_crossed {
            self.cpu_cycle += 1;
        }
    }


    /// JSR 指令实现
    fn jsr(&mut self) {
        let (operand_address,page_crossed) = self.get_operand_address();

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

        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;

    }

    /// STX 指令实现
    fn stx(&mut self) {
        let (operand_address,page_crossed) = self.get_operand_address();
        self.write(operand_address, self.registers.x);
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        self.registers.pc += self.instruction_info.operand_size as u16;
    }

    /// LDX 指令实现
    fn ldx(&mut self) {
        let (operand_address,page_crossed) = self.get_operand_address();
        
        self.cpu_cycle += self.instruction_info.instruction_cycle as u64;
        self.registers.pc+=self.instruction_info.operand_size as u16;

        let operand = self.read(operand_address);
        self.registers.x = operand;
        self.check_zsflag(operand);
        if page_crossed{
            self.cpu_cycle += 1;
        }
    }


    /// JMP 指令实现
    fn jmp(&mut self) {
        let (operand_address,page_crossed) = self.get_operand_address();
        self.registers.pc = operand_address;
        self.cpu_cycle += self.instruction_info.instruction_cycle as u64;
    }

    fn pha (&mut self) {
        self.stack_push(self.registers.a);
        self.registers.pc += self.instruction_info.operand_size as u16;
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
    }

    fn plp (&mut self) {
        self.registers.p = self.stack_pop();
        self.registers.set_flag(StatusFlags::BreakCommand, false);
        self.registers.set_flag(StatusFlags::Unused, true);
        self.registers.pc += self.instruction_info.operand_size as u16;
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
    }

    fn bmi(&mut self) {
        let (operand_address,page_crossed) = self.get_operand_address();
        if self.registers.get_flag(StatusFlags::Negative) {
            self.registers.pc = operand_address;
            self.cpu_cycle += 1;   
            if page_crossed {
                self.cpu_cycle += 1;
            }
        } else {
            self.registers.pc += self.instruction_info.operand_size as u16;
        }
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
    }

    fn ora (&mut self) {
        let (operand_address,page_crossed) = self.get_operand_address();
        let operand = self.read(operand_address);
        self.registers.a |= operand;
        self.registers.pc += self.instruction_info.operand_size as u16;
        self.check_zsflag(self.registers.a);
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        if page_crossed{
            self.cpu_cycle+=1;
        }
    }

    fn clv(&mut self) {
        self.registers.set_flag(StatusFlags::Overflow, false);
        self.registers.pc += self.instruction_info.operand_size as u16;
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
    }

    fn eor(&mut self) {
        let (operand_address,page_crossed) = self.get_operand_address();
        let operand = self.read(operand_address);
        self.registers.a ^= operand;
        self.registers.pc += self.instruction_info.operand_size as u16;
        self.check_zsflag(self.registers.a);
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        if page_crossed{
            self.cpu_cycle+=1;
        }
    }

    fn adc(&mut self) {
        let (operand_address,page_crossed) = self.get_operand_address();
        // 根据寻址模式获取操作数值
        let operand = self.read(operand_address);
    
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
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        if page_crossed{
            self.cpu_cycle+=1;
        }
        self.registers.pc += self.instruction_info.operand_size as u16;
    }
    
    fn ldy(&mut self) {
        let (operand_address,page_crossed) = self.get_operand_address();
        // 根据寻址模式获取操作数值
        let operand = self.read(operand_address);
    
        // 将操作数存入 Y 寄存器
        self.registers.y = operand;

        // 更新标志寄存器
        self.check_zsflag(self.registers.y);
    
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        if page_crossed{
            self.cpu_cycle+=1;
        }
        self.registers.pc += self.instruction_info.operand_size as u16;
    }

    fn cpy(&mut self) {
        let (operand_address,page_crossed) = self.get_operand_address();
        // 根据寻址模式获取操作数值
        let operand = self.read(operand_address);
    
        // 获取 Y 寄存器的当前值
        let y = self.registers.y;
    
        // 计算 CPY 操作的结果
        let result = y.wrapping_sub(operand);
    
        // 更新标志寄存器
        self.registers.set_flag(StatusFlags::Carry, y >= operand);
        self.check_zsflag(result);
    
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        self.registers.pc += self.instruction_info.operand_size as u16;
    }

    fn cpx(&mut self) {
        let (operand_address,page_crossed) = self.get_operand_address();
        // 根据寻址模式获取操作数值
        let operand = self.read(operand_address);
    
        // 获取 X 寄存器的当前值
        let x = self.registers.x;
    
        // 计算 CPX 操作的结果
        let result = x.wrapping_sub(operand);
    
        // 更新标志寄存器
        self.registers.set_flag(StatusFlags::Carry, x >= operand);
        self.check_zsflag(result);
    
        // 根据寻址模式和是否跨页来更新 CPU 周期
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        self.registers.pc += self.instruction_info.operand_size as u16;
    }

    fn sbc(&mut self) {
        let (operand_address,page_crossed) = self.get_operand_address();
        let acc = self.registers.a;
        let operand = self.read(operand_address);

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
    
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        if page_crossed{
            self.cpu_cycle+=1;
        }
        self.registers.pc += self.instruction_info.operand_size as u16;
    }

    fn iny(&mut self) {
        // 更新 Y 寄存器
        self.registers.y =self.registers.y.wrapping_add(1);
    
        // 更新标志寄存器
        self.check_zsflag(self.registers.y);
    
        // 根据寻址模式和是否跨页来更新 CPU 周期
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        self.registers.pc += self.instruction_info.operand_size as u16;
    }

    fn inx(&mut self) {
        // 更新 X 寄存器
        self.registers.x = self.registers.x.wrapping_add(1);
    
        // 更新标志寄存器
        self.check_zsflag(self.registers.x);
    
        // 根据寻址模式和是否跨页来更新 CPU 周期
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        self.registers.pc += self.instruction_info.operand_size as u16;
    }

    fn dey(&mut self) {
        // 更新 Y 寄存器
        self.registers.y= self.registers.y.wrapping_sub(1);
    
        // 更新标志寄存器
        self.check_zsflag(self.registers.y);
    
        // 根据寻址模式和是否跨页来更新 CPU 周期
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        self.registers.pc += self.instruction_info.operand_size as u16;
    }

    fn dex(&mut self) {
        // 更新 X 寄存器
        self.registers.x = self.registers.x.wrapping_sub(1);
    
        // 更新标志寄存器
        self.check_zsflag(self.registers.x);
    
        // 根据寻址模式和是否跨页来更新 CPU 周期
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        self.registers.pc += self.instruction_info.operand_size as u16;
    }

    fn tay (&mut self) {
        // 更新 Y 寄存器
        self.registers.y = self.registers.a;
    
        // 更新标志寄存器
        self.check_zsflag(self.registers.y);
    
        // 根据寻址模式和是否跨页来更新 CPU 周期
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        self.registers.pc += self.instruction_info.operand_size as u16;
    }

    fn tax (&mut self) {
        // 更新 X 寄存器
        self.registers.x = self.registers.a;
    
        // 更新标志寄存器
        self.check_zsflag(self.registers.x);
    
        // 根据寻址模式和是否跨页来更新 CPU 周期
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        self.registers.pc += self.instruction_info.operand_size as u16;
    }

    fn tya (&mut self) {
        // 更新 A 寄存器
        self.registers.a = self.registers.y;
    
        // 更新标志寄存器
        self.check_zsflag(self.registers.a);
    
        // 根据寻址模式和是否跨页来更新 CPU 周期
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        self.registers.pc += self.instruction_info.operand_size as u16;
    }

    fn txa (&mut self) {
        // 更新 A 寄存器
        self.registers.a = self.registers.x;
    
        // 更新标志寄存器
        self.check_zsflag(self.registers.a);
    
        // 根据寻址模式和是否跨页来更新 CPU 周期
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        self.registers.pc += self.instruction_info.operand_size as u16;
    }

    fn tsx (&mut self) {
        // 更新 X 寄存器
        self.registers.x = self.registers.sp;
    
        // 更新标志寄存器
        self.check_zsflag(self.registers.x);
    
        // 根据寻址模式和是否跨页来更新 CPU 周期
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        self.registers.pc += self.instruction_info.operand_size as u16;
    }

    fn txs (&mut self) {
        // 更新 SP 寄存器
        self.registers.sp = self.registers.x;
    
        // 根据寻址模式和是否跨页来更新 CPU 周期
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        self.registers.pc += self.instruction_info.operand_size as u16;
    }

    fn rti (&mut self) {
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
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
    }

    fn lsr(&mut self){
        let (operand_address,page_crossed) = self.get_operand_address();
        self.registers.pc += self.instruction_info.operand_size as u16;
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        match self.instruction_info.addressing_mode {
            AddressingMode::Accumulator =>{
                let mut value = self.registers.a;
                self.registers.set_flag(StatusFlags::Carry, value & 0x01 == 0x01);
                value >>= 1;
                self.check_zsflag(value);
                self.registers.a = value;
            }
            AddressingMode::ZeroPage|AddressingMode::ZeroPageX|AddressingMode::Absolute|AddressingMode::AbsoluteX =>{
                let mut value = self.read(operand_address);
                self.registers.set_flag(StatusFlags::Carry, value & 0x01 == 0x01);
                value >>= 1;
                self.check_zsflag(value);
                self.write(operand_address, value);
            }
            _=>panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::LSR, self.instruction_info.addressing_mode),
        }
    }

    fn asl (&mut self) {
        let (operand_address,page_crossed) = self.get_operand_address();
        self.registers.pc += self.instruction_info.operand_size as u16;
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        match self.instruction_info.addressing_mode {
            AddressingMode::Accumulator =>{
                let mut value = self.registers.a;
                self.registers.set_flag(StatusFlags::Carry, value & 0x80 == 0x80);
                value <<= 1;
                self.check_zsflag(value);
                self.registers.a = value;
            }
            AddressingMode::ZeroPage|AddressingMode::ZeroPageX|AddressingMode::Absolute|AddressingMode::AbsoluteX =>{
                let mut value = self.read(operand_address);
                self.registers.set_flag(StatusFlags::Carry, value & 0x80 == 0x80);
                value <<= 1;
                self.check_zsflag(value);
                self.write(operand_address, value);
            }
            _=>panic!("当前指令:{:?} 不存在寻址模式{:?}", Instruction::ASL, self.instruction_info.addressing_mode),
        }
    }

    fn ror(&mut self) {
        let (operand_address,page_crossed) = self.get_operand_address();
        let operand = match self.instruction_info.addressing_mode {
            AddressingMode::Accumulator => self.registers.a,
            _ => self.read(operand_address),
        };
    
        // 将操作数最低位旋转到C标志位
        let carry = operand & 1;
    
        // 向右旋转操作数
        let result = (operand >> 1) | (self.registers.get_flag(StatusFlags::Carry) as u8) << 7;
    
        match self.instruction_info.addressing_mode {
            AddressingMode::Accumulator => self.registers.a = result,
            _ => self.write(operand_address, result),
        };
    
        // 更新 C 和 Z 标志位
        self.registers.set_flag(StatusFlags::Carry, carry != 0);
        self.check_zsflag(result);
    
        // 更新 CPU 周期
        self.registers.pc += self.instruction_info.operand_size as u16;
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
    }

    fn rol(&mut self) {
        let (operand_address,page_crossed) = self.get_operand_address();
        let operand = match self.instruction_info.addressing_mode {
            AddressingMode::Accumulator => self.registers.a,
            _ => self.read(operand_address),
        };
    
        // 将操作数最高位旋转到C标志位
        let carry = operand & 0x80;
    
        // 向左旋转操作数
        let result = (operand << 1) | (self.registers.get_flag(StatusFlags::Carry) as u8);
    
        match self.instruction_info.addressing_mode {
            AddressingMode::Accumulator => self.registers.a = result,
            _ => self.write(operand_address, result),
        };
    
        // 更新 C 和 Z 标志位
        self.registers.set_flag(StatusFlags::Carry, carry != 0);
        self.check_zsflag(result);
    
        // 更新 CPU 周期
        self.registers.pc += self.instruction_info.operand_size as u16;
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
    }
    
    fn sty(&mut self){
        let (operand_address,page_crossed) = self.get_operand_address();
        self.write(operand_address, self.registers.y);
        self.registers.pc += self.instruction_info.operand_size as u16;
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
    }

    fn inc(&mut self){
        let (operand_address,page_crossed) = self.get_operand_address();
        let operand = self.read(operand_address);
        let result = operand.wrapping_add(1);
        self.check_zsflag(result);
        self.write(operand_address, result);
        self.registers.pc += self.instruction_info.operand_size as u16;
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
    }

    fn dec(&mut self){
        let (operand_address,page_crossed) = self.get_operand_address();
        let operand = self.read(operand_address);
        let result = operand.wrapping_sub(1);
        self.check_zsflag(result);
        self.write(operand_address, result);
        self.registers.pc += self.instruction_info.operand_size as u16;
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
    }

    fn lax(&mut self){
        let (operand_address,page_crossed) = self.get_operand_address();
        let operand = self.read(operand_address);
        self.check_zsflag(operand);
        self.registers.a = operand;
        self.registers.x = operand;

        self.registers.pc += self.instruction_info.operand_size as u16;
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
        if page_crossed {
            self.cpu_cycle += 1;
        }
    }

    fn sax(&mut self){
        let (operand_address,page_crossed) = self.get_operand_address();

        let value = self.registers.x & self.registers.a;
        self.write(operand_address, value);

        self.registers.pc += self.instruction_info.operand_size as u16;
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
    }

    fn dcp(&mut self){
        let (operand_address,page_crossed) = self.get_operand_address();
        let operand = self.read(operand_address);

        let value = operand.wrapping_sub(1);
        self.write(operand_address,value);

        let result16 = (self.registers.a as u16).wrapping_sub(value as u16);
        self.registers.set_flag(StatusFlags::Carry, result16<0x100);
        self.check_zsflag(result16 as u8);

        self.registers.pc += self.instruction_info.operand_size as u16;
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;

    }

    fn isc(&mut self){
        let (operand_address,page_crossed) = self.get_operand_address();
        let operand = self.read(operand_address);


        // INC
        let result = operand.wrapping_add(1);
        self.write(operand_address, result);


        // SBC
        let acc = self.registers.a;
        // 获取借位标志
        let borrow = if self.registers.get_flag(StatusFlags::Carry) { 0 } else { 1 };

        // 执行减法操作
        let result = acc.wrapping_sub(result).wrapping_sub(borrow);

        // 更新状态寄存器
        self.registers.set_flag(StatusFlags::Carry, (acc as i16 - operand as i16 - borrow as i16) >= 0);
        self.check_zsflag(result);
        self.registers.set_flag(StatusFlags::Overflow, (((acc ^ operand) & (acc ^ result)) & 0x80) != 0);

        // 更新累加器寄存器
        self.registers.a = result;


        self.registers.pc += self.instruction_info.operand_size as u16;
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64;
    }

    fn slo(&mut self){
        let (operand_address,page_crossed) = self.get_operand_address();

        // ASL
        let mut value = self.read(operand_address);
        self.registers.set_flag(StatusFlags::Carry, value & 0x80 == 0x80);
        value <<= 1;
        self.write(operand_address, value);

        self.registers.a|=value;
        self.check_zsflag(self.registers.a);

        self.registers.pc += self.instruction_info.operand_size as u16;
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64; 
    }

    fn rla(&mut self){
        let (operand_address,page_crossed) = self.get_operand_address();

        // rol
        let operand = self.read(operand_address);
    
        // 将操作数最高位旋转到C标志位
        let carry = operand & 0x80;
    
        // 向左旋转操作数
        let result = (operand << 1) | (self.registers.get_flag(StatusFlags::Carry) as u8);
    
        self.write(operand_address, result);
    
        // 更新 C 和 Z 标志位
        self.registers.set_flag(StatusFlags::Carry, carry != 0);

        // and 
        self.registers.a&=result;
        self.check_zsflag(self.registers.a);


        self.registers.pc += self.instruction_info.operand_size as u16;
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64; 
    }

    fn sre(&mut self){
        let (operand_address,page_crossed) = self.get_operand_address();

        // LSR
        let mut value = self.read(operand_address);
        self.registers.set_flag(StatusFlags::Carry, value & 0x01 == 0x01);
        value >>= 1;
        self.write(operand_address, value);

        // EOR
        self.registers.a^=value;
        self.check_zsflag(self.registers.a);

        self.registers.pc += self.instruction_info.operand_size as u16;
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64; 
    }

    fn rra(&mut self){
        let (operand_address,page_crossed) = self.get_operand_address();

        // ROR
        let operand = self.read(operand_address);
    
        // 将操作数最低位旋转到C标志位
        let carry = operand & 1;
    
        // 向右旋转操作数
        let result = (operand >> 1) | (self.registers.get_flag(StatusFlags::Carry) as u8) << 7;
    
        self.write(operand_address, result);
    
        // 更新 C  标志位
        self.registers.set_flag(StatusFlags::Carry, carry != 0);

        // ADC
        // 根据寻址模式获取操作数值
        let operand = result;
    
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
        
        self.check_zsflag(self.registers.a);

        self.registers.pc += self.instruction_info.operand_size as u16;
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64; 

    }

    fn brk (&mut self){
        self.registers.pc+=2;
        self.stack_push_16(self.registers.pc);
        self.stack_push(self.registers.p);
        self.registers.set_flag(StatusFlags::InterruptDisable, true);
        self.registers.set_flag(StatusFlags::BreakCommand, true);
        self.registers.pc = self.read_u16(0xFFFE);
        self.cpu_cycle+=7;
    }

    fn cli (&mut self){
        self.registers.set_flag(StatusFlags::InterruptDisable, false);
        self.registers.pc += self.instruction_info.operand_size as u16;
        self.cpu_cycle+=self.instruction_info.instruction_cycle as u64; 
    }
    // ... 实现其他指令
}

 