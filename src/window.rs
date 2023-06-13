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
use rfd::FileDialog;

#[derive( serde::Serialize)]

#[serde(default)] // if we add new fields, give them default values when deserializing old state
struct Status {
    paused: bool,
    rom_path: String,
    log_enabled: bool,
    run_to_cycle: String,
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
    ppuctrl: u8,
    ppumask: u8,
    ppustatus: u8,
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
            window_status: Status { 
                paused: true,
                rom_path: String::from(""),
                log_enabled: false,
                run_to_cycle: "0".to_string(),
            },
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
                    ppuctrl: 0,
                    ppumask: 0,
                    ppustatus: 0,
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
        self.emulator_state.ppu_state.ppuctrl = self.emulator.bus.borrow().registers.ppuctrl;
        self.emulator_state.ppu_state.ppumask = self.emulator.bus.borrow().registers.ppumask;
        self.emulator_state.ppu_state.ppustatus = self.emulator.bus.borrow().registers.ppustatus;
    }
    fn update_bus_state(&mut self) {
        let interrupt_status =  self.emulator.bus.borrow_mut().interrupt_status;
        self.emulator_state.bus_state.nmi = interrupt_status>>1 & 1 == 1;
        self.emulator_state.bus_state.irq = interrupt_status & 1 == 1;
        self.emulator_state.bus_state.reset = interrupt_status>>2 & 1 == 1;
    }
    fn update_emulator_state(&mut self) {
        self.update_cpu_state();
        self.update_ppu_state();
        self.update_bus_state();
    }

    fn loop_to_frame(&mut self) -> Frame {
        while !self.emulator.ppu.new_frame {
            self.emulator.cpu_step();
        }
        self.emulator.ppu.new_frame = false;
        Frame {
            data: self.emulator.ppu.frame_color_index_cache.to_vec(),
            width: 256,
            height: 240,
        }
    }

    fn run_to_cycle(&mut self, cycle: u64) {
        while self.emulator.cpu.cpu_cycle < cycle {
            self.emulator.cpu_step();
        }
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



impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.window_status.paused {
            // 接收新图像
            if self.current_time.elapsed().as_secs_f64() > 1.0 / self.sample_frq {
                let new_frame = self.loop_to_frame();
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
            // 加载rom
            ui.horizontal(|ui| {
                ui.label(format!("ROM: {}",&self.window_status.rom_path));
                if ui.button("Load").clicked() {
                    let files = FileDialog::new()
                        .add_filter("nes", &["nes"])
                        .set_directory("/")
                        .pick_file().unwrap();
                    self.window_status.rom_path = files.file_name().unwrap().to_str().unwrap().to_string();
                    self.emulator.load_rom(files.to_str().unwrap());
                    self.update_emulator_state();
                }
            });
            // 重新加载
            ui.horizontal(|ui| {
                if ui.button("Reset").clicked() {
                    self.emulator.reset();
                    self.update_emulator_state();
                }
                if ui.button("HardReset").clicked() {
                    self.emulator.hard_reset();
                    self.update_emulator_state();
                }
            });

            // 是否记录日志
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.window_status.log_enabled, "Log");
            });
            
            
            // 增加暂停按钮
            if ui.button(if self.window_status.paused {"继续"} else {"暂停"}).clicked(){
                self.window_status.paused = !self.window_status.paused;
                self.update_emulator_state();
            }
            // 按钮使能
            ui.set_enabled(self.window_status.paused);
            if ui.button("cpu单步执行（step）").clicked(){
                self.emulator.cpu_step_debug();
                self.update_emulator_state();
                println!("{}",self.emulator.get_log());
            }
            if ui.button("cpu时钟执行（clock）").clicked(){
                self.emulator.cpu_clock_debug();
                self.update_emulator_state();
            }
            if ui.button("下一帧").clicked(){
                let new_frame = self.loop_to_frame();
                self.image = self.frame_to_color_image(&new_frame);
                self.update_emulator_state();
            }
            // 运行到cyc为止,输入数字
            ui.horizontal(|ui|  {
                ui.label("运行到");
                ui.add( egui::TextEdit::singleline(&mut self.window_status.run_to_cycle).desired_width(100.0));
                if ui.button("运行").clicked(){
                    self.run_to_cycle(self.window_status.run_to_cycle.parse::<u64>().unwrap());
                    self.update_emulator_state();
                }
            });
        });

        // 右侧状态栏显示cpu、ppu、bus状态
        egui::SidePanel::right("side_panel_right").show(ctx, |ui| {
            ui.heading("CPU");
            ui.label(format!("A: {:02X}\nX: {:02X}\nY: {:02X}\nPC: {:04X}\nSP: {:02X}\nP: {:08b}:{:02X}\n   CZIDB-VN\ncycles: {}",
                self.emulator_state.cpu_state.a,
                self.emulator_state.cpu_state.x,
                self.emulator_state.cpu_state.y,
                self.emulator_state.cpu_state.pc,
                self.emulator_state.cpu_state.sp,
                self.emulator_state.cpu_state.p,
                self.emulator_state.cpu_state.p,
                self.emulator_state.cpu_state.cycles));

            // 当前执行执行的指令反编译
            let disasm = self.emulator.cpu.disassemble_instruction_short();
            ui.label(format!("Disasm:{}", disasm));
            
            ui.separator();

            ui.heading("PPU");
            ui.label(format!("Scanline: {}", self.emulator_state.ppu_state.scanline));
            ui.label(format!("Dot: {}", self.emulator_state.ppu_state.dot));
            ui.label(format!("Cycles: {}", self.emulator_state.ppu_state.cycles));
            ui.label(format!("ppuctrl: {:08b}", self.emulator_state.ppu_state.ppuctrl));
            ui.label(format!("ppumask: {:08b}", self.emulator_state.ppu_state.ppumask));
            ui.label(format!("ppustatus: {:08b}", self.emulator_state.ppu_state.ppustatus));
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
            self.emulator.refresh_input(input_state);

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
