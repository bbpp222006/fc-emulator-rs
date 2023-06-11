// mapper.rs

// 引入标准库中的类型和特质
use super::Mapper;



// 定义 NromMapper 结构体
#[derive(Debug)]
pub struct Mapper003 {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    prg_rom_init: Vec<u8>,
    chr_rom_init: Vec<u8>,
    mirror_mode: u8,
    chr_rom_bank : u8,
}

impl Mapper003 {
    // NromMapper 的构造函数
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirror_mode: u8) -> Self {
        Mapper003 {
            prg_rom_init: prg_rom.clone(),
            chr_rom_init: chr_rom.clone(),
            prg_rom,
            chr_rom,
            mirror_mode,
            chr_rom_bank : 0,
        }
    }
}

// 为 Mapper003 实现 Mapper trait
impl Mapper for Mapper003 {
    fn read_prg_rom(&self, addr: u16) -> u8 {
        let addr = addr as usize % self.prg_rom.len();
        self.prg_rom[addr]
    }
    fn read_prg_ram(&self, addr: u16) -> u8 {
        todo!()
    }

    fn write_prg_rom(&mut self, addr: u16, data: u8) {
        self.chr_rom_bank = data & 0x3;
    }
    fn write_prg_ram(&mut self, _addr: u16, _data: u8) {
    }

    fn read_chr_rom(&self, addr: u16) -> u8 {
        let addr = self.chr_rom_bank as usize * 0x2000 + addr as usize % 0x2000;
        let a = self.chr_rom[addr];
        a
    }


    fn write_chr_rom(&mut self, addr: u16, data: u8) {
        let addr = addr as usize % self.chr_rom.len();
        self.chr_rom[addr as usize] = data;
    }
    

    fn ppu_mirror_mode(&self) -> u8 {
        self.mirror_mode
    }
    
    fn reset(&mut self) {
        self.chr_rom_bank = 0;
        self.prg_rom = self.prg_rom_init.clone();
        self.chr_rom = self.chr_rom_init.clone();
    }
}
