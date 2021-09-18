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
    ttf::Sdl2TtfContext,
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
    ttf_context: Sdl2TtfContext,
}

impl Win {
    pub fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let events = sdl_context.event_pump().unwrap();
        let timer = sdl_context.timer().unwrap();
        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).unwrap();

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
            ttf_context,
        }
    }

    pub fn main_window(&mut self) -> LoadedEmulator {
        // Load a font
        let font = self
            .ttf_context
            .load_font("lambda_blue/fonts/open-sans/OpenSans-ExtraBold.ttf", 56)
            .unwrap();

        let loaded_emulators_for_roms = match file_system::load_emulators() {
            Ok(result) => result,
            Err(_) => Emulators::from_strings(EMULATORS.to_vec()),
        };

        let mut texts_to_draw: Vec<Text> = text_font::generate_text_list(
            &font,
            &self.texture_creator,
            loaded_emulators_for_roms
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
                                        return LoadedEmulator::No;
                                    }
                                    "Back" => {
                                        texts_to_draw = text_font::generate_text_list(
                                            &font,
                                            &self.texture_creator,
                                            loaded_emulators_for_roms
                                                .emulator_names_to_string()
                                                .as_slice(),
                                        );

                                        render_text_list!(texts_to_draw, self.canvas);
                                    }
                                    "Load Rom" => {
                                        let choose_file_dialog = file_system::choose_file_dialog();

                                        if let Ok(rom_file_name) = choose_file_dialog {
                                            let mut to_save = loaded_emulators_for_roms.clone();

                                            to_save
                                                .load_rom_into_emulator(Rom::new(rom_file_name), i);

                                            file_system::save_emulators(to_save.emulators());

                                            return LoadedEmulator::Yes(
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
                                                &["Load Rom", "Back"],
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

                                            return LoadedEmulator::Yes(ret_emulator);
                                        } else {
                                            let mut z =
                                                loaded_emulators_for_roms.load_roms_for_emulator(i);

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
            let elapsed = (end - start) as f32 / self.timer.performance_frequency() as f32 * 1000.;

            self.timer.delay(((1000. / 10.) - elapsed).floor() as u32);
        }

        LoadedEmulator::No
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
