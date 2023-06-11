pub mod mapper000;
mod mapper003;
mod mapper001;

use mapper000::NromMapper;
use mapper003::Mapper003;
use mapper001::Mapper001;

#[derive(Debug)]
pub struct RomHeader {
    pub prg_rom_size: usize,
    pub chr_rom_size: usize,
    pub mapper_number: u8,
    pub mirroring_type: u8,
    pub battery_backed_ram: bool,
    pub trainer: bool,
    pub nes2_0: bool,
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


// 定义一个通用的 Mapper trait
pub trait Mapper: Send {
    fn read_prg_rom(&self, addr: u16) -> u8;
    fn read_prg_ram(&self, addr: u16) -> u8;
    fn write_prg_rom(&mut self, addr: u16, data: u8);
    fn write_prg_ram(&mut self, addr: u16, data: u8);
    fn read_chr_rom(&self, addr: u16) -> u8;
    fn write_chr_rom(&mut self, addr: u16, data: u8);
    fn ppu_mirror_mode(&self) -> u8;
    fn reset(&mut self);
}


pub fn create_mapper(
    rom_data: &[u8],
) -> Box<dyn Mapper> {

    let rom_header = parse_rom_header(&rom_data);
    println!("prg_rom_size:{}kb,chr_rom_size:{}kb,mapper_number:{},mirroring_type:{},battery_backed_ram:{},trainer:{},nes2_0:{}",
             rom_header.prg_rom_size,rom_header.chr_rom_size,rom_header.mapper_number,rom_header.mirroring_type,rom_header.battery_backed_ram,rom_header.trainer,rom_header.nes2_0);

    // 提取PRG-ROM和CHR-ROM数据
    let (prg_rom, chr_rom) = parse_prg_and_chr_rom_data(&rom_data);

    // 提取中断信息
    let interrupt_vectors = parse_interrupt_vectors(&prg_rom);

    match rom_header.mapper_number {
        0 => Box::new(NromMapper::new(prg_rom, chr_rom, rom_header.mirroring_type)),
        1 => Box::new(Mapper001::new(prg_rom, chr_rom, rom_header.mirroring_type)),
        3 => Box::new(Mapper003::new(prg_rom, chr_rom, rom_header.mirroring_type)),
        // 在这里添加其他 Mapper 的实现
        _ => panic!("Unsupported mapper ID: {}", rom_header.mapper_number),
    }
}



// 解析中断向量
fn parse_interrupt_vectors(prg_rom: &Vec<u8>) -> InterruptVectors {
    let nmi_vector = u16::from_le_bytes([prg_rom[prg_rom.len() - 6], prg_rom[prg_rom.len() - 5]]);
    let reset_vector = u16::from_le_bytes([prg_rom[prg_rom.len() - 4], prg_rom[prg_rom.len() - 3]]);
    let irq_vector = u16::from_le_bytes([prg_rom[prg_rom.len() - 2], prg_rom[prg_rom.len() - 1]]);

    InterruptVectors {
        nmi_vector,
        reset_vector,
        irq_vector,
    }
}

fn parse_prg_and_chr_rom_data(rom_data: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let header_size = 16;
    let prg_rom_size = rom_data[4] as usize * 16384; // PRG-ROM size is at offset 4
    let chr_rom_size = rom_data[5] as usize * 8192; // CHR-ROM size is at offset 5

    // Copy PRG-ROM data
    let prg_rom_data = rom_data[header_size..header_size + prg_rom_size].to_vec();

    // Handle CHR-ROM or CHR-RAM
    let chr_rom_data = if chr_rom_size == 0 {
        vec![0; 8192] // CHR-RAM size could be different, adjust accordingly
    } else {
        rom_data[header_size + prg_rom_size..header_size + prg_rom_size + chr_rom_size].to_vec()
    };

    (prg_rom_data, chr_rom_data)
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