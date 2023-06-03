use std::collections::HashSet;
use std::{default, thread};
use crossbeam::channel::{bounded, select, Receiver, Sender};
use egui::Key;
use crate::mapper::{Mapper, create_mapper};
use crate::bus::{vram,cpu_ram,registers,palettes,apu_io_registers};

pub struct RWMessage {
    pub operate_type: RWType,
    pub address: u16,
    pub value: Option<u8>,
}

pub enum RWType {
    Read,
    Write,
    ReadReg,
    WriteReg,
    ReadInerruptStatus,
    WriteInerruptStatus,
}

pub struct RWResult {
    pub data: Option<u8>,
    pub is_success: bool,
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
    // 0b0000_0xxx
    //   |||| |||+-- IRQ
    //   |||| ||+--- VBlank/NMI
    //   |||| |+---- Reset
    interrupt_status: u8, 
    cpu_ram: cpu_ram::CpuRam,
    registers: registers::Registers,
    vram: vram::Vram,
    vram_buffer: u8, // cpu通过PPUDATA 读写VRAM时，需要一个buffer
    palettes: palettes::Palettes,
    apu_io_registers: apu_io_registers::ApuIoRegisters,
    mapper: Box<dyn Mapper>,
}

impl Bus {
    pub fn new(input_stream: Receiver<HashSet<Key>>) -> Self {
        let default_mapper = Box::new(crate::mapper::mapper000::NromMapper::new(vec![0,0], vec![0,0], 1));
        Bus {
            interrupt_status: 0b0000_0000,
            cpu_ram: cpu_ram::CpuRam::new(),
            registers: registers::Registers::new(),
            vram: vram::Vram::new(),
            vram_buffer: 0,
            palettes: palettes::Palettes::new(),
            apu_io_registers: apu_io_registers::ApuIoRegisters::new(input_stream),
            mapper: default_mapper,
        }
    }

    pub fn reset(&mut self) {
        self.cpu_ram.reset();
        // self.palettes.reset();
        self.registers.reset();
        self.vram.reset();
        self.apu_io_registers.reset(); // debug
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        self.mapper= create_mapper(&rom);
    }

    pub fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                //高三位为0:  系统主内存
                self.cpu_ram.read(addr)
            }
            0x2000..=0x3FFF => {
                //高三位为1:  PPU 寄存器
                let mut out_data = self.registers.read(addr);
                // 一些附加影响
                match 0x2000+(addr & 0x0007) as usize {
                    0x2007 => {
                        match self.vram.vram_addr {
                            0x2000..=0x3eff => {
                                out_data = self.vram_buffer;
                                // 读取 PPUDATA 寄存器，进行 VRAM 读
                                self.vram_buffer = self.vram.read(self.vram.vram_addr);
                            }
                            0x3f00..=0x3fff => {
                                // 读取 PPUDATA 寄存器，进行调色板读
                                out_data = self.palettes.read(self.vram.vram_addr);
                            }
                            _ => (),
                        }
                        // 读取 PPUDATA 寄存器后，地址会增加 1 或 32，取决于 PPUCTRL 寄存器的第 2 位
                        self.vram.vram_addr+= if self.registers.read(0x2000) & 0b0000_0100 != 0 { 32 } else { 1 };
                    },
                    _ => (),
                }
                out_data
            }
            0x4000..=0x401F => {
                //高三位为2:  APU 寄存器
                self.apu_io_registers.read(addr)
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

                // 一些附加影响
                match 0x2000+(addr & 0x0007) as usize {
                    0x2006 => {
                        // 写入 PPUADDR 寄存器，更改vram_addr
                        self.vram.vram_addr= ((self.vram.vram_addr << 8) & 0xFF00) | (data as u16);
                    },
                    0x2007 => {
                        // 写入 PPUDATA 寄存器，进行 VRAM 写
                        self.vram.write(self.vram.vram_addr, data);
                        // VRAM 地址自增
                        self.vram.vram_addr+= if self.registers.read(0x2000) & 0b0000_0100 != 0 { 32 } else { 1 };
                        // println!("{}", format!("write vram: {:04X} {:02X}", self.vram.vram_addr, data));
                    },
                    _ => (),
                }
            }
            // 0x4000 - 0x401F: APU 和 I/O 寄存器
            0x4000..=0x401F => {
                self.apu_io_registers.write(addr, data)
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

    pub fn ppu_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                // pattern table
                let a = self.mapper.read_chr_rom(addr);
                a
            }
            0x2000..=0x3FFF => {
                // nametable,attribute table,palette table 都在这里
                self.vram.read(addr)
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
                // nametable,attribute table 都在这里
                self.vram.write(addr, data);
            }
            0x3F00..=0x3FFF => {
                // palette table
                self.palettes.write(addr, data);
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

pub fn start_bus_thread(bus2cpu:Sender<RWResult>,cpu2bus:Receiver<RWMessage>,bus2ppu:Sender<RWResult>,ppu2bus:Receiver<RWMessage>,bus2apu:Sender<RWResult>,apu2bus:Receiver<RWMessage>,rom2bus:Receiver<Vec<u8>>,input_stream:Receiver<HashSet<egui::Key>>) {
    let mut bus = Bus::new(input_stream);
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
                        RWType::ReadInerruptStatus => {
                            data = Some(bus.interrupt_status);
                            is_success = true;
                        }
                        RWType::WriteInerruptStatus => {
                            bus.interrupt_status = msg.value.expect("写信息中未能找到具体数值");
                            is_success = true;
                        }
                        _ => {
                            unreachable!("cpu不可能的读写类型")
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
                        RWType::ReadReg => {
                            data = Some(bus.cpu_read(msg.address));
                            is_success = true;
                        }
                        RWType::WriteReg => {
                            bus.cpu_write(msg.address,msg.value.expect("写信息中未能找到具体数值"));
                            is_success = true;
                        }
                        RWType::ReadInerruptStatus => {
                            data = Some(bus.interrupt_status);
                            is_success = true;
                        }
                        RWType::WriteInerruptStatus => {
                            bus.interrupt_status = msg.value.expect("写信息中未能找到具体数值");
                            is_success = true;
                        }
                        _ => {
                            unreachable!("cpu不可能的读写类型")
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
                        _ => {
                            unreachable!("apu不可能的读写类型")
                        }
                    }
                    bus2apu.send(RWResult{data,is_success}).expect("发送读写结果时发生错误")
                }
            }
        }
    });
}
