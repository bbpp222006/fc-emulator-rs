// src/memory/mod.rs

pub mod mapper;

use crate::memory::mapper::{Mapper,NromMapper,create_mapper};

pub const PRG_ROM_BANK_SIZE: usize = 0x4000;
pub const CHR_ROM_BANK_SIZE: usize = 0x2000;

pub struct RomHeader {
    pub  prg_rom_size: usize,
    pub  chr_rom_size: usize,
    pub  mapper_number: u8,
    pub  mirroring_type: u8,
    pub  battery_backed_ram: bool,
    pub  trainer: bool,
    pub  nes2_0: bool,
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

pub struct Memory {
   pub  ram: [u8; 0x800], // 2KB RAM
    sram: [u8; 8192], // 8KB 存档 SRAM
    ppu_registers: [u8; 0x8], // PPU 寄存器
    apu_io_registers: [u8; 0x20], // APU 和 I/O 寄存器
    mapper: Box<dyn Mapper>, // mapper 对象，处理卡带相关的内存映射和访问
}

impl Memory {
    // Memory 构造函数
    pub fn new(mapper: Box<dyn Mapper>) -> Self {
        Memory { ram: [0; 0x800],sram: [0; 8192], ppu_registers: [0; 0x8], apu_io_registers: [0; 0x20], mapper }
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

    // 从内存地址读取一个 16 位字
    pub fn read_u16(&self, address: u16) -> u16 {
        let low_byte = self.read(address) as u16;
        let high_byte = self.read(address + 1) as u16;
        (high_byte << 8) | low_byte
    }

    // 从内存地址读取一个 16 位字，零页专用
    pub fn read_u16_z(&self, address:u8 ) -> u16 {
        let low_byte = self.read(address as u16) as u16;
        let high_byte = self.read(address.wrapping_add(1) as u16) as u16;
        (high_byte << 8) | low_byte
    }


     // 用于单元测试的 from_data 方法
     pub fn from_data(data: Vec<u8>) -> Self {
        // 创建一个简单的 NROM Mapper 实例
        let mapper = Box::new(NromMapper::new(data, vec![], 0));

        // 使用 NROM Mapper 实例创建一个 Memory 实例
        Memory { ram: [0; 0x800],sram: [0; 8192], ppu_registers: [0; 0x8], apu_io_registers: [0; 0x20], mapper }
    }

    pub fn load_rom(&mut self, rom_data: Vec<u8>) {
        // 解析ROM文件头
        let rom_header = parse_rom_header(&rom_data);

        // 提取PRG-ROM和CHR-ROM数据
        let (prg_rom, chr_rom) = parse_prg_and_chr_rom_data(&rom_data);

        // 根据ROM文件头信息创建一个适当的映射器（Mapper）实例
        let mapper = create_mapper(rom_header, prg_rom, chr_rom);

        // 将创建的映射器实例存储在 Memory 结构体中
        self.mapper = mapper;
    }
}

impl std::default::Default for Memory {
    fn default() -> Self {
        // 使用一个空的 Vec<u8> 创建一个默认的 Mapper 实例
        let default_mapper: Box<dyn Mapper> = create_mapper(
            RomHeader::default(),
            vec![],
            vec![],
        );
        Memory {ram: [0; 0x800],sram: [0; 8192], ppu_registers: [0; 0x8], apu_io_registers: [0; 0x20], mapper: default_mapper}
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
    let irq_brk_vector = u16::from_le_bytes([prg_rom[prg_rom.len() - 2], prg_rom[prg_rom.len() - 1]]);

    InterruptVectors {
        nmi_vector,
        reset_vector,
        irq_vector: irq_brk_vector,
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
