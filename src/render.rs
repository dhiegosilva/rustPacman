// Rendering code

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use crate::constants::{GRID_W, GRID_H, TILE, VIEW_W, VIEW_H, SCORE_AREA};
use crate::maze::get_maze;

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
    // Compact bitmask: each digit is 4x6, stored as 6 u8s (one per row)
    const DIGITS: [[u8; 6]; 10] = [
        [0b1111, 0b1001, 0b1001, 0b1001, 0b1001, 0b1111], // 0
        [0b0010, 0b0110, 0b0010, 0b0010, 0b0010, 0b0111], // 1
        [0b1111, 0b0001, 0b1111, 0b1000, 0b1000, 0b1111], // 2
        [0b1111, 0b0001, 0b1111, 0b0001, 0b0001, 0b1111], // 3
        [0b1001, 0b1001, 0b1111, 0b0001, 0b0001, 0b0001], // 4
        [0b1111, 0b1000, 0b1111, 0b0001, 0b0001, 0b1111], // 5
        [0b1111, 0b1000, 0b1111, 0b1001, 0b1001, 0b1111], // 6
        [0b1111, 0b0001, 0b0001, 0b0001, 0b0001, 0b0001], // 7
        [0b1111, 0b1001, 0b1111, 0b1001, 0b1001, 0b1111], // 8
        [0b1111, 0b1001, 0b1111, 0b0001, 0b0001, 0b1111], // 9
    ];
    
    let score_str = format!("{:06}", score);
    let char_w = 4;
    let spacing = 1;
    let pixel_size = 2;
    let score_width = (score_str.len() as i32 * (char_w + spacing) * pixel_size) as i32;
    let mut x_pos = ox + (sw - score_width) / 2;
    let y_pos = oy + 5;
    
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for ch in score_str.chars() {
        if let Some(digit) = ch.to_digit(10).and_then(|d| DIGITS.get(d as usize)) {
            for (row, &bits) in digit.iter().enumerate() {
                for col in 0..char_w {
                    if (bits >> (char_w - 1 - col)) & 1 != 0 {
                        let _ = canvas.fill_rect(Rect::new(
                            x_pos + (col as i32 * pixel_size),
                            y_pos + (row as i32 * pixel_size),
                            pixel_size as u32,
                            pixel_size as u32,
                        ));
                    }
                }
            }
            x_pos += (char_w + spacing) * pixel_size;
        }
    }
}

pub fn draw_game(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    cache: &mut RenderCache,
    eaten: &[bool],
    player_x: i32,
    player_y: i32,
    ghosts: &[(i32, i32, bool)],
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
    let maze = get_maze();
    for y in 0..GRID_H {
        let y_idx = y as usize;
        if y_idx >= maze.len() {
            continue;
        }
        let row = maze[y_idx];
        if row.len() < GRID_W as usize {
            continue;
        }
        let bytes = row.as_bytes();
        for x in 0..GRID_W {
            let x_idx = x as usize;
            if x_idx >= bytes.len() {
                continue;
            }
            let c = bytes[x_idx];
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

    // Ghosts (different colors, blue when vulnerable)
    let ghost_colors = [Color::RGB(255, 0, 0), Color::RGB(255, 184, 255), Color::RGB(0, 255, 255)]; // Red, Pink, Cyan
    for (i, (ghost_x, ghost_y, ghost_vulnerable)) in ghosts.iter().enumerate() {
        let ghost_color = if *ghost_vulnerable {
            if power_pellet_timer < 120 && (frame / 10) % 2 == 0 {
                Color::RGB(255, 255, 255) // White (flashing when about to expire)
            } else {
                Color::RGB(0, 100, 255) // Blue (vulnerable)
            }
        } else {
            ghost_colors[i.min(2)]
        };
        canvas.set_draw_color(ghost_color);
        let _ = canvas.fill_rect(to_screen(
            *ghost_x * TILE,
            *ghost_y * TILE,
            TILE,
            TILE,
        ));
    }

    // Dead overlay
    if !alive {
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 180));
        let _ = canvas.fill_rect(Rect::new(0, 0, ww, wh));
    }

    Ok(())
}

