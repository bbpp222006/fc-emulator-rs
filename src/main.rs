use fc_emulator_rs::tests::{test_rom,test_run};
use env_logger;
fn main() {
    // env_logger::init();
    test_run::run_test();
    print!("Hello World")
}

