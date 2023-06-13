pub struct Registers {
    // PPUCTRL 寄存器，用于控制 PPU 的一些行为。
    // 7 6 5 4 3 2 1 0
    // V P H B S I N N
    // | | | | | | + +-- 名称表基地址(0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
    // | | | | | +------ ppu读写vram的步长(0 = $1 横向，1 = $20 =32 纵向)
    // | | | | +-------- 精灵的图案表基地址(0 = $0000; 1 = $1000; 在8x16 模式时忽略)
    // | | | +---------- 背景图案表基地址(0 = $0000; 1 = $1000)
    // | | +------------ 精灵大小(0 = 8x8 像素; 1 = 8x16 像素)
    // | +-------------- PPU 主从模式(0 = 读写; 1 = 仅读)，fc并没有使用
    // +---------------- NMI 使能(0 = 禁用; 1 = 启用)，vblank时触发nmi
    pub ppuctrl: u8,

    // PPUMASK 寄存器，用于控制 PPU 的一些行为。
    // 7 6 5 4 3 2 1 0
    // B G R s b M m G
    // | | | | | | | +-- 显示模式(0 = 彩色; 1 = 灰阶)
    // | | | | | | +---- 是否显示最左侧的8像素背景(0 = 隐藏; 1 = 显示)
    // | | | | | +------ 是否显示最左侧的8像素精灵(0 = 隐藏; 1 = 显示)
    // | | | | +-------- 是否显示背景(0 = 隐藏; 1 = 显示)
    // | | | +---------- 是否显示精灵(0 = 隐藏; 1 = 显示)
    // | | +------------ 增强红颜色(0 = 禁用; 1 = 启用)，PAL为绿色
    // | +-------------- 增强绿颜色(0 = 禁用; 1 = 启用)，PAL为红色
    // +---------------- 增强蓝颜色(0 = 禁用; 1 = 启用)
    pub ppumask: u8,

    // PPUSTATUS 寄存器，用于存储 PPU 的一些状态。
    // 7 6 5 4 3 2 1 0
    // V S O . . . . .
    // | | | + + + + +-- ppu 的open bus，未使用
    // | | +------------ 精灵溢出标志，当精灵数量超过8个时，该标志会被置位
    // | +-------------- 精灵0的碰撞标志，当精灵0与背景发生碰撞时，该标志会被置位，在预渲染期间被清除，用于光栅计时
    // +---------------- vblank标志，当ppu处于vblank时，该标志会被置位，结束或者读取该寄存器会清除该标志
    pub ppustatus: u8,


    // OAMADDR 寄存器，用于存储 OAM 读写的地址。
    // 7 6 5 4 3 2 1 0
    // a a a a a a a a
    // + + + + + + + +-- OAM 地址
    pub oamaddr: u8,

    // OAMDATA 寄存器，用于存储 OAM 读写的数据。
    // 7 6 5 4 3 2 1 0
    // d d d d d d d d
    // + + + + + + + +-- OAM 数据
    pub oamdata: u8,

    // PPUSCROLL 寄存器，用于存储 PPU 的滚动位置，写两次，第一次写入垂直滚动位置，第二次写入水平滚动位置。
    pub ppuscroll: u8,

    // PPUADDR 寄存器，用于存储 PPU 的写地址，写两次，第一次写入高字节6位，第二次写入低字节8位。
    pub ppuaddr: u8,

    // PPUDATA 寄存器，用于存储 PPU 的读写数据。读写后指针增长与ppuctrl的第2位有关，+1或者32
    pub ppudata: u8, 
    
    // OAMDMA 寄存器，只写，DMA访问精灵ram
    pub oamdma: u8,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            ppuctrl: 0,
            ppumask: 0,
            ppustatus: 0,
            oamaddr: 0,
            oamdata: 0,
            ppuscroll: 0,
            ppuaddr: 0,
            ppudata: 0,
            oamdma: 0,
            
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        let reg_addr = 0x2000+(addr & 0x0007) as usize; 
        
        let out_data = match reg_addr {
            0x2000 => self.ppuctrl,
            0x2001 => self.ppumask,
            0x2002 => self.ppustatus,
            0x2003 => self.oamaddr,
            0x2004 => self.oamdata,
            0x2005 => self.ppuscroll,
            0x2006 => self.ppuaddr,
            0x2007 => self.ppudata,
            _ => panic!("invalid ppu register addr: {:04X}", addr),
        };
        
        out_data
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        let reg_addr = 0x2000+(addr & 0x0007) as usize; 
        
        match reg_addr {
            0x2000 => self.ppuctrl = data,
            0x2001 => self.ppumask = data,
            0x2002 => {}//self.ppustatus = data, cpu不能写入ppustatus
            0x2003 => self.oamaddr = data,
            0x2004 => self.oamdata = data,
            0x2005 => self.ppuscroll = data,
            0x2006 => self.ppuaddr = data,
            0x2007 => self.ppudata = data,
            _ => panic!("invalid ppu register addr: {:04X}", addr),
        }
    }

    pub fn reset(&mut self) {
        self.ppuctrl = 0;
        self.ppumask = 0;
        self.ppustatus = 0;
        // self.oamaddr = 0;
        self.oamdata = 0;
        self.ppuscroll = 0;
        // self.ppuaddr = 0;
        self.ppudata = 0;
        self.oamdma = 0;
    }
}

