
use crate::utils::window::MyApp;
use eframe::egui;
use egui_extras::image::RetainedImage;
use crossbeam::channel::{bounded, select, Receiver, Sender};
use egui::{ColorImage, Color32};
use rand::Rng;
use std::thread;
use std::time::Duration;

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
pub fn run_test() {
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

