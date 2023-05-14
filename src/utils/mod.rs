pub mod window;
// pub mod bus;

pub use self::window::Window;


pub enum GlobalSignal{
    Clock,
    GetLog,
    Reset,
    Step,
} 