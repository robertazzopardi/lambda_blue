use std::process::Command;
use window::Win;

pub const EMULATOR_CHIP_8_NAME: &str = "CHIP 8";

const EXIT_TEXT: &str = "Exit";
const EMULATORS: [&str; 2] = [EMULATOR_CHIP_8_NAME, EXIT_TEXT];
// const COUNT: usize = EMULATORS.len();

#[macro_use]
mod text_font {
    use crate::window::get_centered_rect;
    use sdl2::{
        pixels::Color,
        rect::Rect,
        render::{Texture, TextureCreator, TextureQuery},
        ttf::Font,
        video::WindowContext,
    };

    macro_rules! text_list {
        ($font:expr, $texture_creator:expr, $names:expr) => {
            $names
                .iter()
                .enumerate()
                .map(|(index, name)| {
                    Text::new(*name, &$font, &$texture_creator, 0, (index * 80) as u32).unwrap()
                })
                .collect()
        };
    }

    pub struct FontTexture<'a> {
        font: Font<'a, 'a>,
    }

    pub struct Text<'a> {
        texture: Texture<'a>,
        target: Rect,
        text: &'a str,
    }

    impl<'a> Text<'_> {
        /// Get a new text object
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
                target: get_centered_rect(width, height, x, y),
                text,
            })
        }

        /// Get a reference to the text's texture.
        pub fn texture(&self) -> &Texture<'_> {
            &self.texture
        }

        /// Get a reference to the text's target.
        pub fn target(&self) -> Rect {
            self.target
        }

        /// Get a reference to the text's text.
        pub fn text(&self) -> &str {
            self.text
        }
    }
}

mod emulator {
    use sdl2::filesystem;
    use serde::{Deserialize, Serialize};
    use std::{fs, thread};

    use crate::file_system;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Rom {
        name: String,
    }

    impl Rom {
        pub fn new(name: String) -> Self {
            Self { name }
        }

        /// Get a reference to the rom's name.
        pub fn name(&self) -> &str {
            self.name.as_str()
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Emulators {
        pub emulators: Vec<Emulator>,
    }

    impl Drop for Emulators {
        fn drop(&mut self) {
            self.save_emulators()
        }
    }

    impl Emulators {
        pub fn from_strings(place_holders: Vec<&str>) -> Self {
            Emulators {
                emulators: place_holders
                    .iter()
                    .map(|s| Emulator::new(s.to_string()))
                    .collect(),
            }
        }

        pub fn save_emulators(&mut self) {
            let json_string = serde_json::to_string(&self.emulators).unwrap();

            let save_path = file_system::append_to_exec_dir("emulators.txt");

            let save_thread = thread::spawn(|| {
                fs::write(save_path, json_string).expect("Could not save emulator configuration!");
            });

            save_thread.join().expect("Thread error")
        }

        pub fn load_emulators() -> Result<Emulators, String> {
            let read_emulators = fs::read_to_string("./emulators.txt");

            if let Ok(read_emulators) = read_emulators {
                if let Ok(emulators) = serde_json::from_str(&read_emulators) {
                    emulators
                } else {
                    Err("Could not deserialise emulators.txt".to_string())
                }
            } else {
                Err("Could not read emulator file".to_string())
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Emulator {
        pub name: String,
        roms: Vec<Rom>,
        pub loaded_rom: Option<Rom>,
    }

    impl Emulator {
        pub fn new(name: String) -> Self {
            Self {
                name,
                roms: Vec::new(),
                loaded_rom: None,
            }
        }
    }
}

mod file_system {
    use native_dialog::FileDialog;
    use std::{env, path::PathBuf};

    pub fn get_execution_dir() -> PathBuf {
        let mut path_buf = env::current_exe().unwrap();
        path_buf.pop();

        path_buf
    }

    pub fn append_to_exec_dir(to_append: &str) -> PathBuf {
        let mut new_path = get_execution_dir();
        new_path.push(to_append);

        new_path
    }

    pub fn choose_file_dialog() -> Result<String, String> {
        let file_dialog = FileDialog::new();
        let path = file_dialog
            .set_location(home::home_dir().unwrap().to_str().unwrap())
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
    extern crate sdl2;

    use crate::{
        emulator::{Emulator, Emulators, Rom},
        file_system,
        text_font::Text,
        EMULATORS, EXIT_TEXT,
    };
    use sdl2::{
        event::Event,
        keyboard::Keycode,
        rect::{Point, Rect},
        render::{Canvas, TextureCreator},
        ttf::{Font, Sdl2TtfContext},
        video::{Window, WindowContext},
        EventPump, TimerSubsystem,
    };

    pub const WIDTH: u32 = 600;
    pub const HEIGHT: u32 = 600;

    // handle the annoying Rect i32
    macro_rules! rect(
        ($x:expr, $y:expr, $w:expr, $h:expr) => (
            Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
        )
    );

    macro_rules! draw_text_list {
        ($list:expr, $canvas:expr) => {
            for text in $list.iter() {
                $canvas.copy(&text.texture(), None, Some(text.target()))?;
            }
        };
    }

    macro_rules! render_text_list {
        ( $list:expr,$canvas:expr) => {
            $canvas.clear();
            draw_text_list!($list, $canvas);
            $canvas.present();
        };
    }

    pub struct Win {
        texture_creator: TextureCreator<WindowContext>,
        ttf_context: Sdl2TtfContext,
        events: EventPump,
        timer: TimerSubsystem,
        canvas: Canvas<Window>,
        running: bool,
    }

    impl Win {
        pub fn new() -> Self {
            let sdl_context = sdl2::init().unwrap();
            let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).unwrap();
            let events = sdl_context.event_pump().unwrap();
            let timer = sdl_context.timer().unwrap();

            let window = sdl_context
                .video()
                .unwrap()
                .window("Lambda Blue", WIDTH, HEIGHT)
                .position_centered()
                .build()
                .map_err(|e| e.to_string())
                .unwrap();

            let canvas = window
                .into_canvas()
                .accelerated()
                .build()
                .map_err(|e| e.to_string())
                .unwrap();

            let texture_creator = canvas.texture_creator();

            Self {
                texture_creator,
                ttf_context,
                events,
                timer,
                canvas,
                running: true,
            }
        }

        pub fn main_window(&mut self) -> Result<Option<Emulator>, String> {
            // Load a font
            let font: Font = self
                .ttf_context
                .load_font("lambda_blue/fonts/open-sans/OpenSans-ExtraBold.ttf", 56)?;

            let loaded_emulators = Emulators::load_emulators();

            let loaded_emulators = match loaded_emulators {
                Ok(result) => result,
                Err(_) => Emulators::from_strings(vec!["Load Rom", "Back"]),
            };

            let emulator_names: Vec<Text> = text_list!(font, self.texture_creator, EMULATORS);

            let no_roms_text_vec: Vec<Text> = text_list!(
                font,
                self.texture_creator,
                loaded_emulators
                    .emulators
                    .iter()
                    .map(|e| e.name.as_str())
                    .collect::<Vec<&str>>()
            );

            let rom_names: Option<Vec<Text>> = None;

            let mut emulator = String::new();
            let mut rom = String::new();

            render_text_list!(emulator_names, self.canvas);

            while self.running {
                let start = self.timer.performance_counter();

                for event in self.events.poll_iter() {
                    match event {
                        Event::Quit { .. }
                        | Event::KeyDown {
                            keycode: Some(Keycode::Escape),
                            ..
                        } => self.running = false,
                        Event::MouseButtonUp { x, y, .. } => {
                            let mouse_pos = Point::new(x, y);

                            if emulator.is_empty() {
                                for name in emulator_names.iter() {
                                    if name.target().contains_point(mouse_pos) {
                                        // println!("{}", name.text());

                                        if name.text() == EXIT_TEXT {
                                            self.running = false;
                                        } else {
                                            emulator = name.text().to_string().clone();

                                            let mut text_to_render = &no_roms_text_vec;
                                            if let Some(names) = &rom_names {
                                                text_to_render = names;
                                            }
                                            render_text_list!(text_to_render, self.canvas);
                                        }
                                    }
                                }
                            } else {
                                for name in no_roms_text_vec.iter() {
                                    if name.target().contains_point(mouse_pos) {
                                        // println!("{} {:?}", name.text(), home::home_dir());

                                        if name.text() == "Back" {
                                            emulator = String::new();
                                            render_text_list!(emulator_names, self.canvas);
                                            break;
                                        }

                                        if let Ok(file_name) = file_system::choose_file_dialog() {
                                            rom = file_name;
                                            self.running = false;
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }

                let end = self.timer.performance_counter();
                let elapsed =
                    (end - start) as f32 / self.timer.performance_frequency() as f32 * 1000.;

                self.timer.delay(((1000. / 10.) - elapsed).floor() as u32);
            }

            if !emulator.is_empty() && !rom.is_empty() {
                let mut em = Emulator::new(emulator);
                em.loaded_rom = Some(Rom::new(rom));

                return Ok(Some(em));
            }

            Ok(None)
        }
    }

    // Scale fonts to a reasonable size when they're too big (though they might look less smooth)
    pub fn get_centered_rect(rect_width: u32, rect_height: u32, x: u32, y: u32) -> Rect {
        let wr = rect_width as f32 / WIDTH as f32;
        let hr = rect_height as f32 / HEIGHT as f32;

        let (w, h) = if wr > 1f32 || hr > 1f32 {
            println!("Scaling down! The text will look worse!");
            if wr > hr {
                let h = (rect_height as f32 / wr) as u32;
                (WIDTH, h)
            } else {
                let w = (rect_width as f32 / hr) as u32;
                (w, HEIGHT)
            }
        } else {
            (rect_width, rect_height)
        };

        rect!(x, y, w, h)
    }
}

fn start_emulator(path: &str, rom: &str) {
    Command::new(path)
        .arg(rom)
        .output()
        .expect("Could not run the chip 8 emulator");
}

fn main() -> Result<(), String> {
    // let file_utility = file_system::FileUtility::new();

    'running: loop {
        let loaded_emulator = Win::new().main_window().unwrap();
        match loaded_emulator {
            Some(emulator) => {
                if emulator.name == EMULATOR_CHIP_8_NAME {
                    // let emulator_path = executable_path.clone() + "/emulator_chip8";
                    let emulator_path = file_system::append_to_exec_dir("emulator_chip8");

                    start_emulator(
                        emulator_path.to_str().unwrap(),
                        emulator.loaded_rom.unwrap().name(),
                    );
                } else if emulator.name == EXIT_TEXT {
                    break 'running;
                }
            }
            None => break 'running,
        }
    }

    Ok(())
}
