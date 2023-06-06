pub struct Nametable{
    ram : [u8; 0x1000],
}

impl Default for Nametable {
    fn default() -> Self {
        Nametable {
            ram: [0; 0x1000],
        }
    }
}

impl Nametable {
    pub fn new() -> Self {
        self::Nametable::default()
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x2000..=0x3eff => {
                // 0x2000 ~ 0x3eff 是 nametable
                let ram_addr = addr & 0x0fff;
                self.ram[ram_addr as usize]
            },
            _ => panic!("invalid nametable addr: {:04X}", addr),
        }
    }   

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x2000..=0x3eff => {
                // 0x2000 ~ 0x3eff 是 nametable
                let ram_addr = addr & 0x0fff;
                self.ram[ram_addr as usize] = data;
            },
            _ => panic!("invalid nametable addr: ${:04X}", addr),
        }
    }

    pub fn reset(&mut self) {
        self.ram = [0; 0x1000];
    }
}