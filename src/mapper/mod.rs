pub mod mapper000;

use mapper000::NromMapper;

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
    fn write_prg_rom(&mut self, addr: u16, data: u8);
    fn read_chr_rom(&self, addr: u16) -> u8;
    fn write_chr_rom(&mut self, addr: u16, data: u8);
    fn ppu_mirror_mode(&self) -> u8;
    fn reset(&mut self);
}


pub fn create_mapper(
    rom_data: &[u8],
) -> Box<dyn Mapper> {

    let rom_header = parse_rom_header(&rom_data);

    // 提取PRG-ROM和CHR-ROM数据
    let (prg_rom, chr_rom) = parse_prg_and_chr_rom_data(&rom_data);

    // 提取中断信息
    let interrupt_vectors = parse_interrupt_vectors(&prg_rom);

    match rom_header.mapper_number {
        0 => Box::new(NromMapper::new(prg_rom, chr_rom, rom_header.mirroring_type)),
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