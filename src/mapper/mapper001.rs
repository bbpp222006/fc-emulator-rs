// mapper.rs

// 引入标准库中的类型和特质
use super::Mapper;



// 定义 NromMapper 结构体
#[derive(Debug)]
pub struct Mapper001 {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mirror_mode: u8,
}

impl Mapper001 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirror_mode: u8) -> Self {
        Mapper001 {
            prg_rom,
            chr_rom,
            mirror_mode,
        }
    }
}

impl Mapper for Mapper001 {
    fn read_prg_rom(&self, addr: u16) -> u8 {
        let addr = addr as usize % self.prg_rom.len();
        self.prg_rom[addr]
    }

    fn read_prg_ram(&self, addr: u16) -> u8 {
        let addr = addr as usize % self.prg_rom.len();
        self.prg_rom[addr]
    }

    fn write_prg_rom(&mut self, _addr: u16, _data: u8) {
    }
    fn write_prg_ram(&mut self, _addr: u16, _data: u8) {
    }

    fn read_chr_rom(&self, addr: u16) -> u8 {
        let addr = addr as usize % self.chr_rom.len();
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
    }
}
