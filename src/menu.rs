//! Main menu for maze selection

use sdl2::pixels::Color;
use sdl2::rect::Rect;

pub enum MenuAction {
    None,
    SelectMaze(usize),
}

pub struct Menu {
    pub selected: usize,
}

impl Menu {
    pub fn new() -> Self {
        Self { selected: 0 }
    }

    pub fn process_input(&mut self, _dx: i32, dy: i32) -> MenuAction {
        if dy < 0 && self.selected > 0 {
            self.selected -= 1;
        } else if dy > 0 && self.selected < 1 {
            self.selected += 1;
        }
        MenuAction::None
    }

    pub fn select(&self) -> MenuAction {
        MenuAction::SelectMaze(self.selected)
    }

    pub fn draw(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) -> Result<(), String> {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        let (ww, wh) = canvas.window().size();
        let center_x = ww as i32 / 2;
        let start_y = wh as i32 / 2 - 60;

        // Title
        self.draw_text_simple(canvas, "PAC-MAN", center_x, start_y - 40, 3, Color::RGB(255, 255, 0))?;
        
        // Menu options
        let options = ["Maze 1: Classic", "Maze 2: Simple"];
        for (i, option) in options.iter().enumerate() {
            let color = if i == self.selected {
                Color::RGB(255, 255, 0)
            } else {
                Color::RGB(255, 255, 255)
            };
            self.draw_text_simple(canvas, option, center_x, start_y + (i as i32 * 40), 2, color)?;
        }

        self.draw_text_simple(canvas, "Arrow Keys: Select", center_x, start_y + 100, 1, Color::RGB(150, 150, 150))?;
        self.draw_text_simple(canvas, "Enter: Start", center_x, start_y + 120, 1, Color::RGB(150, 150, 150))?;

        canvas.present();
        Ok(())
    }

    /// Simple text rendering using a minimal bitmap font
    /// This is much simpler than the previous implementation - just renders ASCII characters
    fn draw_text_simple(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, 
                       text: &str, center_x: i32, y: i32, scale: i32, color: Color) -> Result<(), String> {
        // Use a simple 5x7 bitmap font stored as a string pattern
        // Each character is 5 bits wide, stored as 7 rows
        let font_data: std::collections::HashMap<char, [u8; 7]> = [
            ('A', [0x0E, 0x11, 0x1F, 0x11, 0x11, 0x11, 0x11]),
            ('B', [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E]),
            ('C', [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E]),
            ('E', [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F]),
            ('I', [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x1F]),
            ('K', [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11]),
            ('L', [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F]),
            ('M', [0x11, 0x1B, 0x15, 0x11, 0x11, 0x11, 0x11]),
            ('N', [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11]),
            ('O', [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E]),
            ('P', [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10]),
            ('R', [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11]),
            ('S', [0x0E, 0x11, 0x10, 0x0E, 0x01, 0x11, 0x0E]),
            ('T', [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04]),
            ('U', [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E]),
            ('W', [0x11, 0x11, 0x11, 0x11, 0x15, 0x1B, 0x11]),
            ('Y', [0x11, 0x11, 0x0A, 0x04, 0x04, 0x04, 0x04]),
            ('Z', [0x1F, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1F]),
            ('1', [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E]),
            ('2', [0x0E, 0x11, 0x01, 0x06, 0x08, 0x10, 0x1F]),
            ('-', [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00]),
            (' ', [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            (':', [0x00, 0x04, 0x00, 0x00, 0x00, 0x04, 0x00]),
        ].iter().cloned().collect();

        let char_width = 5;
        let spacing = 1;
        let text_width = text.chars().count() as i32 * (char_width + spacing) * scale;
        let mut x_pos = center_x - text_width / 2;

        canvas.set_draw_color(color);
        for ch in text.chars() {
            let ch_upper = ch.to_uppercase().next().unwrap_or(' ');
            if let Some(&glyph) = font_data.get(&ch_upper).or_else(|| font_data.get(&' ')) {
                for (row_idx, &row_data) in glyph.iter().enumerate() {
                    for col_idx in 0..char_width {
                        if (row_data >> (4 - col_idx)) & 1 != 0 {
                            let _ = canvas.fill_rect(Rect::new(
                                x_pos + (col_idx as i32 * scale),
                                y + (row_idx as i32 * scale),
                                scale as u32,
                                scale as u32,
                            ));
                        }
                    }
                }
            }
            x_pos += (char_width + spacing) * scale;
        }
        Ok(())
    }
}
