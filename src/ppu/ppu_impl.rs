struct Registers {
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
    ppuctrl: u8,

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
    ppumask: u8,

    // PPUSTATUS 寄存器，用于存储 PPU 的一些状态。
    // 7 6 5 4 3 2 1 0
    // V S O . . . . .
    // | | | + + + + +-- ppu 的open bus，未使用
    // | | +------------ 精灵溢出标志，当精灵数量超过8个时，该标志会被置位
    // | +-------------- 精灵0的碰撞标志，当精灵0与背景发生碰撞时，该标志会被置位，在预渲染期间被清除，用于光栅计时
    // +---------------- vblank标志，当ppu处于vblank时，该标志会被置位，结束或者读取该寄存器会清除该标志
    ppustatus: u8,

    // OAMADDR 寄存器，用于存储 OAM 读写的地址。
    // 7 6 5 4 3 2 1 0
    // a a a a a a a a
    // + + + + + + + +-- OAM 地址
    oamaddr: u8,

    // OAMDATA 寄存器，用于存储 OAM 读写的数据。
    // 7 6 5 4 3 2 1 0
    // d d d d d d d d
    // + + + + + + + +-- OAM 数据
    oamdata: u8,

    // PPUSCROLL 寄存器，用于存储 PPU 的滚动位置，写两次，第一次写入垂直滚动位置，第二次写入水平滚动位置。
    ppuscroll: u8,

    // PPUADDR 寄存器，用于存储 PPU 的写地址，写两次，第一次写入高字节6位，第二次写入低字节8位。
    ppuaddr: u8,

    // PPUDATA 寄存器，用于存储 PPU 的读写数据。读写后指针增长与ppuctrl的第2位有关，+1或者32
    ppudata: u8, 
    
    // OAMDMA 寄存器，只写，DMA访问精灵ram
    oamdma: u8,
}

pub struct SpriteEvaluationState {
    // 用于读取 OAM（Object Attribute Memory）的地址
    oam_address: u8,

    // 当前正在评估的精灵的索引
    sprite_index: u8,

    // 在精灵评估阶段的各个步骤之间用于临时存储数据的缓冲区
    temp_sprite_data: [u8; 4],

    // 当前精灵数据的读取步骤（0-4，对应于精灵数据的 Y 坐标、tile 索引、属性和 X 坐标）
    read_phase: u8,

    // 指示当前是否在读取“垃圾”精灵数据（即超过 8 个的精灵数据）
    reading_garbage: bool,

    // 跟踪在当前行找到的精灵数量，如果超过8个，就设置 sprite_overflow 标志
    sprite_count: u8,
}


pub struct Ppu {
    // VRAM (Video RAM) 是 PPU 用于存储图像数据的内存。在 NES 中，这包括了背景图案表，精灵图案表，名称表和属性表等内容。
    vram: [u8; 0x8000],

    // OAM (Object Attribute Memory) 用于存储精灵的属性。在 NES 中，它可以存储 64 个精灵的信息。
    oam: [u8; 0x100],

    // 现在获取的图块的数据。
    current_tile_data: u16,
    
    // 图块数据的移位寄存器。
    tile_shift_registers: [u16; 2],

    // 记录 PPU 当前经过的周期数。每个 PPU 周期，PPU 可能会进行一些工作，例如更新扫描线，读写内存等。
    cycles: usize,

    // PPU 寄存器，用于存储 PPU 的状态，例如当前扫描线，滚动位置等。
    registers: Registers,

    // 当前的扫描线位置，范围从 0 到 261，表示一帧中所有的扫描线（包括可见扫描线和垂直空白等）。
    scanline: u16,

    // 当前在扫描线中的位置，范围从 0 到 340，表示一个扫描线中所有的像素点（包括可见像素和水平空白等）。
    dot: u16,
    
    // 当前扫描线是否在渲染。虽然 PPU 在整个帧周期内都在运行，但只有在一部分时间内它才在屏幕上渲染像素（即所谓的 "可见扫描线" 时期）。
    rendering_enabled: bool,

    // 当前扫描线是否在垂直空白期。垂直空白期是每一帧渲染结束后的一个时间段，这个时期内 PPU 不会渲染任何东西，但可以进行 VRAM 的读写。
    in_vblank: bool,

    // 当前扫描线是否在水平空白期。水平空白期是每一条扫描线渲染结束后的一个时间段，这个时期内 PPU 不会渲染任何东西，但可以进行 VRAM 的读写。
    in_hblank: bool,

    // 背景和精灵的渲染位置。这两个值在渲染期间不断更新，以决定从哪里获取图案数据。
    bg_pattern_table_address: u16,
    spr_pattern_table_address: u16,

    // 用于存储即将要渲染的背景和精灵像素的缓冲区。
    bg_pixel_buffer: [u8; 256],
    spr_pixel_buffer: [u8; 256],

    // PPUCTRL，PPUMASK，PPUSTATUS，OAMADDR，OAMDATA，PPUSCROLL，PPUADDR，PPUDATA 这些寄存器的值
    control_register: u8,
    mask_register: u8,
    status_register: u8,
    oam_address: u8,
    scroll_register: u16,
    ppu_address: u16,
    ppu_data: u8,

    // PPU 内部的两个缓冲寄存器
    address_latch: bool,
    high_byte_buffer: u8,

    // PPU 的内部精灵评估状态
    sprite_evaluation_state: SpriteEvaluationState,
    sprite_shift_registers: [u8; 8],

    // PPU 的渲染计数器
    fine_x_scroll: u8,
    y_scroll: u8,
    x_scroll: u8,

    // PPU 的背景渲染状态
    tile_data: u64,
    tile_latch: u8,

    // PPU 的精灵渲染状态
    sprite_count: u8,
    sprite_patterns: [u8; 8],
    sprite_positions: [u8; 8],
    sprite_priorities: [u8; 8],
    sprite_indexes: [u8; 8],

    // PPU 的命中和溢出状态
    sprite_zero_hit: bool,
    sprite_overflow: bool,

    // 颜色调色板
    palette: [u8; 32],  
}


impl Ppu {
    pub fn new() -> Self {
        Self {
            vram: [0; 0x8000],
            oam: [0; 0x100],
            cycles: 0,
            registers: todo!(),
            scanline: todo!(),
            dot: todo!(),
            rendering_enabled: todo!(),
            in_vblank: todo!(),
            in_hblank: todo!(),
            bg_pattern_table_address: todo!(),
            spr_pattern_table_address: todo!(),
            bg_pixel_buffer: todo!(),
            spr_pixel_buffer: todo!(),
            control_register: todo!(),
            mask_register: todo!(),
            status_register: todo!(),
            oam_address: todo!(),
            scroll_register: todo!(),
            ppu_address: todo!(),
            ppu_data: todo!(),
            address_latch: todo!(),
            high_byte_buffer: todo!(),
            sprite_evaluation_state: todo!(),
            sprite_shift_registers: todo!(),
            fine_x_scroll: todo!(),
            y_scroll: todo!(),
            x_scroll: todo!(),
            tile_data: todo!(),
            tile_latch: todo!(),
            sprite_count: todo!(),
            sprite_patterns: todo!(),
            sprite_positions: todo!(),
            sprite_priorities: todo!(),
            sprite_indexes: todo!(),
            sprite_zero_hit: todo!(),
            sprite_overflow: todo!(),
            palette: todo!(),
            current_tile_data: 0,
            tile_shift_registers: [0; 2],
        }
    }

    pub fn reset(&mut self) {
        self.vram = [0; 0x8000];
        self.oam = [0; 0x100];
        self.cycles = 0;
        // ... reset other fields
    }

    fn start_of_scanline(&mut self) {
        todo!()
    }

    fn load_tile_data_to_shift_registers(&mut self, tile_data: [u8; 16]) {
        // 在每个图块中，每8个字节的数据代表一个8x8像素的图块的低位平面或高位平面
        // 低位平面的每一位和对应的高位平面的位结合起来，形成一个2位的颜色索引
        // 我们需要将这16字节的数据分别加载到两个移位寄存器中
        for i in 0..8 {
            let low_plane = tile_data[i];
            let high_plane = tile_data[i+8];
            for j in 0..8 {
                let color_index = ((high_plane >> (7-j) & 1) << 1) | (low_plane >> (7-j) & 1);
                self.tile_shift_registers[0] = (self.tile_shift_registers[0] << 2) | color_index as u16;
            }
        }
    
        // 在完成图块数据的加载后，我们需要将移位寄存器中的数据复制到“当前图块”寄存器中
        // “当前图块”寄存器将被用来在绘制期间提供像素数据
        self.current_tile_data = self.tile_shift_registers[0];
    }
    

 
    fn fetch_tile_data(&mut self) {
        // 在PPU的内存中，图块数据被存储在两个图案表（pattern tables）中。
        // 每个图案表都有0x1000字节，分别存储了256个8x8的图块。我们需要确定我们要从哪个图案表中获取数据。
        let pattern_table_base = match self.registers.ppuctrl>>4 & 1 {
            0 => 0x0000,
            1 => 0x1000,
            _ => unreachable!(),
        };
    
        // 接下来我们需要确定要加载的图块的索引。在NES中，图块的索引被存储在两个名称表（nametables）中。
        // 每个名称表都有0x3C0字节，分别存储了30x32=960个图块的索引。
        // 我们需要根据PPU的当前扫描线（scanline）和周期（dot）来确定我们需要加载哪个图块。
        let nametable_base = match self.registers.ppuctrl & 0x3 {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2C00,
            _ => unreachable!(),
        };
        let tile_x = (self.dot - 1) / 8;
        let tile_y = self.scanline / 8;
        let tile_index_address = nametable_base + tile_y * 32 + tile_x;
        let tile_index = self.vram[tile_index_address as usize];
    
        // 现在我们有了图块的索引，我们就可以从图案表中获取图块的数据了。
        // 每个图块都有16字节，包括8字节的低位平面和8字节的高位平面。
        // 我们需要分别读取这两个平面的数据，然后将它们合并起来形成最终的图块数据。
        let tile_data_address = pattern_table_base + tile_index as u16 * 16;
        let mut tile_data = [0; 16];
        for i in 0..16 {
            tile_data[i] = self.vram[(tile_data_address + i as u16) as usize];
        }
    
        // 最后我们将图块数据加载到shift registers中
        self.load_tile_data_to_shift_registers(tile_data);
    }
    
    
    

    pub fn step(&mut self) {
        match self.dot {
            0 => {
                // 在每个扫描线的开始，我们可能需要做一些准备工作
                self.start_of_scanline();
            }
            1..=256 | 321..=336 => {
                // 在可见扫描线和两个“空闲”周期中，PPU 需要获取背景和精灵的图块数据
                self.fetch_tile_data();
            }
            257..=320 => {
                // 在这个阶段，PPU 需要获取下一行将要显示的精灵的数据
                todo!();
            }
            337..=340 => {
                // 在每个扫描线的最后几个周期中，PPU 将进行一些清理工作
                todo!();
                
            }
            _ => unreachable!(),
        }
    
        // 更新 PPU 的当前周期和扫描线
        self.cycles+=1;
        self.dot += 1;
        if self.dot > 340 {
            self.dot = 0;
            self.scanline += 1;
            if self.scanline > 261 {
                self.scanline = 0;
            }
        }
    }
    

    pub fn read(&self, addr: u16) -> u8 {
        // Implement the reading from VRAM/OAM based on the given address
        // You might want to consider mirroring and other specific behaviors of NES PPU
        todo!()
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        // Implement the writing to VRAM/OAM based on the given address and data
        // You might want to consider mirroring and other specific behaviors of NES PPU
    }

    // ... other Ppu methods
}
