pub struct Vram{
    ram : [u8; 0x1000],
    palettes_ram : [u8; 0x0020],
    pub vram_addr: u16,
}

impl Default for Vram {
    fn default() -> Self {
        Vram {
            ram: [0; 0x1000],
            palettes_ram: [0; 0x0020],
            vram_addr: 0,
        }
    }
}

impl Vram {
    pub fn new() -> Self {
        self::Vram::default()
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x2000..=0x3eff => {
                // 0x2000 ~ 0x3eff 是 nametable
                let ram_addr = addr & 0x0fff;
                self.ram[ram_addr as usize]
            },
            0x3f00..=0x3fff => {
                // 0x3f00 ~ 0x3f1f 是调色板
                let ram_addr = addr & 0x001f;
                self.palettes_ram[ram_addr as usize]
            },
            _ => panic!("invalid vram addr: {:04X}", addr),
        }
    }   

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x2000..=0x3eff => {
                // 0x2000 ~ 0x3eff 是 nametable
                let ram_addr = addr & 0x0fff;
                self.ram[ram_addr as usize] = data;
            },
            0x3f00..=0x3fff => {
                // 0x3f00 ~ 0x3f1f 是调色板
                let ram_addr = addr & 0x001f;
                self.palettes_ram[ram_addr as usize] = data;
            },
            _ => panic!("invalid vram addr: {:04X}", addr),
        }
    }

    pub fn reset(&mut self) {
        self.ram = [0; 0x1000];
        self.palettes_ram = [0; 0x0020];
        self.vram_addr = 0;
    }
}