use fc_emulator_rs::tests::test_egui;
use env_logger;
fn main() {
    env_logger::init();
    test_egui::run_test();
}

