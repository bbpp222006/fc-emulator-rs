use crate::utils::Frame;
use crate::utils::Palettes;
use crossbeam::channel::{bounded, select, Receiver, Sender};
use eframe::egui;
use egui::Key;
use egui::{Color32, ColorImage};
use egui_extras::image::RetainedImage;
use rand::Rng;
use std::collections::HashSet;
use std::thread;
use std::time::Duration;

pub struct MyApp {
    image: RetainedImage,
    pip_frame_data_out: Receiver<Frame>,
    pip_input_stream_in: Sender<HashSet<egui::Key>>,
    palette: Palettes,
}

impl MyApp {
    /// Called once before the first frame.
    pub fn new(pip_frame_data_out: Receiver<Frame>,pip_input_stream_in:Sender<HashSet<egui::Key>>) -> Self {
        Self {
            image: RetainedImage::from_image_bytes(
                "rust-logo-256x256.png",
                include_bytes!("rust-logo-256x256.png"),
            )
            .unwrap(),
            pip_frame_data_out,
            pip_input_stream_in,
            palette: Palettes::new(),
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
        select! {
            recv(self.pip_frame_data_out) -> new_frame =>{
                let new_frame= new_frame.expect("接收新图像时发生错误");
                self.image = self.frame_to_color_image(&new_frame);
                // println!("更改图像");
            }
            default =>(),
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("This is an image:");
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
                if ! input_state.is_empty() {
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
