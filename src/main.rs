use fc_emulator_rs::tests::{test_egui,test_rom,test_ppu,test_run};
use env_logger;
fn main() {
    // env_logger::init();
    test_run::run_test();
    print!("Hello World")
}

