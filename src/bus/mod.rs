mod cpu_ram;
mod palettes;
mod registers;
mod vram;

use std::{default, thread};

use crossbeam::channel::{bounded, select, Receiver, Sender};

use crate::mapper::{Mapper, create_mapper};


pub struct RWMessage {
    pub operate_type: RWType,
    pub address: u16,
    pub value: Option<u8>,
}

pub enum RWType {
    Read,
    Write,
}

pub struct RWResult {
    pub data: Option<u8>,
    pub is_success: bool,
}

pub struct InterruptVectors {
    pub nmi_vector: u16,
    pub reset_vector: u16,
    pub irq_vector: u16,
}

#[derive(Clone)]
pub struct CpuBus {
    cpu2bus_out: Receiver<RWMessage>,
    pub cpu2bus_in: Sender<RWMessage>,
    bus2cpu_in: Sender<RWResult>,
    pub bus2cpu_out: Receiver<RWResult>,
}

#[derive(Clone)]
pub struct PpuBus {
    ppu2bus_out: Receiver<RWMessage>,
    pub ppu2bus_in: Sender<RWMessage>,
    bus2ppu_in: Sender<RWResult>,
    pub bus2ppu_out: Receiver<RWResult>,
}

pub struct Bus {
    cpu_ram: cpu_ram::CpuRam,
    palettes: palettes::Palettes,
    registers: registers::Registers,
    vram: vram::Vram,
    apu_io_registers: [u8; 0x20],
    mapper: Box<dyn Mapper>,
}

impl Bus {
    pub fn new() -> Self {
        let default_mapper = Box::new(crate::mapper::mapper000::NromMapper::new(vec![0,0], vec![0,0], 1));
        Bus {
            cpu_ram: cpu_ram::CpuRam::new(),
            palettes: palettes::Palettes::new(),
            registers: registers::Registers::new(),
            vram: vram::Vram::new(),
            apu_io_registers: [0x00; 0x20],
            mapper: default_mapper,
        }
    }

    pub fn reset(&mut self) {
        self.cpu_ram.reset();
        // self.palettes.reset();
        self.registers.reset();
        self.vram.reset();
        self.apu_io_registers = [0xFF; 0x20]; // debug
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        self.mapper= create_mapper(&rom);
    }

    pub fn cpu_read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                //高三位为0:  系统主内存
                self.cpu_ram.read(addr)
            }
            0x2000..=0x3FFF => {
                //高三位为1:  PPU 寄存器
                self.registers.read(addr)
            }
            0x4000..=0x401F => {
                //高三位为2:  APU 寄存器
                self.apu_io_registers[(addr - 0x4000) as usize]
            }
            0x4020..=0x5FFF => {
                //高三位为3:  扩展 ROM
                todo!()
            }
            0x6000..=0x7FFF => {
                //高三位为4: 存档 SRAM
                todo!()
            }
            0x8000..=0xFFFF => {
                //高三位为5:  PRG-ROM
                self.mapper.read_prg_rom(addr)
            }
            _ => 0, // 不可能的地址范围
        }
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        match addr {
            // 0x0000 - 0x1FFF: RAM (2KB, 但前 0x800 字节镜像 3 次)
            0x0000..=0x1FFF => {
                self.cpu_ram.write(addr, data);
            }
            // 0x2000 - 0x3FFF: PPU 寄存器 (8 字节镜像，每 0x8 个地址有一个寄存器)
            0x2000..=0x3FFF => {
                self.registers.write(addr, data);
            }
            // 0x4000 - 0x401F: APU 和 I/O 寄存器
            0x4000..=0x401F => {
                self.apu_io_registers[(addr - 0x4000) as usize] = data;
            }
            0x4020..=0x5FFF => {
                //高三位为3:  扩展 ROM
                todo!()
            }
            0x6000..=0x7FFF => {
                //高三位为4: 存档 SRAM
                todo!()
            }
            // 0x4020 - 0xFFFF: Mapper 寄存器，卡带相关内存区域
            0x8000..=0xFFFF => {
                // 使用 mapper 对象处理卡带相关的内存写入操作
                self.mapper.write_prg_rom(addr, data);
            }
            _ => {} // 其余地址还未实现
        }
    }

    pub fn ppu_read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                // pattern table
                self.mapper.read_chr_rom(addr)
            }
            0x2000..=0x3EFF => {
                //高三位为1:  PPU 寄存器
                self.vram.read(addr)
            }
            0x3F00..=0x3FFF => {
                //高三位为2:  APU 寄存器
                self.palettes.read(addr)
            }
            0x4000..=0xFFFF => {
                //mirror
                self.ppu_read(addr & 0x4000)
            }
            _ => 0, // 不可能的地址范围
        }
    }

    pub fn ppu_write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1FFF => {
                // pattern table
                self.mapper.write_chr_rom(addr, data);
            }
            0x2000..=0x3EFF => {
                //高三位为1:  PPU 寄存器
                self.vram.write(addr, data);
            }
            0x3F00..=0x3FFF => {
                //高三位为2:  APU 寄存器
                self.palettes.write(addr, data);
            }
            0x4000..=0xFFFF => {
                //mirror
                self.ppu_write(addr & 0x4000, data);
            }
            _ => {} // 不可能的地址范围
        }
    }
    
    pub fn apu_read(&self, addr: u16) -> u8 {
        todo!()
    }
    
    pub fn apu_write(&mut self, addr: u16, data: u8) {
        todo!()
    }
}

pub fn start_bus_thread(bus2cpu:Sender<RWResult>,cpu2bus:Receiver<RWMessage>,bus2ppu:Sender<RWResult>,ppu2bus:Receiver<RWMessage>,bus2apu:Sender<RWResult>,apu2bus:Receiver<RWMessage>,rom2bus:Receiver<Vec<u8>>) {
    let mut bus = Bus::new();
    let mut is_success = false;
    let mut data = None;
    thread::spawn(move || {
        loop{
            select! {
                recv(rom2bus) -> msg => {
                    let rom = msg.expect("接收rom时发生错误");
                    println!("开始加载rom");
                    bus.reset();
                    bus.load_rom(rom);
                }
                recv(cpu2bus) -> msg =>{
                    let msg = msg.expect("接收读写请求时发生错误");
                    match msg.operate_type {
                        RWType::Read => {
                            data = Some(bus.cpu_read(msg.address));
                            is_success = true;
                        }
                        RWType::Write => {
                            bus.cpu_write(msg.address,msg.value.expect("写信息中未能找到具体数值"));
                            is_success = true;
                        }
                    }
                    bus2cpu.send(RWResult{data,is_success}).expect("发送读写结果时发生错误")
                }
                recv(ppu2bus) -> msg =>{
                    let msg = msg.expect("接收读写请求时发生错误");
                    match msg.operate_type {
                        RWType::Read => {
                            data = Some(bus.ppu_read(msg.address));
                            is_success = true;
                        }
                        RWType::Write => {
                            bus.ppu_write(msg.address,msg.value.expect("写信息中未能找到具体数值"));
                            is_success = true;
                        }
                    }
                    bus2ppu.send(RWResult{data,is_success}).expect("发送读写结果时发生错误")
                }
                recv(apu2bus) -> msg =>{
                    let msg = msg.expect("接收读写请求时发生错误");
                    match msg.operate_type {
                        RWType::Read => {
                            data = Some(bus.apu_read(msg.address));
                            is_success = true;

                        }
                        RWType::Write => {
                            bus.apu_write(msg.address,msg.value.expect("写信息中未能找到具体数值"));
                            is_success = true;
                        }
                    }
                    bus2apu.send(RWResult{data,is_success}).expect("发送读写结果时发生错误")
                }
            }
        }
    });
}
