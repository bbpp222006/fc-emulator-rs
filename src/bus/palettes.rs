pub struct Palette {
    pub colors: [u32; 64],
}

impl Palette {
    pub fn new() -> Palette {
        Palette {
            colors: [
                0x545454, 0x001E74, 0x081090, 0x300088, 0x440064, 0x5C0030, 0x540400, 0x3C1800,
                0x202A00, 0x083A00, 0x004000, 0x003C00, 0x00323C, 0x000000, 0x000000, 0x000000,
                0x989698, 0x084CC4, 0x3032EC, 0x5C1EE4, 0x8814B0, 0xA01464, 0x982220, 0x783C00,
                0x545A00, 0x287200, 0x087C00, 0x007628, 0x006678, 0x000000, 0x000000, 0x000000,
                0xECEEEC, 0x4C9AEC, 0x787CEC, 0xB062EC, 0xE454EC, 0xEC58B4, 0xEC6A64, 0xD48820,
                0xA0AA00, 0x74C400, 0x4CD020, 0x38CC6C, 0x38B4CC, 0x3C3C3C, 0x000000, 0x000000,
                0xECEEEC, 0xA8CCEC, 0xBCBCEC, 0xD4B2EC, 0xECAEEC, 0xECAED4, 0xECB4B0, 0xE4C490,
                0xCCD278, 0xB4DE78, 0xA8E290, 0x98E2B4, 0xA0D6E4, 0xA0A2A0, 0x000000, 0x000000,
            ],
        }
    }
}

impl Default for Palette {
    fn default() -> Self {
        Palette::new()
    }
}

impl Palette {
    pub fn read(&self, addr: u16) -> u8 {
        let addr = addr & 0x3f;
        let color = self.colors[addr as usize];
        let r = (color >> 16) & 0xff;
        let g = (color >> 8) & 0xff;
        let b = color & 0xff;
        let r = r >> 5;
        let g = g >> 5;
        let b = b >> 5;
        ((b << 6) | (g << 3) | r) as u8
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        let addr = addr & 0x3f;
        let color = self.colors[addr as usize];
        let r = (color >> 16) & 0xff;
        let g = (color >> 8) & 0xff;
        let b = color & 0xff;
        let r = r >> 5;
        let g = g >> 5;
        let b = b >> 5;
        let r = data & 0b0000_0111;
        let g = (data & 0b0011_1000) >> 3;
        let b = (data & 0b1110_0000) >> 6;
        let r = r << 5;
        let g = g << 5;
        let b = b << 5;
        self.colors[addr as usize] = ((r << 16) | (g << 8) | b) as u32;
    }
}