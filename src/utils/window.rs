use winit::{
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{Window as WinitWindow, WindowBuilder},
};

use crate::ppu::renderer::{Renderer,Frame};

pub struct Window {
    window: WinitWindow,
    event_loop: Option<EventLoop<()>>,
    renderer: Renderer,
    close_requested: bool,
}

impl Window {
    pub async fn new() -> Self {
        // 创建一个新的事件循环和窗口
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        // 创建一个新的渲染器
        let renderer = Renderer::new(&window).await;

        Self {
            window,
            event_loop: Some(event_loop),
            renderer,
            close_requested: false,
        }
    }

    pub fn present(&mut self, frame: &Frame) {
        // 使用渲染器将帧绘制到屏幕上
        self.renderer.render(frame);
    }

    pub fn check_close(&self) -> bool {
        // 检查用户是否请求关闭窗口
        self.close_requested
    }
    
    pub fn run(mut self) {
        // 运行窗口的事件循环
        let event_loop = self.event_loop.take().unwrap();
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                winit::event::Event::WindowEvent { event: winit::event::WindowEvent::CloseRequested, .. } => {
                    self.close_requested = true;
                }
                winit::event::Event::WindowEvent { event: winit::event::WindowEvent::Resized(size), .. } => {
                    self.renderer.resize(size);
                }
                _ => {}
            }
        });
    }
}

