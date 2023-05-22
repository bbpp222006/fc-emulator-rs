use crate::utils::Frame;
use crossbeam::channel::{bounded, select, Receiver, Sender};
use eframe::egui;
use egui::{Color32, ColorImage};
use egui_extras::image::RetainedImage;
use rand::Rng;
use std::thread;
use std::time::Duration;

pub struct MyApp {
    image: RetainedImage,
    pip_frame_data_out: Receiver<Frame>,
}

impl MyApp {
    /// Called once before the first frame.
    pub fn new(pip_frame_data_out: Receiver<Frame>) -> Self {
        Self {
            image: RetainedImage::from_image_bytes(
                "rust-logo-256x256.png",
                include_bytes!("rust-logo-256x256.png"),
            )
            .unwrap(),
            pip_frame_data_out,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        select! {
            recv(self.pip_frame_data_out) -> new_frame =>{
                let new_frame= new_frame.expect("接收新图像时发生错误");
                self.image = frame_to_color_image(&new_frame);
                println!("更改图像");
            }
            default =>(),
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("This is an image:");
            ui.add(egui::Image::new(
                self.image.texture_id(ctx),
                ui.available_size(),
            ));
        });
        ctx.request_repaint();
    }
}

fn frame_to_color_image(frame: &Frame) -> RetainedImage {
    // 确保数据长度与图像尺寸匹配
    assert_eq!(
        (frame.width * frame.height * 4) as usize,
        frame.data.len(),
        "数据长度与图像尺寸不匹配"
    );

    // 创建 `ColorImage`

    RetainedImage::from_color_image(
        "debug_name",
        ColorImage::from_rgba_unmultiplied(
            [frame.width as usize, frame.height as usize],
            &frame.data,
        ),
    )
}
