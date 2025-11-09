use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::time::{Duration, Instant};

const GRID_W: i32 = 28;
const GRID_H: i32 = 31;
const TILE: i32 = 6; // 28*6 = 168 wide, close to 2600-ish scale
const VIEW_W: i32 = GRID_W * TILE;
const VIEW_H: i32 = GRID_H * TILE;
const SCORE_AREA: i32 = 30; // Space for score at top
const FPS: u32 = 60;
const DT: f64 = 1.0 / FPS as f64;
const WINDOW_SCALE: i32 = 4; // Scale factor for window size

static MAZE: [&str; GRID_H as usize] = [
    "############################",
    "#............##............#",
    "#.####.#####.##.#####.####.#",
    "#*####.#####.##.#####.####*#",
    "#.####.#####.##.#####.####.#",
    "#..........................#",
    "#.####.##.########.##.####.#",
    "#.####.##.########.##.####.#",
    "#......##....##....##......#",
    "######.##### ## #####.######",
    "#####..##### ## #####..#####",
    "#####.##            ##.#####",
    "#......# ### ## ### #......#",
    "######.# #        # #.######",
    "     #.# #  ####  # #.#     ",
    "######.# #        # #.######",
    "#......# ########## #......#",
    "#####.##            ##.#####",
    "#####..##### ## #####..#####",
    "######.##### ## #####.######",
    "#......##....##....##......#",
    "#.####.##.########.##.####.#",
    "#.####.##.########.##.####.#",
    "#...##................##...#",
    "###.##.####.##.####.##.###.#",
    "#*..   ####.##.####   ..*..#",
    "###.##.####.##.####.##.###.#",
    "#...##................##...#",
    "#.##########.##.##########.#",
    "#..........................#",
    "############################",
];

#[inline]
fn is_wall(x: i32, y: i32) -> bool {
    if x < 0 || x >= GRID_W || y < 0 || y >= GRID_H {
        return true;
    }
    MAZE[y as usize].as_bytes()[x as usize] == b'#'
}

#[inline]
fn is_pellet(x: i32, y: i32) -> bool {
    let c = MAZE[y as usize].as_bytes()[x as usize];
    c == b'.' || c == b'*'
}

#[inline]
fn is_power_pellet(x: i32, y: i32) -> bool {
    if x < 0 || x >= GRID_W || y < 0 || y >= GRID_H {
        return false;
    }
    MAZE[y as usize].as_bytes()[x as usize] == b'*'
}

#[inline]
fn is_empty(x: i32, y: i32) -> bool {
    if x < 0 || x >= GRID_W || y < 0 || y >= GRID_H {
        return false;
    }
    MAZE[y as usize].as_bytes()[x as usize] != b'#'
}

// Tiny deterministic LFSR (same each run unless you change seed)
#[derive(Clone, Copy)]
struct Lfsr {
    s: u16,
}
impl Lfsr {
    fn new(seed: u16) -> Self {
        Self { s: if seed == 0 { 0xACE1 } else { seed } }
    }
    fn next(&mut self) -> u16 {
        let lsb = self.s & 1;
        self.s >>= 1;
        if lsb != 0 {
            self.s ^= 0xB400;
        }
        self.s
    }
    fn range(&mut self, lo: i32, hi: i32) -> i32 {
        let span = (hi - lo + 1) as u16;
        lo + (self.next() % span) as i32
    }
}

#[derive(Default, Clone, Copy)]
struct Ram {
    // player
    px: i32,
    py: i32,
    pdx: i32,
    pdy: i32,
    // one ghost for starter
    gx: i32,
    gy: i32,
    gdx: i32,
    gdy: i32,
    // misc
    frame: u32,
    pellets: i32,
    score: i32,
    alive: bool,
    // Atari 2600 features
    power_pellet_timer: i32, // frames remaining for power pellet effect
    ghost_vulnerable: bool,
    ghost_eaten_count: i32, // for scoring multiplier
}

struct Game {
    ram: Ram,
    eaten: Vec<bool>, // shadow pellet map
    rng: Lfsr,
    player_sub: i32,
    ghost_sub: i32,
    ghost_think_timer: i32,
    // Optimization: reusable buffers to avoid allocations
    ghost_opts_buffer: Vec<(i32, i32, i32)>,
    // Optimization: cached rendering values
    cached_scale: f32,
    cached_ox: i32,
    cached_oy: i32,
    cached_game_start_y: i32,
    cached_sw: i32,
    window_size_changed: bool,
    // Input queuing for better direction changes
    queued_dx: i32,
    queued_dy: i32,
}

impl Game {
    fn new() -> Self {
        let mut pellets = 0;
        for y in 0..GRID_H {
            for x in 0..GRID_W {
                if is_pellet(x, y) {
                    pellets += 1;
                }
            }
        }
        Self {
            ram: Ram {
                px: 13,
                py: 23,
                pdx: 0,
                pdy: 0,
                gx: 13,
                gy: 14,
                gdx: 0,
                gdy: -1,
                frame: 0,
                pellets,
                score: 0,
                alive: true,
                power_pellet_timer: 0,
                ghost_vulnerable: false,
                ghost_eaten_count: 0,
            },
            eaten: vec![false; (GRID_W * GRID_H) as usize],
            rng: Lfsr::new(0xACE1),
            player_sub: 0,
            ghost_sub: 0,
            ghost_think_timer: 0,
            ghost_opts_buffer: Vec::with_capacity(4),
            cached_scale: 1.0,
            cached_ox: 0,
            cached_oy: 0,
            cached_game_start_y: 0,
            cached_sw: 0,
            window_size_changed: true,
            queued_dx: 0,
            queued_dy: 0,
        }
    }

    fn pellet_idx(x: i32, y: i32) -> usize {
        (y * GRID_W + x) as usize
    }

    fn ghost_think(&mut self) {
        // Optimization: Reuse buffer instead of allocating new Vecs
        self.ghost_opts_buffer.clear();
        
        // If vulnerable, try to run away from player (simple flee AI)
        if self.ram.ghost_vulnerable {
            // Flee: prefer direction away from player
            let dx_to_player = self.ram.px - self.ram.gx;
            let dy_to_player = self.ram.py - self.ram.gy;
            
            let open_n = !is_wall(self.ram.gx, self.ram.gy - 1);
            let open_s = !is_wall(self.ram.gx, self.ram.gy + 1);
            let open_w = !is_wall(self.ram.gx - 1, self.ram.gy);
            let open_e = !is_wall(self.ram.gx + 1, self.ram.gy);

            if open_n && self.ram.gdy != 1 {
                let priority = if dy_to_player > 0 { 10 } else { 1 };
                self.ghost_opts_buffer.push((0, -1, priority));
            }
            if open_s && self.ram.gdy != -1 {
                let priority = if dy_to_player < 0 { 10 } else { 1 };
                self.ghost_opts_buffer.push((0, 1, priority));
            }
            if open_w && self.ram.gdx != 1 {
                let priority = if dx_to_player > 0 { 10 } else { 1 };
                self.ghost_opts_buffer.push((-1, 0, priority));
            }
            if open_e && self.ram.gdx != -1 {
                let priority = if dx_to_player < 0 { 10 } else { 1 };
                self.ghost_opts_buffer.push((1, 0, priority));
            }

            if self.ghost_opts_buffer.is_empty() {
                self.ram.gdx = -self.ram.gdx;
                self.ram.gdy = -self.ram.gdy;
                return;
            }
            
            // Choose direction with highest priority (fleeing), or random if equal
            self.ghost_opts_buffer.sort_by(|a, b| b.2.cmp(&a.2));
            let best_priority = self.ghost_opts_buffer[0].2;
            let mut best_count = 0;
            for opt in &self.ghost_opts_buffer {
                if opt.2 == best_priority {
                    best_count += 1;
                } else {
                    break;
                }
            }
            let i = self.rng.range(0, best_count - 1) as usize;
            let (dx, dy, _) = self.ghost_opts_buffer[i];
            self.ram.gdx = dx;
            self.ram.gdy = dy;
            return;
        }
        
        // Normal AI: choose a direction at junctions; avoid immediate reversal unless stuck
        let open_n = !is_wall(self.ram.gx, self.ram.gy - 1);
        let open_s = !is_wall(self.ram.gx, self.ram.gy + 1);
        let open_w = !is_wall(self.ram.gx - 1, self.ram.gy);
        let open_e = !is_wall(self.ram.gx + 1, self.ram.gy);

        // Reuse buffer for normal AI (only need 2-tuples, but buffer has 3-tuples - we'll use first 2)
        self.ghost_opts_buffer.clear();
        if open_n && self.ram.gdy != 1 {
            self.ghost_opts_buffer.push((0, -1, 0));
        }
        if open_s && self.ram.gdy != -1 {
            self.ghost_opts_buffer.push((0, 1, 0));
        }
        if open_w && self.ram.gdx != 1 {
            self.ghost_opts_buffer.push((-1, 0, 0));
        }
        if open_e && self.ram.gdx != -1 {
            self.ghost_opts_buffer.push((1, 0, 0));
        }

        if self.ghost_opts_buffer.is_empty() {
            self.ram.gdx = -self.ram.gdx;
            self.ram.gdy = -self.ram.gdy;
            return;
        }
        let i = self.rng.range(0, self.ghost_opts_buffer.len() as i32 - 1) as usize;
        let (dx, dy, _) = self.ghost_opts_buffer[i];
        self.ram.gdx = dx;
        self.ram.gdy = dy;
    }

    // Process input immediately when key is pressed (instantaneous response)
    fn process_input(&mut self, dx: i32, dy: i32) {
        // Always update queued direction (for perpendicular turns)
        if dx != self.ram.pdx || dy != self.ram.pdy {
            self.queued_dx = dx;
            self.queued_dy = dy;
        }

        // Check if we can change direction immediately
        let is_aligned = self.player_sub == 0;
        let is_perpendicular = (dx != 0 && self.ram.pdy != 0) || (dy != 0 && self.ram.pdx != 0);
        let can_turn = (dx != self.ram.pdx || dy != self.ram.pdy) && 
                       !is_wall(self.ram.px + dx, self.ram.py + dy);
        let can_reverse = dx == -self.ram.pdx && dy == -self.ram.pdy;
        
        // Allow immediate turn if:
        // - Aligned to grid AND can turn, OR
        // - Perpendicular turn (90-degree) - allow even when not perfectly aligned, OR
        // - Reverse direction (180-degree) - always allowed
        if can_turn && (is_aligned || is_perpendicular || can_reverse) {
            self.ram.pdx = dx;
            self.ram.pdy = dy;
            // Clear queue if we successfully turned
            if dx == self.queued_dx && dy == self.queued_dy {
                self.queued_dx = 0;
                self.queued_dy = 0;
            }
        }
    }

    fn tick(&mut self, keyboard: &sdl2::keyboard::KeyboardState) {
        self.ram.frame = self.ram.frame.wrapping_add(1);

        // Check current keyboard state for held keys (fallback)
        if keyboard.is_scancode_pressed(Scancode::Up) {
            self.process_input(0, -1);
        } else if keyboard.is_scancode_pressed(Scancode::Down) {
            self.process_input(0, 1);
        } else if keyboard.is_scancode_pressed(Scancode::Left) {
            self.process_input(-1, 0);
        } else if keyboard.is_scancode_pressed(Scancode::Right) {
            self.process_input(1, 0);
        }

        // player moves every N sub-frames (integer speed) - slower for Atari 2600 feel
        self.player_sub += 1;
        if self.player_sub >= 4 {
            self.player_sub = 0;
            
            // Check queued direction when aligned (for perpendicular turns that were queued)
            if self.queued_dx != 0 || self.queued_dy != 0 {
                if !is_wall(self.ram.px + self.queued_dx, self.ram.py + self.queued_dy) {
                    self.ram.pdx = self.queued_dx;
                    self.ram.pdy = self.queued_dy;
                    self.queued_dx = 0;
                    self.queued_dy = 0;
                } else {
                    // Can't turn to queued direction, clear queue
                    self.queued_dx = 0;
                    self.queued_dy = 0;
                }
            }
            
            let mut nx = self.ram.px + self.ram.pdx;
            let ny = self.ram.py + self.ram.pdy;
            // wrap tunnels on row 14 (index starts at 0)
            if ny == 14 && nx < 0 {
                nx = GRID_W - 1;
            }
            if ny == 14 && nx >= GRID_W {
                nx = 0;
            }
            if !is_wall(nx, ny) {
                self.ram.px = nx;
                self.ram.py = ny;
            } else {
                // Hit a wall, stop and clear queue
                self.ram.pdx = 0;
                self.ram.pdy = 0;
                self.queued_dx = 0;
                self.queued_dy = 0;
            }
        }

        // pellet eat
        if is_pellet(self.ram.px, self.ram.py) {
            let idx = Game::pellet_idx(self.ram.px, self.ram.py);
            if !self.eaten[idx] {
                self.eaten[idx] = true;
                self.ram.pellets -= 1;
                
                // Power pellet (marked with *)
                if is_power_pellet(self.ram.px, self.ram.py) {
                    self.ram.score += 50; // Power pellets worth 50 points
                    self.ram.power_pellet_timer = 900; // ~15 seconds at 60 FPS (slower pace)
                    self.ram.ghost_vulnerable = true;
                    self.ram.ghost_eaten_count = 0; // Reset multiplier
                } else {
                    self.ram.score += 10; // Regular pellets worth 10 points
                }
            }
        }
        
        // Update power pellet timer
        if self.ram.power_pellet_timer > 0 {
            self.ram.power_pellet_timer -= 1;
            if self.ram.power_pellet_timer == 0 {
                self.ram.ghost_vulnerable = false;
            }
        }

        // ghost AI
        self.ghost_think_timer += 1;
        if self.ghost_think_timer >= 8 {
            self.ghost_think();
            self.ghost_think_timer = 0;
        }

        // ghost moves - slower for Atari 2600 feel
        self.ghost_sub += 1;
        if self.ghost_sub >= 5 {
            self.ghost_sub = 0;
            let mut nx = self.ram.gx + self.ram.gdx;
            let ny = self.ram.gy + self.ram.gdy;
            if ny == 14 && nx < 0 {
                nx = GRID_W - 1;
            }
            if ny == 14 && nx >= GRID_W {
                nx = 0;
            }
            if !is_wall(nx, ny) {
                self.ram.gx = nx;
                self.ram.gy = ny;
            } else {
                self.ram.gdx = 0;
                self.ram.gdy = 0;
                self.ghost_think();
            }
        }

        // collision (tile-precise)
        if self.ram.px == self.ram.gx && self.ram.py == self.ram.gy {
            if self.ram.ghost_vulnerable {
                // Eat the ghost! Atari 2600 scoring: 200, 400, 800, 1600
                let points = [200, 400, 800, 1600];
                let multiplier_idx = self.ram.ghost_eaten_count.min(3) as usize;
                self.ram.score += points[multiplier_idx];
                self.ram.ghost_eaten_count += 1;
                
                // Reset ghost to center
                self.ram.gx = 13;
                self.ram.gy = 14;
                self.ram.gdx = 0;
                self.ram.gdy = -1;
            } else {
                self.ram.alive = false;
            }
        }
    }

    fn draw(&mut self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) -> Result<(), String> {
        // clear
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        // Optimization: Cache scaling calculations - only recalculate if window size changed
        let (ww, wh) = canvas.window().size();
        let ww = ww as i32;
        let wh = wh as i32;
        
        if self.window_size_changed {
            let total_view_h = VIEW_H + SCORE_AREA;
            let sx = ww as f32 / VIEW_W as f32;
            let sy = wh as f32 / total_view_h as f32;
            self.cached_scale = sx.min(sy);
            self.cached_sw = (VIEW_W as f32 * self.cached_scale).floor() as i32;
            let total_sh = (total_view_h as f32 * self.cached_scale).floor() as i32;
            self.cached_ox = (ww - self.cached_sw) / 2;
            self.cached_oy = (wh - total_sh) / 2;
            let score_area_scaled = (SCORE_AREA as f32 * self.cached_scale).floor() as i32;
            self.cached_game_start_y = self.cached_oy + score_area_scaled;
            self.window_size_changed = false;
        }
        
        let s = self.cached_scale;
        let ox = self.cached_ox;
        let oy = self.cached_oy;
        let game_start_y = self.cached_game_start_y;
        let sw = self.cached_sw;
        let score_area_scaled = (SCORE_AREA as f32 * s).floor() as i32;

        // Draw score first at the top
        self.draw_score_simple(canvas, ox, oy, s, score_area_scaled, sw);
        
        // Optimization: Batch rendering - collect all rectangles first, then draw in batches
        // Pre-allocate with estimated capacity to avoid reallocations
        let mut wall_rects = Vec::with_capacity(200);
        let mut pellet_rects = Vec::with_capacity(300);
        let mut power_pellet_rects_white = Vec::with_capacity(4);
        let mut power_pellet_rects_cyan = Vec::with_capacity(4);
        
        // Helper to convert game coordinates to screen coordinates
        let to_screen = |x: i32, y: i32, w: i32, h: i32| -> Rect {
            let rx = ox + ((x as f32) * s) as i32;
            let ry = game_start_y + ((y as f32) * s) as i32;
            let rw = (w as f32 * s).ceil() as i32;
            let rh = (h as f32 * s).ceil() as i32;
            Rect::new(rx, ry, rw as u32, rh as u32)
        };

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
                        let idx = Game::pellet_idx(x, y);
                        if !self.eaten[idx] {
                            pellet_rects.push(to_screen(
                                x * TILE + TILE / 2 - 1,
                                y * TILE + TILE / 2 - 1,
                                2,
                                2,
                            ));
                        }
                    }
                    b'*' => {
                        let idx = Game::pellet_idx(x, y);
                        if !self.eaten[idx] {
                            let flash = (self.ram.frame / 15) % 2 == 0;
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

        // Draw in batches (only 4-5 set_draw_color calls instead of 868+)
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
            self.ram.px * TILE,
            self.ram.py * TILE,
            TILE,
            TILE,
        ));

        // Ghost (red normally, blue when vulnerable)
        let ghost_color = if self.ram.ghost_vulnerable {
            if self.ram.power_pellet_timer < 120 && (self.ram.frame / 10) % 2 == 0 {
                Color::RGB(255, 255, 255) // White (flashing when about to expire)
            } else {
                Color::RGB(0, 100, 255) // Blue (vulnerable)
            }
        } else {
            Color::RGB(255, 0, 0) // Red (normal)
        };
        canvas.set_draw_color(ghost_color);
        let _ = canvas.fill_rect(to_screen(
            self.ram.gx * TILE,
            self.ram.gy * TILE,
            TILE,
            TILE,
        ));

        // dead overlay
        if !self.ram.alive {
            canvas.set_draw_color(Color::RGBA(0, 0, 0, 180));
            let _ = canvas.fill_rect(Rect::new(0, 0, ww as u32, wh as u32));
        }

        canvas.present();
        Ok(())
    }

    // Simple score display using rectangles (Atari 2600 style - pixel font)
    fn draw_score_simple(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, 
                         ox: i32, oy: i32, _s: f32, _score_area_scaled: i32, sw: i32) {
        let score_str = format!("{:06}", self.ram.score);
        let char_w = 4;
        let char_h = 6;
        let spacing = 1;
        let pixel_size = 2; // Size of each pixel in the font
        
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
}

fn main() -> Result<(), String> {
    // Init SDL
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    // Calculate window size to fit game content with minimal borders
    let window_w = (VIEW_W * WINDOW_SCALE) as u32;
    let window_h = ((VIEW_H + SCORE_AREA) * WINDOW_SCALE) as u32;
    
    let window = video
        .window("Pacman - Atari 2600 Style (Rust)", window_w, window_h)
        .position_centered()
        .opengl()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window
        .into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    // main loop
    let mut event_pump = sdl.event_pump()?;
    let mut game = Game::new();
    let mut acc = 0.0f64;
    let mut prev = Instant::now();
    let dt = DT;

    'running: loop {
        // Process ALL events immediately - instantaneous input response
        // No queuing, no waiting - process input the moment the key is pressed
        for e in event_pump.poll_iter() {
            match e {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { scancode: Some(Scancode::Escape), .. } => break 'running,
                Event::KeyDown { scancode: Some(Scancode::Up), .. } => {
                    // Process input INSTANTLY - no delay, no queuing
                    game.process_input(0, -1);
                }
                Event::KeyDown { scancode: Some(Scancode::Down), .. } => {
                    game.process_input(0, 1);
                }
                Event::KeyDown { scancode: Some(Scancode::Left), .. } => {
                    game.process_input(-1, 0);
                }
                Event::KeyDown { scancode: Some(Scancode::Right), .. } => {
                    game.process_input(1, 0);
                }
                Event::Window { win_event, .. } => {
                    // Optimization: Mark window size changed on resize
                    if matches!(win_event, sdl2::event::WindowEvent::Resized(_, _) | 
                                       sdl2::event::WindowEvent::SizeChanged(_, _)) {
                        game.window_size_changed = true;
                    }
                }
                _ => {}
            }
        }

        // fixed-step accumulator
        let now = Instant::now();
        let elapsed = now.duration_since(prev);
        prev = now;
        acc += (elapsed.as_secs_f64()).min(0.25); // avoid spiral

        let keys = event_pump.keyboard_state();
        while acc >= dt {
            if game.ram.alive {
                game.tick(&keys);
            }
            acc -= dt;
        }

        game.draw(&mut canvas)?;
        // small sleep to reduce CPU if vsync off
        std::thread::sleep(Duration::from_millis(1));
    }
    Ok(())
}
