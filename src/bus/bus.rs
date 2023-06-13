use std::collections::{HashSet, VecDeque};
use std::{default, thread};
use crossbeam::channel::{bounded, select, Receiver, Sender};
use egui::Key;
use crate::mapper::{Mapper, create_mapper};
use crate::bus::{nametable,registers,palettes,apu_io_registers};

use super::{cpu_ram, oam};

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

pub struct Bus {
    // 0b0000_0xxx
    //   |||| |||+-- IRQ
    //   |||| ||+--- VBlank/NMI
    //   |||| |+---- Reset
    pub interrupt_status: u8, 
    pub registers: registers::Registers,
    pub ppustatus_racing: bool, // 用于记录ppustatus的读写竞争状态

    nametable: nametable::Nametable,
    vram_buffer: u8, // cpu通过PPUDATA 读写VRAM时，需要一个buffer
    vram_addr: u16, // cpu通过PPUSCROLL/PPUADDR 读写VRAM时，需要一个addr
    pub oam: oam::Oam,
    palettes: palettes::Palettes,
    pub apu_io_registers: apu_io_registers::ApuIoRegisters,
    mapper: Box<dyn Mapper>,
    cpu_ram: cpu_ram::CpuRam, // debug
}

impl Bus {
    pub fn new(input_stream:Receiver<HashSet<egui::Key>>) -> Self {
        let default_mapper = Box::new(crate::mapper::mapper000::NromMapper::new(vec![0,0], vec![0,0], 1));
        Bus {
            interrupt_status: 0b0000_0000,
            registers: registers::Registers::new(),
            ppustatus_racing: false,
            nametable: nametable::Nametable::new(),
            vram_buffer: 0,
            vram_addr: 0,
            oam: oam::Oam::new(),
            palettes: palettes::Palettes::new(),
            apu_io_registers: apu_io_registers::ApuIoRegisters::new(),
            mapper: default_mapper,
            cpu_ram: cpu_ram::CpuRam::new(), // debug
        }
    }

    pub fn refresh_input(&mut self, new_input: HashSet<egui::Key>) {
        self.apu_io_registers.inpur_reg = new_input;         
    }

    pub fn reset(&mut self) {
        // self.palettes.reset();
        self.registers.reset();
        self.nametable.reset();
        self.oam.reset();
        self.vram_addr = 0;
        self.apu_io_registers.reset(); // debug
        self.cpu_ram.reset(); // debug
        self.mapper.reset(); 
        self.interrupt_status = 0b0000_0000;
        self.vram_buffer=0;
        self.vram_addr=0;
        self.palettes.reset();
    }

    pub fn hard_reset(&mut self) {
        self.reset();

    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        self.mapper= create_mapper(&rom);
    }

    // 无副作用的读，用于调试
    pub fn cpu_read_debug(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1fff => {
                // 系统主内存
                self.cpu_ram.read(addr)
            }
            0x2000..=0x3FFF => {
                //高三位为1:  PPU 寄存器
                let mut out_data = self.registers.read(addr);
                // 一些附加影响
                match 0x2000+(addr & 0x0007) as usize {
                    0x2004 => {
                        // 读取 OAMDATA 寄存器，进行 OAM 读
                        out_data = self.oam.read(self.oam.oam_addr);
                    },
                    0x2007 => {
                        // 读取 PPUDATA 寄存器，进行 VRAM 读
                        match self.vram_addr {
                            0x0..=0x3eff => {
                                out_data = self.vram_buffer;
                            }
                            0x3f00..=0x3fff => {
                                // 调色板,无缓冲
                                out_data = self.palettes.read(self.vram_addr);
                            }
                            _ => (),
                        }
                    },
                    _ => (),
                }
                out_data
            }
            0x4000..=0x401F => {
                //高三位为2:  APU ,io寄存器
                let out_data = self.apu_io_registers.read_debug(addr);
                out_data
            }
            0x4020..=0x5FFF => {
                //高三位为3:  扩展 ROM
                todo!()
            }
            0x6000..=0x7FFF => {
                //高三位为4: 存档 SRAM
                self.mapper.read_prg_ram(addr)
            }
            0x8000..=0xFFFF => {
                //高三位为5:  PRG-ROM
                self.mapper.read_prg_rom(addr)
            }
            _ => 0, // 不可能的地址范围
        }
    }


    pub fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1fff => {
                // 系统主内存
                self.cpu_ram.read(addr)
            }
            0x2000..=0x3FFF => {
                //高三位为1:  PPU 寄存器
                let mut out_data = self.registers.read(addr);
                // 一些附加影响
                match 0x2000+(addr & 0x0007) as usize {
                    0x2002 => {
                        out_data = if self.ppustatus_racing { 
                            self.registers.ppustatus & 0x7F
                        } else {
                            self.registers.ppustatus 
                        };
                        self.registers.ppustatus &= 0x7F; // 读取ppustatus会清除vblank标志
                        // https://www.nesdev.org/wiki/NMI 读写竞争 
                        // https://github.com/christopherpow/nes-test-roms/tree/master/ppu_vbl_nmi/source 测试时需要考虑
                        
                    },
                    0x2004 => {
                        // 读取 OAMDATA 寄存器，进行 OAM 读
                        out_data = self.oam.read(self.oam.oam_addr);
                        // 读取 OAMDATA 寄存器后，地址会增加 1
                        self.oam.oam_addr += 1;
                    },
                    0x2007 => {
                        // 读取 PPUDATA 寄存器，进行 VRAM 读
                        match self.vram_addr {
                            0x0..=0x3eff => {
                                out_data = self.vram_buffer;
                                match self.vram_addr {
                                    0x0 ..= 0x1fff => {
                                        // chr rom
                                        self.vram_buffer = self.mapper.read_chr_rom(self.vram_addr);
                                    }
                                    0x2000..=0x3eff => {
                                        // nametable
                                        self.vram_buffer = self.nametable.read(self.vram_addr);
                                    }
                                    _ => (),
                                }
                            }
                            0x3f00..=0x3fff => {
                                // 调色板,无缓冲
                                out_data = self.palettes.read(self.vram_addr);
                            }
                            _ => (),
                        }
                        // 读取 PPUDATA 寄存器后，地址会增加 1 或 32，取决于 PPUCTRL 寄存器的第 2 位
                        self.vram_addr+= if self.registers.read(0x2000) & 0b0000_0100 != 0 { 32 } else { 1 };
                    },
                    _ => (),
                }
                
                out_data
            }
            0x4000..=0x401F => {
                //高三位为2:  APU ,io寄存器
                let out_data = self.apu_io_registers.read(addr);
                out_data
            }
            0x4020..=0x5FFF => {
                //高三位为3:  扩展 ROM
                todo!()
            }
            0x6000..=0x7FFF => {
                //高三位为4: 存档 SRAM
                println!("read SRAM:{:04X}",addr);
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
            0x0000..=0x1fff => {
                // 系统主内存
                self.cpu_ram.write(addr, data);
            }
            // 0x2000 - 0x3FFF: PPU 寄存器 (8 字节镜像，每 0x8 个地址有一个寄存器)
            0x2000..=0x3FFF => {
                self.registers.write(addr, data);
                // 一些附加影响
                match 0x2000+(addr & 0x0007) as usize {
                    0x2003 => {
                        // 写入 OAMADDR 寄存器，更改 oam_addr
                        self.oam.oam_addr = data as u16;
                    },
                    0x2004 => {
                        // 写入 OAMDATA 寄存器，进行 OAM 写
                        self.oam.write(self.oam.oam_addr, data);
                        // OAM 地址自增
                        self.oam.oam_addr+= 1;
                    },
                    0x2006 => {
                        // 写入 PPUADDR 寄存器，更改vram_addr
                        self.vram_addr= ((self.vram_addr << 8) & 0xFF00) | (data as u16);
                    },
                    0x2007 => {
                        // 写入 PPUDATA 寄存器，进行 VRAM 写
                        match self.vram_addr {
                            0x0..=0x3eff => {
                                match self.vram_addr {
                                    0x0 ..= 0x1fff => {
                                        // chr rom
                                        self.mapper.write_chr_rom(self.vram_addr, data);
                                    }
                                    0x2000..=0x3eff => {
                                        // nametable
                                        self.nametable.write(self.vram_addr, data);
                                    }
                                    _ => (),
                                }
                            }
                            0x3f00..=0x3fff => {
                                // 调色板,无缓冲
                                self.palettes.write(self.vram_addr, data);
                            }
                            _ => (),
                        }
                        // VRAM 地址自增
                        self.vram_addr+= if self.registers.read(0x2000) & 0b0000_0100 != 0 { 32 } else { 1 };
                        // println!("{}", format!("write vram: {:04X} {:02X}", self.vram_addr, data));
                    },
                    _ => (),
                }
            }
            // 0x4000 - 0x401F: APU 和 I/O 寄存器
            0x4000..=0x401F => {
                match addr {
                    0x4014 => {
                        // 写入 OAMDMA 寄存器，进行 OAMDMA 操作
                        let start_addr = (data as u16) << 8;
                        for i in 0..0xff {
                            let data = self.cpu_read(start_addr + i);
                            self.oam.write(i, data);
                        }
                    },
                    _ => {
                        self.apu_io_registers.write(addr, data);
                    },
                }
                
            }
            0x4020..=0x5FFF => {
                //高三位为3:  扩展 ROM
                todo!()
            }
            0x6000..=0x7FFF => {
                //高三位为4: 存档 SRAM
                self.mapper.write_prg_ram(addr, data);
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
            0x2000..=0x3EFF => {
                // nametable,attribute table
                self.nametable.read(addr)
            }
            0x3F00..=0x3FFF => {
                // 调色板
                self.palettes.read(addr)
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
                self.nametable.write(addr, data);
            }
            0x3F00..=0x3FFF => {
                // 调色板
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




// pub fn start_bus_thread(bus2cpu:Sender<RWResult>,cpu2bus:Receiver<RWMessage>,bus2ppu:Sender<RWResult>,ppu2bus:Receiver<RWMessage>,bus2apu:Sender<RWResult>,apu2bus:Receiver<RWMessage>,rom2bus:Receiver<Vec<u8>>,input_stream:Receiver<HashSet<egui::Key>>) {
//     let mut bus = Bus::new();
//     let mut is_success = false;
//     let mut data = None;
//     thread::spawn(move || {
//         loop{
//             select! {
//                 recv(input_stream) -> input => {
//                     match input {
//                         Ok(input) => {
//                             let mut current_input = VecDeque::from([0; 8].to_vec());
//                             for key in input {
//                                 match key {
//                                     Key::J => current_input[0] = 1,
//                                     Key::K => current_input[1] = 1,
//                                     Key::Space => current_input[2] = 1,
//                                     Key::Enter => current_input[3] = 1,
//                                     Key::W => current_input[4] = 1,
//                                     Key::S => current_input[5] = 1,
//                                     Key::A => current_input[6] = 1,
//                                     Key::D => current_input[7] = 1,
//                                     _ => {}
//                                 }
//                             }
//                             if bus.apu_io_registers.input_enable {
//                                 // println!("接收到输入{:?}",current_input);
//                                 bus.apu_io_registers.current_input = current_input;
//                             }
//                         },
//                         Err(_) => {}
//                     }
//                 }
//                 recv(rom2bus) -> msg => {
//                     let rom = msg.expect("接收rom时发生错误");
//                     println!("开始加载rom");
//                     bus.reset();
//                     bus.load_rom(rom);
//                 }
//                 recv(cpu2bus) -> msg =>{
//                     let msg = msg.expect("接收读写请求时发生错误");
//                     match msg.operate_type {
//                         RWType::Read => {
//                             data = Some(bus.cpu_read(msg.address));
//                             is_success = true;
//                         }
//                         RWType::Write => {
//                             bus.cpu_write(msg.address,msg.value.expect("写信息中未能找到具体数值"));
//                             is_success = true;
//                         }
//                         RWType::ReadInerruptStatus => {
//                             data = Some(bus.interrupt_status);
//                             is_success = true;
//                         }
//                         RWType::WriteInerruptStatus => {
//                             bus.interrupt_status = msg.value.expect("写信息中未能找到具体数值");
//                             is_success = true;
//                         }
//                         _ => {
//                             unreachable!("cpu不可能的读写类型")
//                         }
//                     }
//                     bus2cpu.send(RWResult{data,is_success}).expect("发送读写结果时发生错误")
//                 }
//                 recv(ppu2bus) -> msg =>{
//                     let msg = msg.expect("接收读写请求时发生错误");
//                     match msg.operate_type {
//                         RWType::Read => {
//                             data = Some(bus.ppu_read(msg.address));
//                             is_success = true;
//                         }
//                         RWType::Write => {
//                             bus.ppu_write(msg.address,msg.value.expect("写信息中未能找到具体数值"));
//                             is_success = true;
//                         }
//                         RWType::ReadReg => {
//                             data = Some(bus.cpu_read(msg.address));
//                             is_success = true;
//                         }
//                         RWType::WriteReg => {
//                             bus.cpu_write(msg.address,msg.value.expect("写信息中未能找到具体数值"));
//                             is_success = true;
//                         }
//                         RWType::ReadInerruptStatus => {
//                             data = Some(bus.interrupt_status);
//                             is_success = true;
//                         }
//                         RWType::WriteInerruptStatus => {
//                             bus.interrupt_status = msg.value.expect("写信息中未能找到具体数值");
//                             is_success = true;
//                         }
//                         _ => {
//                             unreachable!("cpu不可能的读写类型")
//                         }
//                     }
//                     bus2ppu.send(RWResult{data,is_success}).expect("发送读写结果时发生错误")
//                 }
//                 recv(apu2bus) -> msg =>{
//                     let msg = msg.expect("接收读写请求时发生错误");
//                     match msg.operate_type {
//                         RWType::Read => {
//                             data = Some(bus.apu_read(msg.address));
//                             is_success = true;
//                         }
//                         RWType::Write => {
//                             bus.apu_write(msg.address,msg.value.expect("写信息中未能找到具体数值"));
//                             is_success = true;
//                         }
//                         _ => {
//                             unreachable!("apu不可能的读写类型")
//                         }
//                     }
//                     bus2apu.send(RWResult{data,is_success}).expect("发送读写结果时发生错误")
//                 }
//             }
//         }
//     });
// }
