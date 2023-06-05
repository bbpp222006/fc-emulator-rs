use crate::utils::Frame;
use crate::utils::Palettes;
use crossbeam::channel::{bounded, select, Receiver, Sender};
use eframe::egui;
use egui::{Color32, ColorImage};
use egui_extras::image::RetainedImage;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::time::Instant;

pub struct MyApp {
    image: RetainedImage,
    pip_frame_data_out: Receiver<Frame>,
    pip_input_stream_in: Sender<HashSet<egui::Key>>,
    palette: Palettes,
    current_time: std::time::Instant,
    fps_target: f64,
    fps_history: VecDeque<f64>,
    sample_frq: f64,
    fps_show: f64,
}

impl MyApp {
    /// Called once before the first frame.
    pub fn new(pip_frame_data_out: Receiver<Frame>,pip_input_stream_in:Sender<HashSet<egui::Key>>) -> Self {
        let fps_target = 60.0;
        Self {
            image: RetainedImage::from_image_bytes(
                "rust-logo-256x256.png",
                include_bytes!("rust-logo-256x256.png"),
            )
            .unwrap(),
            pip_frame_data_out,
            pip_input_stream_in,
            palette: Palettes::new(),
            current_time: std::time::Instant::now(),
            fps_target,
            fps_history:VecDeque::from([0.0;20]),
            sample_frq: fps_target,
            fps_show:0.0,
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
        
        // 从管道中接收新图像
        if self.current_time.elapsed().as_secs_f64() > 1.0/self.sample_frq {
            if let Some(new_frame) = self.pip_frame_data_out.try_recv().ok(){
                self.image = self.frame_to_color_image(&new_frame);
                self.fps_history.push_front(1.0/self.current_time.elapsed().as_secs_f64());
                self.fps_history.pop_back();
                let sum: f64 = self.fps_history.iter().sum();
                self.fps_show = if !self.fps_history.is_empty() { sum / self.fps_history.len() as f64 } else { 0.0 };
                self.current_time = std::time::Instant::now();
            }
        }

        if self.fps_history.front().unwrap_or(&0.0)>&self.fps_target{
            self.sample_frq-=1.0;
        }else {
            self.sample_frq+=1.0;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            
            ui.heading(format!("FPS: {:.2}, sample frq: {:.2}", self.fps_show,self.sample_frq));
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
                    match self.pip_input_stream_in.try_send(input_state){
                        Ok(_) => {
                            
                        },
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
