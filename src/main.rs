mod tests;
mod cpu;
mod memory;

use tests::test_rom;

fn main() {
    test_rom::run_test();
}
