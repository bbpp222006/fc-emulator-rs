use fc_emulator_rs::tests::test_wgpu;
use env_logger;
fn main() {
    env_logger::init();
    pollster::block_on(test_wgpu::run_test());
}
