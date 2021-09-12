extern crate emulator_chip8 as chip8;

const EXIT_TEXT: &str = "Exit";
const EMULATORS: [&str; 2] = [chip8::emulator_driver::NAME, EXIT_TEXT];
// const COUNT: usize = EMULATORS.len();

mod text_font {
    use sdl2::{
        pixels::Color,
        rect::Rect,
        render::{Texture, TextureCreator, TextureQuery},
        ttf::Font,
        video::WindowContext,
    };

    use crate::window::{HEIGHT, WIDTH};

    pub struct Text<'a> {
        pub texture: Texture<'a>,
        pub target: Rect,
        pub text: &'a str,
    }

    // handle the annoying Rect i32
    macro_rules! rect(
        ($x:expr, $y:expr, $w:expr, $h:expr) => (
            Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
        )
    );

    impl<'a> Text<'_> {
        pub fn new(
            text: &'a str,
            font: &Font,
            texture_creator: &'a TextureCreator<WindowContext>,
            x: u32,
            y: u32,
        ) -> Result<Text<'a>, String> {
            let surface = font
                .render(text)
                .blended(Color::WHITE)
                .map_err(|e| e.to_string())?;

            let texture = texture_creator
                .create_texture_from_surface(&surface)
                .map_err(|e| e.to_string())?;

            let TextureQuery { width, height, .. } = texture.query();

            // If the example text is too big for the screen, downscale it (and center irregardless)
            Ok(Text {
                texture,
                target: get_centered_rect(width, height, WIDTH, HEIGHT, x, y),
                text,
            })
        }
    }

    // Scale fonts to a reasonable size when they're too big (though they might look less smooth)
    fn get_centered_rect(
        rect_width: u32,
        rect_height: u32,
        cons_width: u32,
        cons_height: u32,
        x: u32,
        y: u32,
    ) -> Rect {
        let wr = rect_width as f32 / cons_width as f32;
        let hr = rect_height as f32 / cons_height as f32;

        let (w, h) = if wr > 1f32 || hr > 1f32 {
            println!("Scaling down! The text will look worse!");
            if wr > hr {
                let h = (rect_height as f32 / wr) as u32;
                (cons_width, h)
            } else {
                let w = (rect_width as f32 / hr) as u32;
                (w, cons_height)
            }
        } else {
            (rect_width, rect_height)
        };

        rect!(x, y, w, h)
    }
}

mod emulator {
    pub struct EmulatorAndRom {
        pub emulator: String,
        pub rom: String,
    }

    impl EmulatorAndRom {
        pub fn new(emulator: String, rom: String) -> EmulatorAndRom {
            EmulatorAndRom { emulator, rom }
        }
    }
}

mod file_system {
    use native_dialog::FileDialog;

    pub fn choose_file() -> Result<String, String> {
        let home_dir = home::home_dir().unwrap();

        let path = FileDialog::new()
            .set_location(home_dir.to_str().unwrap())
            .show_open_single_file()
            .unwrap();

        let path = match path {
            Some(it) => it,
            None => return Err("Something went wrong choosing a rom file!".to_string()),
        };
        Ok(path.to_str().unwrap().to_string().replace("file://", ""))
    }
}

mod window {
    use crate::{emulator::EmulatorAndRom, file_system, text_font::Text, EMULATORS, EXIT_TEXT};
    use sdl2::{event::Event, keyboard::Keycode, rect::Point, ttf::Font};
    use std::path::Path;

    extern crate sdl2;

    pub const WIDTH: u32 = 600;
    pub const HEIGHT: u32 = 600;

    macro_rules! draw_text_list {
        ($list:expr, $canvas:expr) => {
            for text in $list.iter() {
                $canvas.copy(&text.texture, None, Some(text.target))?;
            }
        };
    }

    pub fn main_window() -> Result<Option<EmulatorAndRom>, String> {
        let sdl_context = sdl2::init()?;

        let window = sdl_context
            .video()?
            .window("Lambda Blue", WIDTH, HEIGHT)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let mut events = sdl_context.event_pump()?;

        let mut canvas = window
            .into_canvas()
            .present_vsync()
            .accelerated()
            .build()
            .map_err(|e| e.to_string())?;

        let texture_creator = canvas.texture_creator();
        // Load a font
        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
        let path: &Path = Path::new("lambda_blue/fonts/open-sans/OpenSans-ExtraBold.ttf");
        let mut font: Font = ttf_context.load_font(path, 56)?;
        font.set_style(sdl2::ttf::FontStyle::BOLD);

        let emulator_names: Vec<Text> = EMULATORS
            .iter()
            .enumerate()
            .map(|(index, name)| {
                Text::new(*name, &font, &texture_creator, 0, (index * 80) as u32).unwrap()
            })
            .collect();

        let no_roms_text_vec: Vec<Text> = vec!["Load Rom", "Back"]
            .iter()
            .enumerate()
            .map(|(index, name)| {
                Text::new(*name, &font, &texture_creator, 0, (index * 80) as u32).unwrap()
            })
            .collect();

        let rom_names: Option<Vec<Text>> = None;

        let mut emulator = String::new();
        let mut rom = String::new();

        'running: loop {
            for event in events.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    Event::MouseButtonUp { x, y, .. } => {
                        let mouse_pos = Point::new(x, y);

                        if emulator.is_empty() {
                            for name in emulator_names.iter() {
                                if name.target.contains_point(mouse_pos) {
                                    println!("{}", name.text);
                                    if name.text == EXIT_TEXT {
                                        break 'running;
                                    }
                                    emulator = name.text.to_string();
                                }
                            }
                        } else {
                            for name in no_roms_text_vec.iter() {
                                if name.target.contains_point(mouse_pos) {
                                    println!("{} {:?}", name.text, home::home_dir());

                                    if name.text == "Back" {
                                        emulator = String::new();
                                        break;
                                    }

                                    if let Ok(file) = file_system::choose_file() {
                                        rom = file;
                                        break 'running;
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            canvas.clear();

            if emulator.is_empty() {
                draw_text_list!(emulator_names, canvas);
            } else if let Some(names) = &rom_names {
                draw_text_list!(names, canvas);
            } else {
                draw_text_list!(no_roms_text_vec, canvas);
            }

            canvas.present();
        }

        if !emulator.is_empty() && !rom.is_empty() {
            return Ok(Some(EmulatorAndRom::new(emulator, rom)));
        }
        Ok(None)
    }
}

fn main() -> Result<(), String> {
    'running: loop {
        let s = window::main_window();
        if let Ok(emulator_and_rom) = s {
            match emulator_and_rom {
                Some(emulator::EmulatorAndRom { emulator, rom }) => match emulator.as_str() {
                    chip8::emulator_driver::NAME => {
                        chip8::emulator_driver::start(Some(rom.as_str()))?
                    }
                    _ => {}
                },
                _ => {
                    println!("Exiting!");
                    break 'running;
                }
            }
        }
    }

    Ok(())
}
