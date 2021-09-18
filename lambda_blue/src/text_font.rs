use crate::window::get_centered_rect;
use sdl2::{
    pixels::Color,
    rect::Rect,
    render::{Texture, TextureCreator, TextureQuery},
    surface::Surface,
    ttf::Font,
    video::WindowContext,
};

pub fn generate_text_list<'a>(
    font: &Font,
    texture_creator: &'a TextureCreator<WindowContext>,
    names: &[&'a str],
) -> Vec<Text<'a>> {
    names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            Text::new(name, font, texture_creator, 0, (index * 80) as u32).unwrap()
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
