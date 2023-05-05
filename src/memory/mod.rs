// src/memory/mod.rs

pub mod mapper;

use mapper::{Mapper,NromMapper};

pub const PRG_ROM_BANK_SIZE: usize = 0x4000;
pub const CHR_ROM_BANK_SIZE: usize = 0x2000;

pub struct Memory {
    pub mapper: Box<dyn Mapper>,
}

impl Memory {
    pub fn new(mapper: Box<dyn Mapper>) -> Self {
        Memory { mapper }
    }

    // 读取8位数据
    pub fn read_byte(&self, address: u16) -> u8 {
        self.mapper.read(address)
    }

    // 写入8位数据
    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.mapper.write(address, value);
    }
}

pub fn create_mapper(mapper_id: u8, prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Box<dyn Mapper> {
    match mapper_id {
        0 => Box::new(NromMapper::new(prg_rom, chr_rom)),
        // 1 => Box::new(Mmc1Mapper::new(prg_rom, chr_rom)),
        // 在这里添加其他映射器类型的实例创建代码
        _ => panic!("Unsupported mapper ID: {}", mapper_id),
    }
}
