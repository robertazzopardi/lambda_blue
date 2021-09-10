extern crate emulator_chip8 as chip8;

fn main() -> Result<(), String> {
    chip8::emulator_driver::start()
}
