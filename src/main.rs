use fc_emulator_rs::tests::test_rom;
use env_logger;
fn main() {
    env_logger::init();
    test_rom::run_test();
}
