// pub mod window;
// pub mod bus;

// pub use self::window::Window;


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