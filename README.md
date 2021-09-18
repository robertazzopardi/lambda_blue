# Lambda Blue Emulator Runner

A wrapper around all of the emulators I have written and hope to write

## Requirements

- SDL2
- SDL2_ttf

## Building and Running

Build first so that all of the emulators are built

```sh
cargo build --release
```

Run with the --bin flag to specify the main gui to run

```sh
cargo run --release --bin lambda_blue
```

## Available Emulators

- [Chip8](https://github.com/robertazzopardi/emulator_chip8)
