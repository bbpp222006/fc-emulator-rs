pub struct CpuRam{
    ram: [u8; 0x800],
}

impl Default for CpuRam {
    fn default() -> Self {
        CpuRam {
            ram: [0; 0x800],
        }
    }
} 

impl CpuRam {
    pub fn new() -> Self {
        self::CpuRam::default()
    }

    pub fn read(&self, addr: u16) -> u8 {
        let ram_addr = addr & 0x07FF;
        self.ram[ram_addr as usize]
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        let ram_addr = addr & 0x07FF;
        self.ram[ram_addr as usize]= data;
    }
}