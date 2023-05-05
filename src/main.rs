// src/main.rs

use std::fs::File;
use std::io::Read;

// 导入所需的模块
mod memory;
use memory::{create_mapper, Memory};

// ... 略过其他代码 ...
#[derive(Debug)]
struct RomHeader {
    prg_rom_size: usize,
    chr_rom_size: usize,
    mapper_number: u8,
    mirroring_type: u8,
    battery_backed_ram: bool,
    trainer: bool,
    nes2_0: bool,
}


fn main() {
    // ... 略过其他代码 ...

    // 读取ROM文件
    let mut rom_file = File::open("rom/nestest.nes").expect("Cannot open ROM file");
    let mut rom_data = Vec::new();
    rom_file.read_to_end(&mut rom_data).expect("Cannot read ROM file");

    // 解析ROM文件头部信息，获取所需数据，如映射器ID、PRG ROM数据和CHR ROM数据
    let rom_header = parse_rom_header(&rom_data);

    // 打印文件头信息
    println!("prg_rom_size: {} x 16kb, chr_rom_size: {} x 8kb, mapper_id: {}",
             rom_header.prg_rom_size / 16 / 1024,
             rom_header.chr_rom_size / 8 / 1024,
             rom_header.mapper_number);
    let mapper_id = rom_header.mapper_number;
    let (prg_rom, chr_rom) = parse_prg_and_chr_rom_data(&rom_data);

    // 使用create_mapper函数创建对应的映射器实例
    let mapper = create_mapper(mapper_id, prg_rom, chr_rom);

    // 使用创建好的映射器实例初始化Memory结构体
    let memory = Memory::new(mapper);

    // ... 略过其他代码 ...
}

// ... 略过其他代码 ...
fn parse_rom_header(rom_data: &[u8]) -> RomHeader {
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


fn parse_prg_and_chr_rom_data(rom_data: &[u8]) -> (Vec<u8>, Vec<u8>) {
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
