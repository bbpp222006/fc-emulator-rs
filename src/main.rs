// src/main.rs

use std::fs::File;
use std::io::Read;

// 导入所需的模块
mod memory;
mod utils;
mod cpu;
use memory::{create_mapper, Memory};
use utils::{parse_rom_header, parse_prg_and_chr_rom_data, parse_interrupt_vectors};




fn main() {
    // ... 略过其他代码 ...

    // 读取ROM文件
    let mut rom_file = File::open("rom/nestest.nes").expect("Cannot open ROM file");
    let mut rom_data = Vec::new();
    rom_file.read_to_end(&mut rom_data).expect("Cannot read ROM file");

    // 解析ROM文件头部信息，获取所需数据，如映射器ID、PRG ROM数据和CHR ROM数据
    let rom_header = parse_rom_header(&rom_data);

    // 打印文件头信息
    println!("prg_rom_size: {} x 16kb, chr_rom_size: {} x 8kb, mapper_id: {}, mirroring_type: {}, battery_backed_ram: {}, trainer: {}, nes2_0: {}",
             rom_header.prg_rom_size / 16 / 1024,
             rom_header.chr_rom_size / 8 / 1024,
             rom_header.mapper_number,
             rom_header.mirroring_type,
             rom_header.battery_backed_ram,
             rom_header.trainer,
             rom_header.nes2_0);


    let mapper_id = rom_header.mapper_number;
    let (prg_rom, chr_rom) = parse_prg_and_chr_rom_data(&rom_data);
    let interrupt_vectors = parse_interrupt_vectors(&prg_rom);
    // 打印文件头信息
    println!("NMI: {:04X}, RESET: {:04X}, IRQ: {:04X}",
    interrupt_vectors.nmi_vector, interrupt_vectors.reset_vector, interrupt_vectors.irq_vector);
    
    // 使用create_mapper函数创建对应的映射器实例
    let mapper = create_mapper(mapper_id, prg_rom, chr_rom, rom_header.mirroring_type);

    // 使用创建好的映射器实例初始化Memory结构体
    let memory = Memory::new(mapper);
    
    // 读取各个中断向量的指向的地址，并反编译查看指令
    let mut disassembler = cpu::disassembler::Disassembler::new(memory, interrupt_vectors.reset_vector, interrupt_vectors.reset_vector + 0x100);
    let (disassembly, next_address) = disassembler.disassemble_instruction(interrupt_vectors.reset_vector);
    println!("{:04X}: {}", interrupt_vectors.reset_vector, disassembly);
    

    // ... 略过其他代码 ...
}


