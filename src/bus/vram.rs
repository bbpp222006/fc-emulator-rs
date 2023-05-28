pub struct Vram{
    ram : [u8; 0x1000],
}

impl Default for Vram {
    fn default() -> Self {
        Vram {
            ram: [0; 0x1000],
        }
    }
}

impl Vram {
    pub fn new() -> Self {
        self::Vram::default()
    }

    pub fn read(&self, addr: u16) -> u8 {
        let ram_addr = addr & 0x0fff;
        self.ram[ram_addr as usize]
    }   

    pub fn write(&mut self, addr: u16, data: u8) {
        let ram_addr = addr & 0x0fff;
        self.ram[ram_addr as usize]= data;
    }

    pub fn reset(&mut self) {
        self.ram = [0; 0x1000];
    }
}