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

fn start_emulator_process(file_to_run: &str, rom: &str) {
    Command::new(file_to_run)
        .arg(rom)
        .output()
        .expect("Could not run the chip 8 emulator");
}

fn main() -> Result<(), String> {
    // would like to define the window here, but there is an issue with the file dialog re-opeing
    // let mut window = Win::new();
    'running: loop {
        let loaded_emulator = Win::new().main_window();
        match loaded_emulator {
            LoadedEmulator::Yes(emulator) => {
                if EMULATORS.contains(&emulator.name()) {
                    start_emulator_process(emulator.get_path(), emulator.get_rom_path());
                } else if emulator.name() == EXIT_TEXT {
                    break 'running;
                }
            }
            LoadedEmulator::No => break 'running,
        }
    }

    Ok(())
}
