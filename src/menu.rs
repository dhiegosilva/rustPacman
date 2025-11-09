// Main menu for maze selection

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
        self.draw_text(canvas, "PAC-MAN", center_x, start_y - 40, 3, Color::RGB(255, 255, 0))?;
        
        // Menu options
        let options = ["Maze 1: Classic", "Maze 2: Simple"];
        for (i, option) in options.iter().enumerate() {
            let color = if i == self.selected {
                Color::RGB(255, 255, 0)
            } else {
                Color::RGB(255, 255, 255)
            };
            self.draw_text(canvas, option, center_x, start_y + (i as i32 * 40), 2, color)?;
        }

        self.draw_text(canvas, "Arrow Keys: Select", center_x, start_y + 100, 1, Color::RGB(150, 150, 150))?;
        self.draw_text(canvas, "Enter: Start", center_x, start_y + 120, 1, Color::RGB(150, 150, 150))?;

        canvas.present();
        Ok(())
    }

    fn draw_text(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, 
                 text: &str, center_x: i32, y: i32, scale: i32, color: Color) -> Result<(), String> {
        // Simple text rendering using rectangles
        let char_w = 5;
        let _char_h = 7;
        let spacing = 1;
        let pixel_size = scale;
        
        // Simple 5x7 font for basic characters
        let font: std::collections::HashMap<char, [[bool; 5]; 7]> = [
            ('A', [[false, true, true, true, false], [true, false, false, false, true], 
                   [true, true, true, true, true], [true, false, false, false, true], 
                   [true, false, false, false, true], [true, false, false, false, true], 
                   [true, false, false, false, true]]),
            ('C', [[false, true, true, true, false], [true, false, false, false, true], 
                   [true, false, false, false, false], [true, false, false, false, false], 
                   [true, false, false, false, false], [true, false, false, false, true], 
                   [false, true, true, true, false]]),
            ('M', [[true, false, false, false, true], [true, true, false, true, true], 
                   [true, false, true, false, true], [true, false, false, false, true], 
                   [true, false, false, false, true], [true, false, false, false, true], 
                   [true, false, false, false, true]]),
            ('P', [[true, true, true, true, false], [true, false, false, false, true], 
                   [true, false, false, false, true], [true, true, true, true, false], 
                   [true, false, false, false, false], [true, false, false, false, false], 
                   [true, false, false, false, false]]),
            ('-', [[false, false, false, false, false], [false, false, false, false, false], 
                   [false, false, false, false, false], [true, true, true, true, true], 
                   [false, false, false, false, false], [false, false, false, false, false], 
                   [false, false, false, false, false]]),
            ('1', [[false, false, true, false, false], [false, true, true, false, false], 
                   [false, false, true, false, false], [false, false, true, false, false], 
                   [false, false, true, false, false], [false, false, true, false, false], 
                   [false, true, true, true, false]]),
            ('2', [[false, true, true, true, false], [true, false, false, false, true], 
                   [false, false, false, false, true], [false, false, true, true, false], 
                   [false, true, false, false, false], [true, false, false, false, false], 
                   [true, true, true, true, true]]),
            ('S', [[false, true, true, true, false], [true, false, false, false, true], 
                   [true, false, false, false, false], [false, true, true, true, false], 
                   [false, false, false, false, true], [true, false, false, false, true], 
                   [false, true, true, true, false]]),
            ('I', [[true, true, true, true, true], [false, false, true, false, false], 
                   [false, false, true, false, false], [false, false, true, false, false], 
                   [false, false, true, false, false], [false, false, true, false, false], 
                   [true, true, true, true, true]]),
            ('L', [[true, false, false, false, false], [true, false, false, false, false], 
                   [true, false, false, false, false], [true, false, false, false, false], 
                   [true, false, false, false, false], [true, false, false, false, false], 
                   [true, true, true, true, true]]),
            ('E', [[true, true, true, true, true], [true, false, false, false, false], 
                   [true, false, false, false, false], [true, true, true, true, false], 
                   [true, false, false, false, false], [true, false, false, false, false], 
                   [true, true, true, true, true]]),
            (' ', [[false, false, false, false, false]; 7]),
            (':', [[false, false, false, false, false], [false, false, true, false, false], 
                   [false, false, false, false, false], [false, false, false, false, false], 
                   [false, false, false, false, false], [false, false, true, false, false], 
                   [false, false, false, false, false]]),
        ].iter().cloned().collect();

        let text_width = text.chars().count() as i32 * (char_w + spacing) * pixel_size;
        let mut x_pos = center_x - text_width / 2;

        canvas.set_draw_color(color);
        for ch in text.chars() {
            let ch_upper = ch.to_uppercase().next().unwrap_or(' ');
            if let Some(glyph) = font.get(&ch_upper).or_else(|| font.get(&' ')) {
                for (row, &row_bits) in glyph.iter().enumerate() {
                    for col in 0..char_w {
                        if row_bits[col as usize] {
                            let _ = canvas.fill_rect(Rect::new(
                                x_pos + (col as i32 * pixel_size),
                                y + (row as i32 * pixel_size),
                                pixel_size as u32,
                                pixel_size as u32,
                            ));
                        }
                    }
                }
            }
            x_pos += (char_w + spacing) * pixel_size;
        }
        Ok(())
    }
}

