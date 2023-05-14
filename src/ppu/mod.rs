// mod.rs

// 导入同级目录下的其他文件作为子模块
pub mod renderer;

// 导出子模块，使其可以在父级作用域（在这个例子中就是`ppu`）被访问
pub use self::renderer::Renderer;


use winit::{
    dpi::PhysicalSize,
    window::{Window, WindowBuilder},
};

// PPU 结构体，包含了PPU的状态
pub struct Ppu {
    // 这里可以包含PPU的状态，例如当前的扫描线，当前的像素等
    // ...
    pub cycle: u32, // 当前循环
    pub scanline: u32, // 当前扫描线
    pub frame: u32, // 当前帧
    
    pub vram: [u8; 0x4000], // VRAM，包含了名称表，属性表和图案表

    // 寄存器
    pub ppu_ctrl: u8, // PPUCTRL寄存器
    pub ppu_mask: u8, // PPUMASK寄存器
    pub ppu_status: u8, // PPUSTATUS寄存器
    pub oam_addr: u8, // OAMADDR寄存器
    pub oam_data: [u8; 256], // OAMDATA寄存器
    pub ppu_scroll: u8, // PPUSCROLL寄存器
    pub ppu_addr: u8, // PPUADDR寄存器
    pub ppu_data: u8, // PPUDATA寄存器
    
    // 其他状态
    pub nmi_occurred: bool, // 是否发生了非屏蔽中断
    pub nmi_output: bool, // NMI输出
    pub nmi_previous: bool, // 上一次的NMI
    pub nmi_delay: u8, // NMI延迟
    // ... 其他可能需要的状态
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            // 初始化PPU的状态
            // ...
            cycle: 0,
            scanline: 0,
            frame: 0,
            vram: [0; 0x4000],
            ppu_ctrl: todo!(),
            ppu_mask: todo!(),
            ppu_status: todo!(),
            oam_addr: todo!(),
            oam_data: todo!(),
            ppu_scroll: todo!(),
            ppu_addr: todo!(),
            ppu_data: todo!(),
            nmi_occurred: todo!(),
            nmi_output: todo!(),
            nmi_previous: todo!(),
            nmi_delay: todo!(),
            
        }
    }

    pub fn get_frame(&self) -> Vec<u8> {
        // 从PPU中获取当前帧的图像
        todo!()
    }

    // PPU的方法，例如读写PPU内存，开始新的帧等
    // ...
}
