use fc_emulator_rs::tests::{test_egui,test_rom,test_ppu};
use env_logger;
fn main() {
    env_logger::init();
    test_ppu::run_test();
}

