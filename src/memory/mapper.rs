// src/memory/mapper.rs

pub trait Mapper {
    // 读取指定地址的数据
    fn read(&self, address: u16) -> u8;

    // 向指定地址写入数据
    fn write(&mut self, address: u16, value: u8);
}

// 示例：NROM Mapper (Mapper ID 0)
pub struct NromMapper {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
}

impl NromMapper {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        NromMapper { prg_rom, chr_rom }
    }
}

impl Mapper for NromMapper {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => {
                let index = address as usize % self.chr_rom.len();
                self.chr_rom[index]
            }
            0x8000..=0xFFFF => {
                let index = address as usize % self.prg_rom.len();
                self.prg_rom[index]
            }
            _ => 0,
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                let index = address as usize % self.chr_rom.len();
                self.chr_rom[index] = value;
            }
            0x8000..=0xFFFF => {
                let index = address as usize % self.prg_rom.len();
                self.prg_rom[index] = value;
            }
            _ => (),
        }
    }
}

// // 示例：MMC1 Mapper (Mapper ID 1)
// pub struct Mmc1Mapper {
//     pub prg_rom: Vec<u8>,
//     pub chr_rom: Vec<u8>,
//     // 添加其他MMC1相关的状态和数据结构
//     // ...
// }

// impl Mmc1Mapper {
//     pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
//         Mmc1Mapper {
//             prg_rom,
//             chr_rom,
//             // 初始化其他MMC1相关的状态和数据结构
//             // ...
//         }
//     }
// }

// impl Mapper for Mmc1Mapper {
//     fn read(&self, address: u16) -> u8 {
//         match address {
//             0x0000..=0x1FFF => {
//                 // 根据MMC1的特定映射逻辑读取CHR ROM数据
//                 // ...
//             }
//             0x8000..=0xFFFF => {
//                 // 根据MMC1的特定映射逻辑读取PRG ROM数据
//                 // ...


//             }
//             _ => 0,
//         }
//     }

//     fn write(&mut self, address: u16, value: u8) {
//         match address {
//             0x0000..=0x1FFF => {
//                 // 根据MMC1的特定映射逻辑写入CHR ROM数据
//                 // ...
//             }
//             0x8000..=0xFFFF => {
//                 // 根据MMC1的特定映射逻辑写入PRG ROM数据
//                 // ...
//             }
//             _ => (),
//         }
//     }
// }
