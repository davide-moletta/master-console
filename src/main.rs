mod hw;
mod utils;

use log::error;
use minifb::{Key, Scale, Window, WindowOptions};
use std::time::{Duration, Instant};

use hw::cpu::Cpu;
use hw::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};

const TEST_ROM_PATH: &str = "hello.gb";
const WINDOW_NAME: &str = "Master Console - Game Boy";

// The Game Boy runs at ~59.7 frames per second
// 1 second / 59.7 = ~16,750 microseconds per frame
const FRAMERATE: u64 = 16750;
// 154 lines × 456 cycles per line
const APPROX_CYCLES_PER_FRAME: u32 = 154 * 456;

fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();

    let mut cpu = Cpu::new(TEST_ROM_PATH);

    // Generate window for rendering
    let mut window = Window::new(
        WINDOW_NAME,
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        WindowOptions {
            scale: Scale::X4,
            ..WindowOptions::default()
        },
    )
    .unwrap();

    let frame_target_duration = Duration::from_micros(FRAMERATE);

    // Start loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let frame_start_time = Instant::now();

        // Run the CPU for one frame's worth of cycles
        let mut cycles_this_frame = 0;
        while cycles_this_frame < APPROX_CYCLES_PER_FRAME {
            let cycles = cpu.step();
            cycles_this_frame += cycles;
        }

        // Update the window with the PPU's buffer
        if let Err(e) = window.update_with_buffer(&cpu.bus.get_frame(), SCREEN_WIDTH, SCREEN_HEIGHT)
        {
            error!("Failed to draw buffer: {}", e)
        }

        // Manually limit the speed
        // If the frame is drawed faster than the target framerate, wait for the difference
        let elapsed = frame_start_time.elapsed();
        if elapsed < frame_target_duration {
            std::thread::sleep(frame_target_duration - elapsed);
        }
    }
}
