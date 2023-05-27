// mapper.rs

// 引入标准库中的类型和特质
use std::fmt;
use crate::mapper::RomHeader;

// 定义一个通用的 Mapper trait
pub trait Mapper: Send {
    fn read_prg_rom(&self, addr: u16) -> u8;
    fn write_prg_rom(&mut self, addr: u16, data: u8);
    fn read_chr_rom(&self, addr: u16) -> u8;
    fn write_chr_rom(&mut self, addr: u16, data: u8);
    fn ppu_mirror_mode(&self) -> u8;
    fn reset(&mut self);
}


pub fn create_mapper(
    rom_header: RomHeader,
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
) -> Box<dyn Mapper> {
    match rom_header.mapper_number {
        0 => Box::new(NromMapper::new(prg_rom, chr_rom, rom_header.mirroring_type)),
        // 在这里添加其他 Mapper 的实现
        _ => panic!("Unsupported mapper ID: {}", rom_header.mapper_number),
    }
}


// 定义 NromMapper 结构体
#[derive(Debug)]
pub struct NromMapper {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mirror_mode: u8,
}

impl NromMapper {
    // NromMapper 的构造函数
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirror_mode: u8) -> Self {
        NromMapper {
            prg_rom,
            chr_rom,
            mirror_mode,
        }
    }
}

// 为 NromMapper 实现 Mapper trait
impl Mapper for NromMapper {
    fn read_prg_rom(&self, addr: u16) -> u8 {
        // 考虑到 NROM 可能只有一个 16KB 的 PRG-ROM，需要处理镜像
        let addr = addr as usize % self.prg_rom.len();
        self.prg_rom[addr]
    }

    fn write_prg_rom(&mut self, _addr: u16, _data: u8) {
        // NROM 映射器通常不支持 PRG-ROM 写入
        // 你可以在这里添加日志或错误处理，或者什么都不做
    }

    fn read_chr_rom(&self, addr: u16) -> u8 {
        let addr = addr as usize % self.chr_rom.len();
        self.chr_rom[addr]
    }


    fn write_chr_rom(&mut self, _addr: u16, _data: u8) {
        // NROM 映射器的 CHR-ROM 通常不可写
        // 你可以在这里添加日志或错误处理，或者什么都不做
    }

    fn ppu_mirror_mode(&self) -> u8 {
        self.mirror_mode
    }
    
    fn reset(&mut self) {
        // 在这个例子中，NromMapper 没有需要重置的内部状态。
        // 如果有需要重置的状态，你可以在这里进行相应的操作。
    }
}
