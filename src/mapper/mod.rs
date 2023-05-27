// src/memory/mod.rs

pub mod mapper000;

use crate::mapper::mapper000::{create_mapper, Mapper, NromMapper};
use crate::bus::{RWMessage,RWResult,RWType};
use crossbeam::channel::{bounded, select, Receiver, Sender};
use std::thread;

pub const PRG_ROM_BANK_SIZE: usize = 0x4000;
pub const CHR_ROM_BANK_SIZE: usize = 0x2000;


pub struct RomHeader {
    pub prg_rom_size: usize,
    pub chr_rom_size: usize,
    pub mapper_number: u8,
    pub mirroring_type: u8,
    pub battery_backed_ram: bool,
    pub trainer: bool,
    pub nes2_0: bool,
}

impl Default for RomHeader {
    fn default() -> Self {
        RomHeader {
            prg_rom_size: 0,
            chr_rom_size: 0,
            mapper_number: 0,
            mirroring_type: 0,
            battery_backed_ram: false,
            trainer: false,
            nes2_0: false,
        }
    }
}

pub struct InterruptVectors {
    pub nmi_vector: u16,
    pub reset_vector: u16,
    pub irq_vector: u16,
}

impl std::default::Default for InterruptVectors {
    fn default() -> Self {
        InterruptVectors {
            nmi_vector: 0,
            reset_vector: 0,
            irq_vector: 0,
        }
    }
}

pub struct Memory {
    ram: [u8; 0x800],             // 2KB RAM
    sram: [u8; 8192],             // 8KB 存档 SRAM
    ppu_registers: [u8; 0x8],     // PPU 寄存器
    apu_io_registers: [u8; 0x20], // APU 和 I/O 寄存器
    mapper: Box<dyn Mapper>,      // mapper 对象，处理卡带相关的内存映射和访问
    interrupt_vectors: InterruptVectors,
   pub channels: MemChannels,
}

pub struct MemChannels {
    bus2mem_out: Receiver<RWMessage>,
    mem2bus_in: Sender<RWResult>,
    rom2mem_out: Receiver<Vec<u8>>,
}


pub fn start_mem_thread(bus2mem_out:Receiver<RWMessage>,mem2bus_in:Sender<RWResult>,rom2mem_out:Receiver<Vec<u8>>) {
    let mut memory = Memory::new(bus2mem_out,mem2bus_in,rom2mem_out);
    let mut is_success = false;
    let mut data = None;
    thread::spawn(move || {
        loop{
            select! {
                recv(memory.channels.rom2mem_out) -> msg => {
                    let rom = msg.expect("接收rom时发生错误");
                    println!("开始加载rom");
                    memory.reset();
                    memory.load_rom(rom);
                }
                recv(memory.channels.bus2mem_out) -> msg =>{
                    let msg = msg.expect("接收读写管道时发生错误");
                    match msg.operate_type {
                        RWType::Read => {
                            data = Some(memory.read(msg.address));
                            is_success = true;
                        }
                        RWType::Write => {
                            memory.write(msg.address, msg.value.expect("写信息中未能找到具体数值"));
                            is_success = true;
                        }
                    }
                    memory.channels
                        .mem2bus_in
                        .send(RWResult { data, is_success}).expect("发送读写信息时发生错误");
                }
            }
        }
    });
}

impl Memory {
    pub fn new(bus2mem_out:Receiver<RWMessage>,mem2bus_in:Sender<RWResult>,rom2mem_out:Receiver<Vec<u8>>) -> Self {
        let default_mapper: Box<dyn Mapper> =
        create_mapper(RomHeader::default(), vec![0; 3], vec![0; 3]);
        Memory {
            ram: [0; 0x800],
            sram: [0; 8192],
            ppu_registers: [0; 0x8],
            apu_io_registers: [0x00; 0x20],
            mapper: default_mapper,
            interrupt_vectors: InterruptVectors::default(),
            channels: MemChannels {
                mem2bus_in,
                bus2mem_out,
                rom2mem_out,
            },
        }
    }
    // 从内存地址读取一个字节
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                //高三位为0:  系统主内存, 4次镜像
                let ram_addr = addr & 0x07FF;
                self.ram[ram_addr as usize]
            }
            0x2000..=0x3FFF => {
                //高三位为1:  PPU 寄存器, 8字节步进镜像
                let ppu_register_addr = (addr & 0x0007) as usize;
                self.ppu_registers[ppu_register_addr]
            }
            0x4000..=0x401F => {
                //高三位为2:  APU 寄存器
                self.apu_io_registers[(addr - 0x4000) as usize]
            }
            0x4020..=0x5FFF => {
                //高三位为3:  扩展 ROM
                unimplemented!()
            }
            0x6000..=0x7FFF => {
                //高三位为4: 存档 SRAM
                self.sram[(addr - 0x6000) as usize]
            }
            0x8000..=0xFFFF => {
                //高三位为5:  PRG-ROM
                self.mapper.read_prg_rom(addr)
            }
            _ => 0, // 不可能的地址范围
        }
    }

    // 向内存地址写入一个字节
    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            // 0x0000 - 0x1FFF: RAM (2KB, 但前 0x800 字节镜像 3 次)
            0x0000..=0x1FFF => {
                let ram_addr = addr & 0x07FF;

                self.ram[ram_addr as usize] = data;
            }
            // 0x2000 - 0x3FFF: PPU 寄存器 (8 字节镜像，每 0x8 个地址有一个寄存器)
            0x2000..=0x3FFF => {
                let ppu_register_addr = (addr & 0x0007) as usize;
                self.ppu_registers[ppu_register_addr] = data;
            }
            // 0x4000 - 0x401F: APU 和 I/O 寄存器
            0x4000..=0x401F => {
                self.apu_io_registers[(addr - 0x4000) as usize] = data;
            }
            0x4020..=0x5FFF => {
                //高三位为3:  扩展 ROM
                unimplemented!()
            }
            0x6000..=0x7FFF => {
                //高三位为4: 存档 SRAM
                self.sram[(addr - 0x6000) as usize] = data;
            }
            // 0x4020 - 0xFFFF: Mapper 寄存器，卡带相关内存区域
            0x8000..=0xFFFF => {
                // 使用 mapper 对象处理卡带相关的内存写入操作
                self.mapper.write_prg_rom(addr, data);
            }
            _ => {} // 其余地址还未实现
        }
    }

    pub fn load_rom(&mut self, rom_data: Vec<u8>) {
        // 解析ROM文件头
        let rom_header = parse_rom_header(&rom_data);

        // 提取PRG-ROM和CHR-ROM数据
        let (prg_rom, chr_rom) = parse_prg_and_chr_rom_data(&rom_data);

        // 提取中断信息
        self.interrupt_vectors = parse_interrupt_vectors(&prg_rom);

        // 根据ROM文件头信息创建一个适当的映射器（Mapper）实例
        let mapper = create_mapper(rom_header, prg_rom, chr_rom);

        // 将创建的映射器实例存储在 Memory 结构体中
        self.mapper = mapper;
    }

    pub fn reset(&mut self) {
        // 重置 RAM
        self.ram = [0; 0x800];
        // 重置 SRAM
        self.sram = [0; 8192];
        // 重置 PPU 寄存器
        self.ppu_registers = [0; 0x8];
        // 重置 APU 和 I/O 寄存器
        self.apu_io_registers = [0xFF; 0x20];
        // 重置 Mapper
        self.mapper.reset();
    }
}


pub fn parse_rom_header(rom_data: &[u8]) -> RomHeader {
    let prg_rom_size = rom_data[4] as usize * 16 * 1024;
    let chr_rom_size = rom_data[5] as usize * 8 * 1024;
    let mapper_number = (rom_data[6] >> 4) | (rom_data[7] & 0xF0);
    let mirroring_type = rom_data[6] & 0x01;
    let battery_backed_ram = (rom_data[6] & 0x02) != 0;
    let trainer = (rom_data[6] & 0x04) != 0;
    let nes2_0 = (rom_data[7] & 0x0C) == 0x08;

    RomHeader {
        prg_rom_size,
        chr_rom_size,
        mapper_number,
        mirroring_type,
        battery_backed_ram,
        trainer,
        nes2_0,
    }
}

// 解析中断向量
pub fn parse_interrupt_vectors(prg_rom: &Vec<u8>) -> InterruptVectors {
    let nmi_vector = u16::from_le_bytes([prg_rom[prg_rom.len() - 6], prg_rom[prg_rom.len() - 5]]);
    let reset_vector = u16::from_le_bytes([prg_rom[prg_rom.len() - 4], prg_rom[prg_rom.len() - 3]]);
    let irq_vector = u16::from_le_bytes([prg_rom[prg_rom.len() - 2], prg_rom[prg_rom.len() - 1]]);

    InterruptVectors {
        nmi_vector,
        reset_vector,
        irq_vector,
    }
}

pub fn parse_prg_and_chr_rom_data(rom_data: &[u8]) -> (Vec<u8>, Vec<u8>) {
    // 从ROM文件头部信息中提取PRG ROM和CHR ROM大小
    let prg_rom_size = rom_data[4] as usize * 16 * 1024;
    let chr_rom_size = rom_data[5] as usize * 8 * 1024;

    // 如果文件是NES 2.0格式，还需要考虑扩展字段中的额外数据
    let byte9 = rom_data[9];
    let prg_rom_size_upper = ((byte9 & 0x0F) as usize) << 8;
    let chr_rom_size_upper = ((byte9 & 0xF0) as usize) << 4;

    let prg_rom_size = prg_rom_size + prg_rom_size_upper * 16 * 1024;
    let chr_rom_size = chr_rom_size + chr_rom_size_upper * 8 * 1024;

    // 从ROM文件中提取PRG ROM和CHR ROM数据
    let prg_rom_start = 16; // NES 2.0 ROM文件头部信息占用16字节
    let prg_rom_end = prg_rom_start + prg_rom_size;
    let chr_rom_end = prg_rom_end + chr_rom_size;

    let prg_rom = rom_data[prg_rom_start..prg_rom_end].to_vec();
    let chr_rom = rom_data[prg_rom_end..chr_rom_end].to_vec();

    (prg_rom, chr_rom)
}
