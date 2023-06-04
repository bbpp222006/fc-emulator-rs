// mod.rs

// 导入同级目录下的其他文件作为子模块
mod renderer;
mod ppu;
// 导出子模块，使其可以在父级作用域（在这个例子中就是`ppu`）被访问
// pub use self::renderer::Renderer;

pub use ppu::{start_ppu_thread,Ppu};