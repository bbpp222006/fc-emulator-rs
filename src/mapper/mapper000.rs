// mapper.rs

// 引入标准库中的类型和特质
use super::Mapper;



// 定义 NromMapper 结构体
#[derive(Debug)]
pub struct NromMapper {
    prg_rom: Vec<u8>,
    prg_ram: Vec<u8>,
    chr_rom: Vec<u8>,
    mirror_mode: u8,
}

impl NromMapper {
    // NromMapper 的构造函数
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirror_mode: u8) -> Self {
        NromMapper {
            prg_rom,
            prg_ram: vec![0; 0x2000],
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
    fn write_prg_ram(&mut self, addr: u16, data: u8) {
        let addr = addr as usize % self.prg_ram.len();
        self.prg_ram[addr] = data;
    }

    fn read_chr_rom(&self, addr: u16) -> u8 {
        let addr = addr as usize % self.chr_rom.len();
        let a = self.chr_rom[addr];
        // println!("{:}",a);
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
        self.prg_ram = vec![0; 0x2000];
    }
}
