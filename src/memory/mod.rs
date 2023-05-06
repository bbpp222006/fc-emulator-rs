// src/memory/mod.rs

pub mod mapper;

use mapper::{Mapper,NromMapper};

pub const PRG_ROM_BANK_SIZE: usize = 0x4000;
pub const CHR_ROM_BANK_SIZE: usize = 0x2000;

pub struct Memory {
    pub mapper: Box<dyn Mapper>,
}

impl Memory {
    // Memory 构造函数
    pub fn new(mapper: Box<dyn Mapper>) -> Self {
        Memory { mapper }
    }

    // 从内存地址读取一个字节
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                // CPU 内存，未实现
                0
            }
            0x8000..=0xFFFF => {
                // PRG-ROM
                self.mapper.read_prg_rom(addr)
            }
            _ => 0, // 不可能的地址范围
        }
    }

    // 向内存地址写入一个字节
    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1FFF => {
                // CPU 内存，未实现
            }
            0x8000..=0xFFFF => {
                // PRG-ROM
                self.mapper.write_prg_rom(addr, data);
            }
            _ => {} // 不可能的地址范围
        }
    }

    // 从内存地址读取一个 16 位字
    pub fn read_u16(&self, address: u16) -> u16 {
        let low_byte = self.read(address) as u16;
        let high_byte = self.read(address + 1) as u16;
        (high_byte << 8) | low_byte
    }


     // 用于单元测试的 from_data 方法
     pub fn from_data(data: Vec<u8>) -> Self {
        // 创建一个简单的 NROM Mapper 实例
        let mapper = Box::new(NromMapper::new(data, vec![], 0));

        // 使用 NROM Mapper 实例创建一个 Memory 实例
        Memory { mapper }
    }
}

pub fn create_mapper(
    mapper_id: u8,
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mirror_mode: u8,
) -> Box<dyn Mapper> {
    match mapper_id {
        0 => Box::new(NromMapper::new(prg_rom, chr_rom, mirror_mode)),
        // 在这里添加其他 Mapper 的实现
        _ => panic!("Unsupported mapper ID: {}", mapper_id),
    }
}