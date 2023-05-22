use eframe::egui;
use egui_extras::image::RetainedImage;
use crossbeam::channel::{bounded, select, Receiver, Sender};
use egui::{ColorImage, Color32};
use rand::Rng;
use std::thread;
use std::time::Duration;


pub struct MyApp {
    image: RetainedImage,
    pip_frame_data_out:Receiver<RetainedImage>,
}
 

impl MyApp {
    /// Called once before the first frame.
    pub fn new(pip_frame_data_out:Receiver<RetainedImage>) -> Self {
        Self {
            image: RetainedImage::from_image_bytes(
                "rust-logo-256x256.png",
                include_bytes!("rust-logo-256x256.png"),
            )
            .unwrap(),
            pip_frame_data_out
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        select! {
            recv(self.pip_frame_data_out) -> new_frame =>{
                self.image = new_frame.expect("接收新图像时发生错误");
                println!("更改图像");
            }
            default =>(),
        };
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("This is an image:");
            ui.add(
                egui::Image::new(self.image.texture_id(ctx), ui.available_size())
            );
        });
        ctx.request_repaint();
    }
}