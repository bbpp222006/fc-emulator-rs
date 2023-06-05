pub struct Oam{
    oam: [u8; 0x100],
    pub oam_addr: u16,
}

// 每个精灵共4字节的属性, 共计64个精灵
// 字节0: Y坐标-1
// 字节1: 精灵图案编号
// 字节2: 属性 
// vhpx_xxpp
// |||| ||**- 调色板高两位
// ||*- ----- 优先级：0-背景前（显示），1-背景后（隐藏）
// |*-- ----- 水平翻转：0-正常，1-翻转
// *--- ----- 垂直翻转：0-正常，1-翻转
// 字节3: X坐标

impl Default for Oam {
    fn default() -> Self {
        Oam {
            oam: [0; 0x100],
            oam_addr: 0,
        }
    }
} 

impl Oam {
    pub fn new() -> Self {
        self::Oam::default()
    }

    pub fn read(&self, addr: u16) -> u8 {
        let ram_addr = addr & 0xFF;
        self.oam[ram_addr as usize]
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        let ram_addr = addr & 0xFF;
        self.oam[ram_addr as usize]= data;
    }

    pub fn reset(&mut self) {
        self.oam = [0; 0x100];
        self.oam_addr = 0;
    }
}