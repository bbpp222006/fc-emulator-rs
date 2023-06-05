use crossbeam::channel::{Receiver, Sender};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::{
    bus::{Bus, RWMessage, RWResult, RWType},
    utils::{Frame, GlobalSignal},
};

pub struct Registers {
    current_vram_address: u16,   // 15 bits v
    temporary_vram_address: u16, // 15 bits t
    fine_x_scroll: u8,           // 3 bits x
    write_toggle: bool,          // 1 bit w
}

enum PpuRegister {
    PpuCtrl,
    PpuMask,
    PpuStatus,
    OamAddr,
    OamData,
    PpuScroll,
    PpuAddr,
    PpuData,
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

pub struct PpuChannels {
    ppu_frame_out: Sender<Frame>,
}

pub struct Ppu {
    bus: Arc<Mutex<Bus>>,
    // OAM (Object Attribute Memory) 用于存储精灵的属性。在 NES 中，它可以存储 64 个精灵的信息。

    // 现在获取的图块的数据。
    current_tile_data: u16,

    // 图块数据的移位寄存器。
    tile_shift_registers: [u16; 2],

    // 记录 PPU 当前经过的周期数。每个 PPU 周期，PPU 可能会进行一些工作，例如更新扫描线，读写内存等。
    cycles: usize,

    // PPU 寄存器，用于存储 PPU 的状态，例如当前扫描线，滚动位置等。
    // registers: Registers,

    // 当前的扫描线位置，范围从 0 到 261，表示一帧中所有的扫描线（包括可见扫描线和垂直空白等）。
    scanline: u16,

    // 当前在扫描线中的位置，范围从 0 到 340，表示一个扫描线中所有的像素点（包括可见像素和水平空白等）。
    dot: u16,

    // // 当前扫描线是否在渲染。虽然 PPU 在整个帧周期内都在运行，但只有在一部分时间内它才在屏幕上渲染像素（即所谓的 "可见扫描线" 时期）。
    // rendering_enabled: bool,

    // PPUSTATUS 寄存器，用于存储 PPU 的一些状态。
    // 7 6 5 4 3 2 1 0
    // V S O . . . . .
    // | | | + + + + +-- ppu 的open bus，未使用
    // | | +------------ 精灵溢出标志，当精灵数量超过8个时，该标志会被置位
    // | +-------------- 精灵0的碰撞标志，当精灵0与背景发生碰撞时，该标志会被置位，在预渲染期间被清除，用于光栅计时
    // +---------------- vblank标志，当ppu处于vblank时，该标志会被置位，结束或者读取该寄存器会清除该标志
    ppustatus: u8,

    nmi_status: bool, // nmi 状态

    frame_color_index_cache: [u8; 256 * 240],
    channels: PpuChannels,
    // // 当前扫描线是否在水平空白期。水平空白期是每一条扫描线渲染结束后的一个时间段，这个时期内 PPU 不会渲染任何东西，但可以进行 VRAM 的读写。
    // in_hblank: bool,

    // // 背景和精灵的渲染位置。这两个值在渲染期间不断更新，以决定从哪里获取图案数据。
    // bg_pattern_table_address: u16,
    // spr_pattern_table_address: u16,

    // // 用于存储即将要渲染的背景和精灵像素的缓冲区。
    // bg_pixel_buffer: [u8; 256],
    // spr_pixel_buffer: [u8; 256],

    // // PPUCTRL，PPUMASK，PPUSTATUS，OAMADDR，OAMDATA，PPUSCROLL，PPUADDR，PPUDATA 这些寄存器的值
    // control_register: u8,
    // mask_register: u8,
    // status_register: u8,
    // oam_address: u8,
    // scroll_register: u16,
    // ppu_address: u16,
    // ppu_data: u8,

    // // PPU 内部的两个缓冲寄存器
    // address_latch: bool,
    // high_byte_buffer: u8,

    // // PPU 的内部精灵评估状态
    // sprite_evaluation_state: SpriteEvaluationState,
    // sprite_shift_registers: [u8; 8],

    // // PPU 的渲染计数器
    // fine_x_scroll: u8,
    // y_scroll: u8,
    // x_scroll: u8,

    // // PPU 的背景渲染状态
    // tile_data: u64,
    // tile_latch: u8,

    // // PPU 的精灵渲染状态
    // sprite_count: u8,
    // sprite_patterns: [u8; 8],
    // sprite_positions: [u8; 8],
    // sprite_priorities: [u8; 8],
    // sprite_indexes: [u8; 8],

    // // PPU 的命中和溢出状态
    // sprite_zero_hit: bool,
    // sprite_overflow: bool,
    // channels: PpuChannels,
}

// pub fn start_ppu_thread(
//     ppu2bus_in: Sender<RWMessage>,
//     bus2ppu_out: Receiver<RWResult>,
//     ppu_frame_out: Sender<Frame>,
//     global_signal_out: Receiver<GlobalSignal>,
//     pip_log_in: Sender<String>,
// ) {
//     let mut ppu = Ppu::new(ppu2bus_in, bus2ppu_out, ppu_frame_out);
//     thread::spawn(move || loop {
//         let global_signal_out = global_signal_out.recv().unwrap();
//         match global_signal_out {
//             GlobalSignal::Reset => {
//                 ppu.reset();
//             }
//             GlobalSignal::Clock => {
//                 for _ in 0..3 {
//                     ppu.step();
//                 }
//             }
//             GlobalSignal::GetLog => {
//                 let log = ppu.get_current_log();
//                 // pip_log_in.send(log).unwrap();
//             }
//             GlobalSignal::Step => {
//                 ppu.step();
//             }
//         }
//     });
// }

impl Ppu {
    pub fn new(bus: Arc<Mutex<Bus>>, ppu_frame_out: Sender<Frame>) -> Self {
        Self {
            bus,
            cycles: 0,
            // registers: todo!(),
            scanline: 0,
            dot: 0,

            // rendering_enabled: todo!(),
            // in_hblank: todo!(),
            // bg_pattern_table_address: todo!(),
            // spr_pattern_table_address: todo!(),
            // bg_pixel_buffer: todo!(),
            // spr_pixel_buffer: todo!(),
            // control_register: todo!(),
            // mask_register: todo!(),
            // status_register: todo!(),
            // oam_address: todo!(),
            // scroll_register: todo!(),
            // ppu_address: todo!(),
            // ppu_data: todo!(),
            // address_latch: todo!(),
            // high_byte_buffer: todo!(),
            // sprite_evaluation_state: todo!(),
            // sprite_shift_registers: todo!(),
            // fine_x_scroll: todo!(),
            // y_scroll: todo!(),
            // x_scroll: todo!(),
            // tile_data: todo!(),
            // tile_latch: todo!(),
            // sprite_count: todo!(),
            // sprite_patterns: todo!(),
            // sprite_positions: todo!(),
            // sprite_priorities: todo!(),
            // sprite_indexes: todo!(),
            // sprite_zero_hit: todo!(),
            // sprite_overflow: todo!(),
            current_tile_data: 0,
            tile_shift_registers: [0; 2],
            frame_color_index_cache: [0; 256 * 240],

            channels: PpuChannels { ppu_frame_out },
            ppustatus: 0,
            nmi_status: false,
        }
    }

    fn read(&self, address: u16) -> u8 {
        let read_result = self.bus.lock().unwrap().ppu_read(address);
        read_result
    }

    fn write(&mut self, address: u16, data: u8) {
        self.bus.lock().unwrap().ppu_write(address, data);
    }

    fn read_oam(&self, address: u16) -> u8 {
        let read_result = self.bus.lock().unwrap().oam.read(address);
        read_result
    }

    fn read_reg(&self, reg: PpuRegister) -> u8 {
        self.bus.lock().unwrap().cpu_read(0x2000 + (reg as u16))
    }

    fn write_reg(&self, reg: PpuRegister, data: u8) {
        self.bus
            .lock()
            .unwrap()
            .cpu_write(0x2000 + (reg as u16), data)
    }

    pub fn reset(&mut self) {
        self.cycles = 0;
        // ... reset other fields
    }

    pub fn get_current_log(&mut self) -> String {
        "ppu 测试".to_string()
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
            let high_plane = tile_data[i + 8];
            for j in 0..8 {
                let color_index = ((high_plane >> (7 - j) & 1) << 1) | (low_plane >> (7 - j) & 1);
                self.tile_shift_registers[0] =
                    (self.tile_shift_registers[0] << 2) | color_index as u16;
            }
        }

        // 在完成图块数据的加载后，我们需要将移位寄存器中的数据复制到“当前图块”寄存器中
        // “当前图块”寄存器将被用来在绘制期间提供像素数据
        self.current_tile_data = self.tile_shift_registers[0];
    }

    fn fetch_tile_data(&mut self,tile_data_address: u16,palette_address: u16) -> Vec<[u8; 8]> {
        // 读取图块数据,返回一个8x8的颜色索引数组
        let mut tile_data = vec![[0; 8]; 8];    

        for y in 0..8 {
            let tail_data_low = self.read(tile_data_address + y);
            let tail_data_high = self.read(tile_data_address + y + 8);
            for x in 0..8 {
                let low = (tail_data_low >> (7 - x)) & 1;
                let high = (tail_data_high >> (7 - x)) & 1;
                let color_index = (high << 1) | low;
                let color = self.read(palette_address + color_index as u16);
                tile_data[y as usize][x as usize] = color;
            }
        }
        tile_data
    }

    fn get_nametable(&self, index: u8) -> [u8; 0x3c0] {
        let nametable_base = match index {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2C00,
            _ => unreachable!(),
        };
        let mut nametable = [0; 0x3c0];
        for i in 0..0x3c0 {
            nametable[i] = self.read(nametable_base + i as u16);
        }
        nametable
    }

    fn get_attribute_table(&self, index: u8) -> [u8; 0x40] {
        let nametable_base = match index {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2C00,
            _ => unreachable!(),
        };
        let mut attribute_table = [0; 0x40];
        for i in 0..0x40 {
            attribute_table[i] = self.read(nametable_base + 0x3c0 + i as u16);
        }
        attribute_table
    }

    pub fn test_render_background(&mut self) {
        // 背景暴力渲染，直接输出当前name table的数据，scorll和mask都不管
        // 一帧中pattern_table_base不会改变，所以这里直接写死
        let name_table = self.get_nametable(0);
        let attribute_table = self.get_attribute_table(0);
        let ctrl = self.read_reg(PpuRegister::PpuCtrl);
        let pattern_table_base = match ctrl >> 4 & 1 {
            0 => 0x0000,
            1 => 0x1000,
            _ => unreachable!(),
        };

        let mut tile_data: [[u8; 8]; 8] = [[0; 8]; 8];
        for i in 0..0x3c0 {
            // 获取tile 索引
            let tile_index = name_table[i];
            // 获取调色板索引
            let pattern_x = i % 32;
            let pattern_y = i / 32;
            let attribute = attribute_table[(pattern_x / 4) + (pattern_y / 4) * 8];
            let palette_index =
                (attribute >> (((pattern_x % 4) / 2) + (((pattern_y % 4) / 2) * 2)) * 2) & 0x3;
            let palette_address = 0x3f00 + palette_index as u16 * 4;
            let tile_data_address = pattern_table_base + tile_index as u16 * 16;
            // 获取tile数据
            // let tail_data_color_index = self.read_tail_data(tile_data_address);

            for y in 0..8 {
                let tail_data_low = self.read(tile_data_address + y);
                let tail_data_high = self.read(tile_data_address + y + 8);
                for x in 0..8 {
                    let low = (tail_data_low >> (7 - x)) & 1;
                    let high = (tail_data_high >> (7 - x)) & 1;
                    let color_index = (high << 1) | low;
                    let color = self.read(palette_address + color_index as u16);
                    tile_data[y as usize][x as usize] = color;
                    let frame_x = pattern_x * 8 + x;
                    let frame_y = pattern_y * 8 + y as usize;
                    self.frame_color_index_cache[frame_y * 256 + frame_x] =
                        tile_data[y as usize][x];
                }
            }
        }
    }

    pub fn test_render_sprite(&mut self) {
        // PPU的OAM是一个256字节的内存，存储了屏幕上最多64个精灵的信息。每个精灵的信息占4个字节，分别是：
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
        for i in (0..256).step_by(4).rev() {
            // 获取当前精灵的信息
            let sprite_y = self.read_oam(i);
            let sprite_tile_index = self.read_oam(i + 1);
            let sprite_attributes = self.read_oam(i + 2);
            let sprite_x = self.read_oam(i + 3);

            if sprite_attributes >> 5 & 1 != 0 {
                // 精灵不可见
                continue;
            }
            let flip_horizontally = sprite_attributes >> 6 & 1 == 1;
            let flip_vertically = sprite_attributes >> 7 & 1 == 1;

            // 调色板索引，每个精灵使用一个4色调色板，存储在0x3F10、0x3F14、0x3F18或0x3F1C的4个地址中
            let palette_address = 0x3F10 + ((sprite_attributes & 0x3) * 4) as u16;

            let ppuctrl = self.read_reg(PpuRegister::PpuCtrl);

            let mut sprite_tile_data:Vec<[u8; 8]>;
            if (ppuctrl >> 5 & 1) != 0 {
                // 8x16模式下，精灵的模式索引是2个字节，其中低位字节指定上半部分的图案，高位字节指定下半部分的图案
                // 由于每个图案是8x8，所以上下两个图案的地址是连续的
                // 图案表的基地址
                let pattern_table_base: u16 = match sprite_tile_index & 1 {
                    0 => 0x0000,
                    1 => 0x1000,
                    _ => unreachable!(),
                };
                let sprite_tile_data_address = pattern_table_base + (sprite_tile_index>>1) as u16 * 0x20;
                sprite_tile_data = self.fetch_tile_data(sprite_tile_data_address, palette_address);
                let mut sprite_tile_data_next = self.fetch_tile_data(sprite_tile_data_address + 0x10, palette_address);
                sprite_tile_data.append(&mut sprite_tile_data_next);
            } else {
                // 8x8模式
                // 图案表的基地址
                let pattern_table_base = match ppuctrl >> 3 & 1 {
                    0 => 0x0000,
                    1 => 0x1000,
                    _ => unreachable!(),
                };
                // 获取精灵的图案数据地址
                let sprite_tile_data_address = pattern_table_base + sprite_tile_index as u16 * 0x10;
                sprite_tile_data = self.fetch_tile_data(sprite_tile_data_address, palette_address);
            }
            // 旋转
            sprite_tile_data = self.rotate(sprite_tile_data, flip_horizontally, flip_vertically);
            // 写入帧缓存 sprite_tile_data：Vec<[u8; 8]>
            self.draw_sprite(sprite_x as usize, sprite_y as usize, sprite_tile_data)
        
        }
    }


    fn draw_sprite(&mut self, sprite_x: usize, sprite_y: usize, sprite_tile_data: Vec<[u8; 8]>) {
        for (y, row) in sprite_tile_data.iter().enumerate() {
            for (x, &color_index) in row.iter().enumerate() {
                let frame_x = sprite_x + x;
                let frame_y = sprite_y + y;
                if frame_x < 256 && frame_y < 240 { // assuming your frame is 256x240
                    self.frame_color_index_cache[frame_y * 256 + frame_x] = color_index;
                }
            }
        }
    }

    fn rotate(&mut self, mut tile_data: Vec<[u8; 8]>, flip_h:bool ,flip_v:bool) -> Vec<[u8; 8]> {
        if flip_h {
            for row in tile_data.iter_mut() {
                row.reverse();
            }
        }
    
        if flip_v {
            tile_data.reverse();
        }
    
        tile_data
    }

    fn set_nmi(&mut self, nmi: bool) {
        // todo： 优化，ppu只能设置nmi，不需要读取其他的
        // nmi 位在第2位
        let interrupt_status = self.bus.lock().unwrap().interrupt_status;
        self.bus.lock().unwrap().interrupt_status = if nmi {
            interrupt_status | 0b00000010
        } else {
            interrupt_status & 0b11111101
        };
    }

    pub fn step(&mut self) {
        match self.dot {
            0 => {
                // 在每个扫描线的开始，我们可能需要做一些准备工作
                // self.start_of_scanline();
            }
            1..=256 | 321..=336 => {
                // 在可见扫描线和两个“空闲”周期中，PPU 需要获取背景和精灵的图块数据
                // self.fetch_tile_data();
            }
            257..=320 => {
                // 在这个阶段，PPU 需要获取下一行将要显示的精灵的数据
                ()
            }
            337..=340 => {
                // 在每个扫描线的最后几个周期中，PPU 将进行一些清理工作
                ()
            }
            _ => unreachable!(),
        }

        // 更新 PPU 的当前周期和扫描线
        self.cycles += 1;
        self.dot += 1;
        if self.dot > 340 {
            self.dot = 0;
            self.scanline += 1;
            if self.scanline == 241 {
                self.ppustatus |= 0x80;
                self.write_reg(PpuRegister::PpuStatus, self.ppustatus);
                self.test_render_background();
                self.test_render_sprite();
                self.channels
                    .ppu_frame_out
                    .send(Frame {
                        data: self.frame_color_index_cache.to_vec(),
                        width: 256,
                        height: 240,
                    })
                    .expect("send frame error");
            }
            if self.scanline > 241 && self.scanline < 261 {
                // vblank期间，如果设置了nmi，那么就触发nmi
                let ppuctrl = self.read_reg(PpuRegister::PpuCtrl);
                if (self.ppustatus & 0x80 == 0x80) && (ppuctrl & 0x80 == 0x80) {
                    if self.nmi_status != true {
                        self.set_nmi(true);
                        self.nmi_status = true;
                    }
                } else {
                    if self.nmi_status != false {
                        self.set_nmi(false);
                        self.nmi_status = false;
                    }
                    // self.set_nmi(false);
                }
            }

            if self.scanline > 261 {
                if self.nmi_status != false {
                    self.set_nmi(false);
                    self.nmi_status = false;
                }
                self.ppustatus &= 0x7f;
                self.write_reg(PpuRegister::PpuStatus, self.ppustatus);
                self.scanline = 0;
            }
        }
    }

    // ... other Ppu methods
}
