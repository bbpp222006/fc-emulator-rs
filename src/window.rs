use crate::cpu::cpu::Interrupt;
use crate::emulator::Emulator;
use crate::utils::Frame;
use crate::utils::Palettes;
use crossbeam::channel::{bounded, select, Receiver, Sender};
use eframe::egui;
use egui::Button;
use egui::{Color32, ColorImage};
use egui_extras::image::RetainedImage;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::time::Instant;

#[derive( serde::Serialize)]

#[serde(default)] // if we add new fields, give them default values when deserializing old state
struct Status {
    paused: bool,
}

struct CpuState {
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    sp: u8,
    p: u8,
    cycles: u64,
    log: VecDeque<String>,
}
struct PpuState{
    scanline: u16,
    dot: u16,
    cycles: u64,
}
struct BusState{
    nmi: bool,
    irq: bool,
    reset: bool,
}
struct EmulatorState {
    cpu_state: CpuState,
    ppu_state: PpuState,
    bus_state: BusState,
    frame: u64,
}

pub struct MyApp {
    emulator: Emulator,
    image: RetainedImage,
    palette: Palettes,
    current_time: std::time::Instant,
    fps_target: f64,
    fps_history: VecDeque<f64>,
    sample_frq: f64,
    fps_show: f64,
    window_status: Status,
    emulator_state: EmulatorState,
}


fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!("fusion-pixel.otf")),
    );

    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("my_font".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}

impl MyApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>,emulator: Emulator) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        let fps_target = 60.0;
        Self {
            emulator,
            image: RetainedImage::from_image_bytes(
                "rust-logo-256x256.png",
                include_bytes!("rust-logo-256x256.png"),
            )
            .unwrap(),
            palette: Palettes::new(),
            current_time: std::time::Instant::now(),
            fps_target,
            fps_history: VecDeque::from([0.0; 20]),
            sample_frq: fps_target,
            fps_show: 0.0,
            window_status: Status { paused: false },
            emulator_state: EmulatorState{
                cpu_state: CpuState{
                    a: 0,
                    x: 0,
                    y: 0,
                    pc: 0,
                    sp: 0,
                    p: 0,
                    cycles: 0,
                    log: VecDeque::with_capacity(100),
                },
                ppu_state: PpuState{
                    scanline: 0,
                    dot: 0,
                    cycles: 0,
                },
                bus_state: BusState{
                    nmi: false,
                    irq: false,
                    reset: false,
                },
                frame: 0,
            },
        }
    }

    fn update_cpu_state(&mut self) {
        self.emulator_state.cpu_state.a = self.emulator.cpu.registers.a;
        self.emulator_state.cpu_state.x = self.emulator.cpu.registers.x;
        self.emulator_state.cpu_state.y = self.emulator.cpu.registers.y;
        self.emulator_state.cpu_state.pc = self.emulator.cpu.registers.pc;
        self.emulator_state.cpu_state.sp = self.emulator.cpu.registers.sp;
        self.emulator_state.cpu_state.p = self.emulator.cpu.registers.p;
        self.emulator_state.cpu_state.cycles = self.emulator.cpu.cpu_cycle;
        self.emulator_state.cpu_state.log.push_front(self.emulator.cpu.get_current_log());
        if self.emulator_state.cpu_state.log.len() > 100 {
            self.emulator_state.cpu_state.log.pop_back();
        }
    }
    fn update_ppu_state(&mut self) {
        self.emulator_state.ppu_state.scanline = self.emulator.ppu.scanline;
        self.emulator_state.ppu_state.dot = self.emulator.ppu.dot;
        self.emulator_state.ppu_state.cycles = self.emulator.ppu.cycles;
    }
    fn update_bus_state(&mut self) {
        let interrupt_status =  self.emulator.bus.borrow_mut().interrupt_status;
        self.emulator_state.bus_state.nmi = interrupt_status>>2 & 1 == 1;
        self.emulator_state.bus_state.irq = interrupt_status & 1 == 1;
        self.emulator_state.bus_state.reset = interrupt_status>>3 & 1 == 1;
    }
    fn update_emulator_state(&mut self) {
        self.update_cpu_state();
        self.update_ppu_state();
        self.update_bus_state();
    }

    pub fn frame_to_color_image(&self, frame: &Frame) -> RetainedImage {
        // 确保数据长度与图像尺寸匹配
        assert_eq!(
            (frame.width * frame.height) as usize,
            frame.data.len(),
            "数据长度与图像尺寸不匹配"
        );

        // 将color index转换为rgba color
        let mut rgba_data = Vec::with_capacity(frame.data.len() * 4);
        for color_index in frame.data.iter() {
            let rgba = self.palette.colors[*color_index as usize];
            rgba.map(|c| rgba_data.push(c));
        }
        // 创建 `ColorImage`
        RetainedImage::from_color_image(
            "debug_name",
            ColorImage::from_rgba_unmultiplied(
                [frame.width as usize, frame.height as usize],
                &rgba_data,
            ),
        )
    }
}

fn loop_to_frame(emulator: &mut Emulator) -> Frame {
    while !emulator.ppu.new_frame {
        emulator.cpu_step();
    }
    emulator.ppu.new_frame = false;
    Frame {
        data: emulator.ppu.frame_color_index_cache.to_vec(),
        width: 256,
        height: 240,
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.window_status.paused {
            // 接收新图像
            if self.current_time.elapsed().as_secs_f64() > 1.0 / self.sample_frq {
                let new_frame = loop_to_frame(&mut self.emulator);
                self.image = self.frame_to_color_image(&new_frame);
                self.fps_history
                    .push_front(1.0 / self.current_time.elapsed().as_secs_f64());
                self.fps_history.pop_back();
                let sum: f64 = self.fps_history.iter().sum();
                self.fps_show = if !self.fps_history.is_empty() {
                    sum / self.fps_history.len() as f64
                } else {
                    0.0
                };
                self.current_time = std::time::Instant::now();
            }

            if !cfg!(debug_assertions) {
                // 这段代码只在release模式下执行
                if self.fps_history.front().unwrap_or(&0.0) > &self.fps_target {
                    self.sample_frq -= 1.0;
                } else {
                    self.sample_frq += 1.0;
                }
            }
        }

        // 增加暂停按钮
        egui::SidePanel::left("side_panel_left").show(ctx, |ui| {
            ui.heading("Controls");
            // 增加暂停按钮
            if ui.button(if self.window_status.paused { "继续"} else {"暂停"}).clicked(){
                self.window_status.paused = !self.window_status.paused;
                self.update_emulator_state();
            }
            // 按钮使能
            ui.set_enabled(self.window_status.paused);
            if ui.button("cpu单步执行（step）").clicked(){
                self.emulator.cpu_step();
                self.update_emulator_state();
            }
            if ui.button("cpu时钟执行（clock）").clicked(){
                self.emulator.cpu_clock();
                self.update_emulator_state();
            }
            if ui.button("ppu执行").clicked(){
                self.emulator.ppu_step();
                self.update_emulator_state();
            }
            if ui.button("下一帧").clicked(){
                let new_frame = loop_to_frame(&mut self.emulator);
                self.image = self.frame_to_color_image(&new_frame);
                self.update_emulator_state();
            }
        });

        // 右侧状态栏显示cpu、ppu、bus状态
        egui::SidePanel::right("side_panel_right").show(ctx, |ui| {
            ui.heading("CPU");
            ui.label(format!("A: {:02X}\nX: {:02X}\nY: {:02X}\nPC: {:04X}\nSP: {:02X}\nP: {:08b}\n   CZIDB-VN",
                self.emulator_state.cpu_state.a,
                self.emulator_state.cpu_state.x,
                self.emulator_state.cpu_state.y,
                self.emulator_state.cpu_state.pc,
                self.emulator_state.cpu_state.sp,
                self.emulator_state.cpu_state.p,));

            // 当前执行执行的指令反编译
            let disasm = self.emulator.cpu.disassemble_instruction_short();
            ui.label(format!("Disasm:{}", disasm));
            

            ui.separator();

            ui.heading("PPU");
            ui.label(format!("Scanline: {}", self.emulator_state.ppu_state.scanline));
            ui.label(format!("Dot: {}", self.emulator_state.ppu_state.dot));
            ui.label(format!("Cycles: {}", self.emulator_state.ppu_state.cycles));
            ui.separator();
            ui.heading("BUS");
            ui.label(format!("NMI: {}", self.emulator_state.bus_state.nmi));
            ui.label(format!("IRQ: {}", self.emulator_state.bus_state.irq));
            ui.label(format!("RESET: {}", self.emulator_state.bus_state.reset));
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!(
                "FPS: {:.2}, sample frq: {:.2}",
                self.fps_show, self.sample_frq
            ));
            ui.add(egui::Image::new(self.image.texture_id(ctx), {
                let min_x = 256.0;
                let min_y = 240.0;
                let (current_x, current_y) = (ui.available_size().x, ui.available_size().y);
                let scale = ((current_x / min_x).min(current_y / min_y)).max(1.0);
                egui::vec2(scale * min_x, scale * min_y)
            }));
            // 接收输入
            let input_state = ui.input(|i| i.keys_down.clone());
            // println!("{:?}", input_state);
            if !input_state.is_empty() {
                match self.emulator.pip_input_stream.0.try_send(input_state) {
                    Ok(_) => {}
                    Err(err) => match err {
                        crossbeam::channel::TrySendError::Full(_) => {
                            // println!("输入管道已满,直接丢弃");
                        }
                        crossbeam::channel::TrySendError::Disconnected(_) => {
                            // println!("输入管道已断开");
                        }
                    },
                };
            }
        });
        ctx.request_repaint();
    }
}

// fn lorem_ipsum(ui: &mut egui::Ui) {
//     ui.with_layout(
//         egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(true),
//         |ui| {
//             ui.label(egui::RichText::new(crate::LOREM_IPSUM_LONG).small().weak());
//             ui.add(egui::Separator::default().grow(8.0));
//             ui.label(egui::RichText::new(crate::LOREM_IPSUM_LONG).small().weak());
//         },
//     );
// }
