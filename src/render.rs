// Rendering code

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use crate::constants::{GRID_W, GRID_H, TILE, VIEW_W, VIEW_H, SCORE_AREA, MAZE};
use crate::maze::is_power_pellet;

pub struct RenderCache {
    pub scale: f32,
    pub ox: i32,
    pub oy: i32,
    pub game_start_y: i32,
    pub sw: i32,
    pub window_size_changed: bool,
}

impl RenderCache {
    pub fn new() -> Self {
        Self {
            scale: 1.0,
            ox: 0,
            oy: 0,
            game_start_y: 0,
            sw: 0,
            window_size_changed: true,
        }
    }

    pub fn update_cache(&mut self, ww: i32, wh: i32) {
        if self.window_size_changed {
            let total_view_h = VIEW_H + SCORE_AREA;
            let sx = ww as f32 / VIEW_W as f32;
            let sy = wh as f32 / total_view_h as f32;
            self.scale = sx.min(sy);
            self.sw = (VIEW_W as f32 * self.scale).floor() as i32;
            let total_sh = (total_view_h as f32 * self.scale).floor() as i32;
            self.ox = (ww - self.sw) / 2;
            self.oy = (wh - total_sh) / 2;
            let score_area_scaled = (SCORE_AREA as f32 * self.scale).floor() as i32;
            self.game_start_y = self.oy + score_area_scaled;
            self.window_size_changed = false;
        }
    }
}

pub fn draw_score(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    score: i32,
    ox: i32,
    oy: i32,
    sw: i32,
) {
    let score_str = format!("{:06}", score);
    let char_w = 4;
    let char_h = 6;
    let spacing = 1;
    let pixel_size = 2;
    
    // Simple block digits (0-9) - each digit is 4x6 pixels
    let digits: [[[bool; 4]; 6]; 10] = [
        // 0
        [[true, true, true, true], [true, false, false, true], [true, false, false, true], 
         [true, false, false, true], [true, false, false, true], [true, true, true, true]],
        // 1
        [[false, false, true, false], [false, true, true, false], [false, false, true, false], 
         [false, false, true, false], [false, false, true, false], [false, true, true, true]],
        // 2
        [[true, true, true, true], [false, false, false, true], [true, true, true, true], 
         [true, false, false, false], [true, false, false, false], [true, true, true, true]],
        // 3
        [[true, true, true, true], [false, false, false, true], [true, true, true, true], 
         [false, false, false, true], [false, false, false, true], [true, true, true, true]],
        // 4
        [[true, false, false, true], [true, false, false, true], [true, true, true, true], 
         [false, false, false, true], [false, false, false, true], [false, false, false, true]],
        // 5
        [[true, true, true, true], [true, false, false, false], [true, true, true, true], 
         [false, false, false, true], [false, false, false, true], [true, true, true, true]],
        // 6
        [[true, true, true, true], [true, false, false, false], [true, true, true, true], 
         [true, false, false, true], [true, false, false, true], [true, true, true, true]],
        // 7
        [[true, true, true, true], [false, false, false, true], [false, false, false, true], 
         [false, false, false, true], [false, false, false, true], [false, false, false, true]],
        // 8
        [[true, true, true, true], [true, false, false, true], [true, true, true, true], 
         [true, false, false, true], [true, false, false, true], [true, true, true, true]],
        // 9
        [[true, true, true, true], [true, false, false, true], [true, true, true, true], 
         [false, false, false, true], [false, false, false, true], [true, true, true, true]],
    ];
    
    // Position score at top of score area, centered horizontally
    let score_width = (score_str.len() as i32 * (char_w + spacing) * pixel_size) as i32;
    let mut x_pos = ox + (sw - score_width) / 2;
    let y_pos = oy + 5;
    
    // Draw score digits
    for ch in score_str.chars() {
        if ch.is_ascii_digit() {
            let digit = ch as usize - '0' as usize;
            if digit < 10 {
                for row in 0..char_h {
                    for col in 0..char_w {
                        if digits[digit][row as usize][col as usize] {
                            let rx = x_pos + (col * pixel_size);
                            let ry = y_pos + (row * pixel_size);
                            canvas.set_draw_color(Color::RGB(255, 255, 255));
                            let _ = canvas.fill_rect(Rect::new(
                                rx, ry, 
                                pixel_size as u32, 
                                pixel_size as u32
                            ));
                        }
                    }
                }
            }
            x_pos += (char_w + spacing) as i32 * pixel_size;
        }
    }
}

pub fn draw_game(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    cache: &mut RenderCache,
    eaten: &[bool],
    player_x: i32,
    player_y: i32,
    ghost_x: i32,
    ghost_y: i32,
    ghost_vulnerable: bool,
    power_pellet_timer: i32,
    frame: u32,
    alive: bool,
) -> Result<(), String> {
    // Clear
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    // Update cache if needed
    let (ww, wh) = canvas.window().size();
    cache.update_cache(ww as i32, wh as i32);
    
    let s = cache.scale;
    let ox = cache.ox;
    let game_start_y = cache.game_start_y;

    // Helper to convert game coordinates to screen coordinates
    let to_screen = |x: i32, y: i32, w: i32, h: i32| -> Rect {
        let rx = ox + ((x as f32) * s) as i32;
        let ry = game_start_y + ((y as f32) * s) as i32;
        let rw = (w as f32 * s).ceil() as i32;
        let rh = (h as f32 * s).ceil() as i32;
        Rect::new(rx, ry, rw as u32, rh as u32)
    };

    // Batch rendering - collect all rectangles first
    let mut wall_rects = Vec::with_capacity(200);
    let mut pellet_rects = Vec::with_capacity(300);
    let mut power_pellet_rects_white = Vec::with_capacity(4);
    let mut power_pellet_rects_cyan = Vec::with_capacity(4);

    // Collect all rectangles
    for y in 0..GRID_H {
        let bytes = MAZE[y as usize].as_bytes();
        for x in 0..GRID_W {
            let c = bytes[x as usize];
            match c {
                b'#' => {
                    wall_rects.push(to_screen(x * TILE, y * TILE, TILE, TILE));
                }
                b'.' => {
                    let idx = (y * GRID_W + x) as usize;
                    if !eaten[idx] {
                        pellet_rects.push(to_screen(
                            x * TILE + TILE / 2 - 1,
                            y * TILE + TILE / 2 - 1,
                            2,
                            2,
                        ));
                    }
                }
                b'*' => {
                    let idx = (y * GRID_W + x) as usize;
                    if !eaten[idx] {
                        let flash = (frame / 15) % 2 == 0;
                        let rect = to_screen(
                            x * TILE + TILE / 2 - 2,
                            y * TILE + TILE / 2 - 2,
                            4,
                            4,
                        );
                        if flash {
                            power_pellet_rects_cyan.push(rect);
                        } else {
                            power_pellet_rects_white.push(rect);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Draw in batches
    if !wall_rects.is_empty() {
        canvas.set_draw_color(Color::RGB(0, 100, 255));
        for rect in &wall_rects {
            let _ = canvas.fill_rect(*rect);
        }
    }
    
    if !pellet_rects.is_empty() {
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        for rect in &pellet_rects {
            let _ = canvas.fill_rect(*rect);
        }
    }
    
    if !power_pellet_rects_white.is_empty() {
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        for rect in &power_pellet_rects_white {
            let _ = canvas.fill_rect(*rect);
        }
    }
    
    if !power_pellet_rects_cyan.is_empty() {
        canvas.set_draw_color(Color::RGB(0, 255, 255));
        for rect in &power_pellet_rects_cyan {
            let _ = canvas.fill_rect(*rect);
        }
    }

    // Player (yellow Pacman)
    canvas.set_draw_color(Color::RGB(255, 255, 0));
    let _ = canvas.fill_rect(to_screen(
        player_x * TILE,
        player_y * TILE,
        TILE,
        TILE,
    ));

    // Ghost (red normally, blue when vulnerable)
    let ghost_color = if ghost_vulnerable {
        if power_pellet_timer < 120 && (frame / 10) % 2 == 0 {
            Color::RGB(255, 255, 255) // White (flashing when about to expire)
        } else {
            Color::RGB(0, 100, 255) // Blue (vulnerable)
        }
    } else {
        Color::RGB(255, 0, 0) // Red (normal)
    };
    canvas.set_draw_color(ghost_color);
    let _ = canvas.fill_rect(to_screen(
        ghost_x * TILE,
        ghost_y * TILE,
        TILE,
        TILE,
    ));

    // Dead overlay
    if !alive {
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 180));
        let _ = canvas.fill_rect(Rect::new(0, 0, ww, wh));
    }

    Ok(())
}

