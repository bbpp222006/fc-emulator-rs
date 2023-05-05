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
    pub fn read_byte(&self, addr: u16) -> u8 {
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
    pub fn write_byte(&mut self, addr: u16, data: u8) {
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