use emulator::LoadedEmulator;
use std::process::Command;
use window::Win;

const EMULATOR_CHIP_8_NAME: &str = "emulator_chip8";

const EXIT_TEXT: &str = "Exit";
const EMULATORS: [&str; 1] = [EMULATOR_CHIP_8_NAME];
// const COUNT: usize = EMULATORS.len();

#[macro_use]
mod text_font {
    use crate::window::get_centered_rect;
    use sdl2::{
        pixels::Color,
        rect::Rect,
        render::{Texture, TextureCreator, TextureQuery},
        surface::Surface,
        ttf::Font,
        video::WindowContext,
    };

    // macro_rules! generate_text_list {
    //     ($font:expr, $texture_creator:expr, $names:expr) => {
    //         $names
    //             .iter()
    //             .enumerate()
    //             .map(|(index, name)| {
    //                 Text::new(name, &$font, &$texture_creator, 0, (index * 80) as u32).unwrap()
    //             })
    //             .collect()
    //     };
    // }

    pub fn generate_text_list<'a>(
        font: &Font,
        texture_creator: &'a TextureCreator<WindowContext>,
        names: &[&'a str],
    ) -> Vec<Text<'a>> {
        names
            .iter()
            .enumerate()
            .map(|(index, name)| {
                Text::new(name, font, &texture_creator, 0, (index * 80) as u32).unwrap()
            })
            .collect()
    }

    pub fn get_text_surface<'a>(text: &'a str, font: &Font) -> Surface<'a> {
        font.render(text)
            .blended(Color::WHITE)
            .map_err(|e| e.to_string())
            .unwrap()
    }

    pub fn generate_texture<'a>(
        surface: &Surface,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> Texture<'a> {
        texture_creator
            .create_texture_from_surface(surface)
            .map_err(|e| e.to_string())
            .unwrap()
    }

    pub fn create_target_for_texture(texture: &Texture, x: u32, y: u32) -> Rect {
        let TextureQuery { width, height, .. } = texture.query();
        get_centered_rect(width, height, x, y)
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
            let surface = get_text_surface(text, font);

            let texture = generate_texture(&surface, texture_creator);

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
    use crate::{file_system, EXIT_TEXT};
    use serde::{Deserialize, Serialize};
    use std::{fs, thread};

    pub enum LoadedEmulator {
        YES(Emulator),
        NO,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Emulators {
        pub emulators: Vec<Emulator>,
    }

    // impl Drop for Emulators {
    //     fn drop(&mut self) {
    //         self.save_emulators()
    //     }
    // }

    impl Emulators {
        pub fn from_strings(place_holders: Vec<&str>) -> Self {
            Emulators {
                emulators: place_holders
                    .iter()
                    .map(|s| Emulator::new(s.to_string()))
                    .collect(),
            }
        }

        // pub fn save_emulators(&self) {
        //     #[cfg(debug_assertions)]
        //     let json_string = serde_json::to_string_pretty(&self.emulators).unwrap();

        //     #[cfg(not(debug_assertions))]
        //     let json_string = serde_json::to_string(&self.emulators).unwrap();

        //     let save_path = file_system::append_to_exec_dir("emulators.json");

        //     let save_thread = thread::spawn(|| {
        //         fs::write(save_path, json_string).expect("Could not save emulator configuration!");
        //     });

        //     save_thread.join().expect("Thread error")
        // }

        // pub fn load_emulators() -> Result<Emulators, String> {
        //     let load_path = file_system::append_to_exec_dir("emulators.json");

        //     let read_emulators = fs::read_to_string(load_path);

        //     if let Ok(read_emulators) = read_emulators {
        //         if read_emulators.is_empty() {
        //             return Err("Emulator file empty".to_string());
        //         }

        //         let data: Vec<Emulator> = serde_json::from_str(&read_emulators).unwrap();

        //         Ok(Emulators { emulators: data })
        //     } else {
        //         Err("Could not read emulator file".to_string())
        //     }
        // }

        // pub fn load_emulators_roms_clone(&mut self, index: usize) -> Vec<Rom> {
        //     self.emulators[index].roms.clone()
        // }

        pub fn index_of_emulator(&self, emulator: &String) -> usize {
            self.emulators
                .iter()
                .position(|emu| emu.name == *emulator)
                .unwrap()
        }

        pub fn load_roms_for_emulator(&self, index: usize) -> Vec<&str> {
            // self.emulators[index].roms
            self.emulators[index]
                .roms()
                .iter()
                .map(|r| r.name())
                .collect::<Vec<&str>>()
        }

        pub fn load_rom_into_emulator(&mut self, rom: Rom, index: usize) {
            self.emulators[index].load_rom(rom)
        }

        pub fn get_emulator_clone(&self, index: usize) -> Emulator {
            self.emulators[index].clone()
        }

        /// Get a reference to the emulators's emulators.
        pub fn emulators(&self) -> &[Emulator] {
            self.emulators.as_slice()
        }

        pub fn emulator_names_to_string(&self) -> Vec<&str> {
            let mut main_page_texts = self
                .emulators
                .iter()
                .map(|emus| emus.name())
                .collect::<Vec<&str>>();

            main_page_texts.push(EXIT_TEXT);

            main_page_texts
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Emulator {
        name: String,
        roms: Vec<Rom>,

        #[serde(skip_serializing, skip_deserializing)]
        loaded_rom: Option<Rom>,
    }

    impl Emulator {
        pub fn new(name: String) -> Self {
            Self {
                name,
                roms: Vec::new(),
                loaded_rom: None,
            }
        }

        pub fn load_rom(&mut self, rom: Rom) {
            if !self.roms.contains(&rom) {
                self.roms.push(rom.clone());
            }

            self.loaded_rom = Some(rom)
        }

        /// Get a reference to the emulator's loaded rom.
        pub const fn loaded_rom(&self) -> Option<&Rom> {
            self.loaded_rom.as_ref()
        }

        /// Get a reference to the emulator's name.
        pub fn name(&self) -> &str {
            self.name.as_str()
        }

        /// Get a reference to the emulator's roms.
        pub fn roms(&self) -> &[Rom] {
            self.roms.as_slice()
        }

        pub fn roms_as_str(&self) -> Vec<&str> {
            self.roms
                .iter()
                .map(|r| r.name.as_str())
                .collect::<Vec<&str>>()
        }
    }
}

mod file_system {
    use native_dialog::FileDialog;
    use std::{env, fs, path::PathBuf, thread};

    use crate::emulator::{Emulator, Emulators};

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

    pub fn save_emulators(emulators: &[Emulator]) {
        #[cfg(debug_assertions)]
        let json_string = serde_json::to_string_pretty(emulators).unwrap();

        #[cfg(not(debug_assertions))]
        let json_string = serde_json::to_string(emulators).unwrap();

        let save_path = append_to_exec_dir("emulators.json");

        let save_thread = thread::spawn(|| {
            fs::write(save_path, json_string).expect("Could not save emulator configuration!");
        });

        save_thread.join().expect("Thread error")
    }

    pub fn load_emulators() -> Result<Emulators, String> {
        let load_path = append_to_exec_dir("emulators.json");

        let read_emulators = fs::read_to_string(load_path);

        if let Ok(read_emulators) = read_emulators {
            if read_emulators.is_empty() {
                return Err("Emulator file empty".to_string());
            }

            let data: Vec<Emulator> = serde_json::from_str(&read_emulators).unwrap();

            Ok(Emulators { emulators: data })
        } else {
            Err("Could not read emulator file".to_string())
        }
    }
}

mod window {
    extern crate sdl2;

    use crate::{
        emulator::{Emulators, LoadedEmulator, Rom},
        file_system,
        text_font::{self, Text},
        EMULATORS, EXIT_TEXT,
    };
    use sdl2::{
        event::Event,
        keyboard::Keycode,
        rect::{Point, Rect},
        render::{Canvas, TextureCreator},
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

    macro_rules! render_text_list {
        ($list:expr, $canvas:expr) => {
            $canvas.clear();
            for text in $list.iter() {
                $canvas
                    .copy(&text.texture(), None, Some(text.target()))
                    .unwrap();
            }
            $canvas.present();
        };
    }

    pub struct Win {
        texture_creator: TextureCreator<WindowContext>,
        events: EventPump,
        timer: TimerSubsystem,
        canvas: Canvas<Window>,
    }

    impl Win {
        pub fn new() -> Self {
            let sdl_context = sdl2::init().unwrap();
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
                events,
                timer,
                canvas,
            }
        }

        // pub fn render_texts(&mut self, texts_to_draw: Vec<Text>) {
        //     self.canvas.clear();

        //     for text in texts_to_draw.iter() {
        //         self.canvas
        //             .copy(&text.texture(), None, Some(text.target()))
        //             .unwrap();
        //     }

        //     self.canvas.present();
        // }

        // pub fn present_emulators(&mut self, font: &Font, emulators: &Emulators) {
        //     self.canvas.clear();

        //     for (index, emulator) in emulators.emulators().iter().enumerate() {
        //         let surface = text_font::get_text_surface(emulator.name(), font);
        //         let texture = &text_font::generate_texture(&surface, &self.texture_creator);
        //         let target = text_font::create_target_for_texture(texture, 0, (index * 80) as u32);

        //         self.canvas.copy(texture, None, target).unwrap();
        //     }

        //     self.canvas.present();
        // }

        pub fn main_window(&mut self) -> LoadedEmulator {
            let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).unwrap();
            // Load a font
            let font = ttf_context
                .load_font("lambda_blue/fonts/open-sans/OpenSans-ExtraBold.ttf", 56)
                .unwrap();

            let loaded_emulators_for_roms = match file_system::load_emulators() {
                Ok(result) => result,
                Err(_) => Emulators::from_strings(EMULATORS.to_vec()),
            };

            let mut texts_to_draw: Vec<Text> = text_font::generate_text_list(
                &font,
                &self.texture_creator,
                &loaded_emulators_for_roms
                    .emulator_names_to_string()
                    .as_slice(),
            );

            render_text_list!(texts_to_draw, self.canvas);

            'running: loop {
                let start = self.timer.performance_counter();

                for event in self.events.poll_iter() {
                    match event {
                        Event::Quit { .. }
                        | Event::KeyDown {
                            keycode: Some(Keycode::Escape),
                            ..
                        } => break 'running,
                        Event::MouseButtonUp { x, y, .. } => {
                            let mouse_pos = Point::new(x, y);

                            for i in 0..texts_to_draw.len() {
                                if texts_to_draw[i].target().contains_point(mouse_pos) {
                                    match texts_to_draw[i].text() {
                                        EXIT_TEXT => {
                                            return LoadedEmulator::NO;
                                        }
                                        "Back" => {
                                            texts_to_draw = text_font::generate_text_list(
                                                &font,
                                                &self.texture_creator,
                                                &loaded_emulators_for_roms
                                                    .emulator_names_to_string()
                                                    .as_slice(),
                                            );

                                            render_text_list!(texts_to_draw, self.canvas);
                                        }
                                        "Load Rom" => {
                                            let choose_file_dialog =
                                                file_system::choose_file_dialog();

                                            if let Ok(rom_file_name) = choose_file_dialog {
                                                let mut to_save = loaded_emulators_for_roms.clone();

                                                to_save.load_rom_into_emulator(
                                                    Rom::new(rom_file_name),
                                                    i,
                                                );

                                                file_system::save_emulators(to_save.emulators());

                                                return LoadedEmulator::YES(
                                                    to_save.get_emulator_clone(i),
                                                );
                                            }
                                        }
                                        _ => {
                                            if loaded_emulators_for_roms
                                                .load_roms_for_emulator(i)
                                                .is_empty()
                                            {
                                                texts_to_draw = text_font::generate_text_list(
                                                    &font,
                                                    &self.texture_creator,
                                                    &vec!["Load Rom", "Back"],
                                                );
                                            } else if loaded_emulators_for_roms
                                                .get_emulator_clone(i)
                                                .roms_as_str()
                                                .contains(&texts_to_draw[i].text())
                                            {
                                                let mut ret_emulator =
                                                    loaded_emulators_for_roms.get_emulator_clone(i);

                                                ret_emulator.load_rom(Rom::new(
                                                    texts_to_draw[i].text().to_string(),
                                                ));

                                                return LoadedEmulator::YES(ret_emulator);
                                            } else {
                                                let mut z = loaded_emulators_for_roms
                                                    .load_roms_for_emulator(i);

                                                z.append(&mut vec!["Load Rom", "Back"]);

                                                texts_to_draw = text_font::generate_text_list(
                                                    &font,
                                                    &self.texture_creator,
                                                    &z,
                                                );
                                            }

                                            render_text_list!(texts_to_draw, self.canvas);
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

            LoadedEmulator::NO
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

fn start_emulator_process(path: &str, rom: &str) {
    Command::new(path)
        .arg(rom)
        .output()
        .expect("Could not run the chip 8 emulator");
}

fn main() -> Result<(), String> {
    'running: loop {
        let loaded_emulator = Win::new().main_window();
        match loaded_emulator {
            LoadedEmulator::YES(emulator) => {
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
            LoadedEmulator::NO => break 'running,
        }
    }

    Ok(())
}
