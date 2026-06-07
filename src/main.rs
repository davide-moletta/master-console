mod hw;
mod utils;

use clap::Parser;
use log::{error, warn};
use minifb::{Key, Scale, Window, WindowOptions};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use hw::cpu;

const WINDOW_NAME: &str = "Master Console - Game Boy";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the ROM file to load (.gb)
    #[arg(short, long)]
    path: PathBuf,
}

fn main() {
    let args = Args::parse();
    simple_logger::SimpleLogger::new().env().init().unwrap();

    let mut cpu = cpu::Cpu::new(args.path);

    // Generate window for rendering
    let mut window = Window::new(
        WINDOW_NAME,
        hw::SCREEN_WIDTH,
        hw::SCREEN_HEIGHT,
        WindowOptions {
            scale: Scale::X4,
            ..WindowOptions::default()
        },
    )
    .unwrap();

    let frame_target_duration = Duration::from_micros(hw::FRAMERATE);

    // Start loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        cpu.unset_buttons();
        let frame_start_time = Instant::now();

        // Update Joypad state based on keyboard
        window.get_keys().iter().for_each(|key| {
            if let Err(e) = cpu.set_button(*key) {
                warn!("Mapping not set: {}", e);
            };
        });

        // Run the CPU for one frame's worth of cycles
        let mut cycles_this_frame = 0;
        while cycles_this_frame < hw::APPROX_CYCLES_PER_FRAME {
            let cycles = cpu.step();
            cycles_this_frame += cycles;
        }

        // Update the window with the PPU's buffer
        if let Err(e) =
            window.update_with_buffer(&cpu.get_frame(), hw::SCREEN_WIDTH, hw::SCREEN_HEIGHT)
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
