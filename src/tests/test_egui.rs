
use crate::utils::Frame;
use crate::window::MyApp;
use eframe::egui;
use egui_extras::image::RetainedImage;
use crossbeam::channel::{bounded, select, Receiver, Sender};
use egui::{ColorImage, Color32};
use rand::Rng;
use std::thread;
use std::time::Duration;

fn create_random_color_image(width: u32, height: u32) -> Frame {
    let size = (width * height) as usize;
    let mut rng = rand::thread_rng();
    let mut data: Vec<u8> = Vec::with_capacity(size);
    for _ in 0..size {
        // 0-64 随机数
        let a = rng.gen_range(0..64);
        data.push(a);
    }

    Frame{
        data,
        width,
        height,
    }
}
pub fn run_test() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(340.0, 261.0)),
        ..Default::default()
    };
    let pip_frame_data: (Sender<Frame>, Receiver<Frame>) = bounded(1);
    let pip_input_stream = bounded(1);

    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs_f32(1.0/30.0));
        print!("1");
        let test_data = create_random_color_image(256,240);
        pip_frame_data.0.send(test_data).expect("发送出错");
    });

    eframe::run_native(
        "Show an image with eframe/egui",
        options,
        Box::new(|cc| Box::new(MyApp::new(pip_frame_data.1,pip_input_stream.0))),
    )
    .unwrap(); 
}

