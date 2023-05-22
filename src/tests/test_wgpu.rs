use std::time::{Duration, Instant};

use crate::emulator::{Emulator};
use crate::ppu::renderer::Frame;
use crate::utils::Window;

const TARGET_FRAME_DURATION: Duration = Duration::from_secs(1);

pub fn run_test() {
    // 创建一个新的模拟器和窗口
    let mut emulator = Emulator::new();
    let mut window = pollster::block_on(Window::new());

    let mut a=0;
    // 主循环
    loop {
        // 获取当前的时间
        let start = Instant::now();

        // 获取新的帧并将其绘制到窗口上
        // let frame = emulator.get_frame();
        let frame = Frame{
            data:vec![2;256*240*4],
            width:256,
            height:240,
        };
        
        window.present(&frame);

        // 检查用户是否请求关闭窗口
        if window.check_close() {
            break;
        }

        // 等待下一帧
        let elapsed = start.elapsed();
        if elapsed < TARGET_FRAME_DURATION {
            std::thread::sleep(TARGET_FRAME_DURATION - elapsed);
        }
    }
}
