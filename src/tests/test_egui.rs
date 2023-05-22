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



fn create_random_color_image(width: usize, height: usize) -> ColorImage {
    let mut rng = rand::thread_rng(); // 创建随机数生成器

    // 生成随机的颜色分量
    let r = rng.gen_range(0..=255);
    let g = rng.gen_range(0..=255);
    let b = rng.gen_range(0..=255);
    let a = rng.gen_range(0..=255); // alpha 通道，你也可以直接设为 255 以保证颜色不透明

    // 创建 Color32
    let color = Color32::from_rgba_unmultiplied(r, g, b, a);

    // 创建 ColorImage
    let image = ColorImage::new([width , height ], color);

    image
}

#[cfg(test)]
fn test() {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(340.0, 261.0)),
        ..Default::default()
    };
    let pip_frame_data: (Sender<RetainedImage>, Receiver<RetainedImage>) = bounded(1);

    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        print!("1");
        let test_data = RetainedImage::from_color_image("debug_name",create_random_color_image(256,240));
        pip_frame_data.0.send(test_data).expect("发送出错");
    });

    eframe::run_native(
        "Show an image with eframe/egui",
        options,
        Box::new(|cc| Box::new(MyApp::new(pip_frame_data.1))),
    )
    .unwrap(); 

    
}
