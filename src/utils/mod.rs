// pub mod window;
// pub mod bus;

// pub use self::window::Window;
mod palettes;


pub use palettes::Palettes;

#[derive(Clone, Copy)]
pub enum GlobalSignal{
    Clock,
    GetLog,
    Reset,
    Step,
} 

pub struct Frame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl Frame {
    pub fn new() -> Self {
        Frame {
            data: vec![0; 256 * 240],
            width: 256,
            height: 240,
        }
    }
}
    