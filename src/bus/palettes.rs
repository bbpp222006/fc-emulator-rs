pub struct Palettes{
    palettes_ram : [u8; 0x0020],
}

impl Default for Palettes {
    fn default() -> Self {
        Palettes {
            palettes_ram: [0; 0x0020],
        }
    }
}

impl Palettes {
    pub fn new() -> Self {
        self::Palettes::default()
    }

    pub fn read(&self, addr: u16) -> u8 {
        // 0x3f00 ~ 0x3f1f 是调色板
        let ram_addr = addr & 0x001f;
        self.palettes_ram[ram_addr as usize]
    }   

    pub fn write(&mut self, addr: u16, data: u8) {
        // 0x3f00 ~ 0x3f1f 是调色板
        let ram_addr = addr & 0x001f;
        self.palettes_ram[ram_addr as usize] = data;
    }

    pub fn reset(&mut self) {
        self.palettes_ram = [0; 0x0020];
    }
}