#[macro_use]
mod text_font;
mod emulator;
mod file_system;
mod window;

use emulator::LoadedEmulator;
use std::process::Command;
use window::Win;

const EMULATOR_CHIP_8_NAME: &str = "emulator_chip8";

const EXIT_TEXT: &str = "Exit";
const EMULATORS: [&str; 1] = [EMULATOR_CHIP_8_NAME];

fn start_emulator_process(path: &str, rom: &str) {
    Command::new(path)
        .arg(rom)
        .output()
        .expect("Could not run the chip 8 emulator");
}

fn main() -> Result<(), String> {
    let mut window = Win::new();
    'running: loop {
        let loaded_emulator = window.main_window();
        match loaded_emulator {
            LoadedEmulator::Yes(emulator) => {
                if EMULATORS.contains(&emulator.name()) {
                    let emulator_path = file_system::append_to_exec_dir(emulator.name());

                    start_emulator_process(
                        emulator_path.to_str().unwrap(),
                        emulator.loaded_rom().unwrap().name(),
                    );
                } else if emulator.name() == EXIT_TEXT {
                    break 'running;
                }
            }
            LoadedEmulator::No => break 'running,
        }
    }

    Ok(())
}
