mod hw;
mod utils;

use log::error;

use hw::cpu::Cpu;

const TEST_ROM_PATH: &str = "hello.gb";

fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();

    let mut cpu = Cpu::new();

    if let Err(e) = cpu.load_rom(TEST_ROM_PATH) {
        error!("Error loading ROM: {}", e);
        std::process::exit(1);
    }

    loop {
        if let Err(e) = cpu.step() {
            error!("Error during CPU loop: {}", e);
            std::process::exit(1);
        }
    }
}
