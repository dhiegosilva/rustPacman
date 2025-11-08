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
        }
    }

    fn pellet_idx(x: i32, y: i32) -> usize {
        (y * GRID_W + x) as usize
    }

    fn ghost_think(&mut self) {
        // If vulnerable, try to run away from player (simple flee AI)
        if self.ram.ghost_vulnerable {
            // Flee: prefer direction away from player
            let dx_to_player = self.ram.px - self.ram.gx;
            let dy_to_player = self.ram.py - self.ram.gy;
            
            let open_n = !is_wall(self.ram.gx, self.ram.gy - 1);
            let open_s = !is_wall(self.ram.gx, self.ram.gy + 1);
            let open_w = !is_wall(self.ram.gx - 1, self.ram.gy);
            let open_e = !is_wall(self.ram.gx + 1, self.ram.gy);

            let mut opts: Vec<(i32, i32, i32)> = Vec::with_capacity(4); // (dx, dy, priority)
            
            if open_n && self.ram.gdy != 1 {
                let priority = if dy_to_player > 0 { 10 } else { 1 }; // prefer north if player is south
                opts.push((0, -1, priority));
            }
            if open_s && self.ram.gdy != -1 {
                let priority = if dy_to_player < 0 { 10 } else { 1 };
                opts.push((0, 1, priority));
            }
            if open_w && self.ram.gdx != 1 {
                let priority = if dx_to_player > 0 { 10 } else { 1 };
                opts.push((-1, 0, priority));
            }
            if open_e && self.ram.gdx != -1 {
                let priority = if dx_to_player < 0 { 10 } else { 1 };
                opts.push((1, 0, priority));
            }

            if opts.is_empty() {
                self.ram.gdx = -self.ram.gdx;
                self.ram.gdy = -self.ram.gdy;
                return;
            }
            
            // Choose direction with highest priority (fleeing), or random if equal
            opts.sort_by(|a, b| b.2.cmp(&a.2));
            let best_priority = opts[0].2;
            let flee_opts: Vec<_> = opts.iter().filter(|o| o.2 == best_priority).collect();
            let i = self.rng.range(0, flee_opts.len() as i32 - 1) as usize;
            let (dx, dy, _) = flee_opts[i];
            self.ram.gdx = *dx;
            self.ram.gdy = *dy;
            return;
        }
        
        // Normal AI: choose a direction at junctions; avoid immediate reversal unless stuck
        let open_n = !is_wall(self.ram.gx, self.ram.gy - 1);
        let open_s = !is_wall(self.ram.gx, self.ram.gy + 1);
        let open_w = !is_wall(self.ram.gx - 1, self.ram.gy);
        let open_e = !is_wall(self.ram.gx + 1, self.ram.gy);

        let mut opts: Vec<(i32, i32)> = Vec::with_capacity(4);
        if open_n && self.ram.gdy != 1 {
            opts.push((0, -1));
        }
        if open_s && self.ram.gdy != -1 {
            opts.push((0, 1));
        }
        if open_w && self.ram.gdx != 1 {
            opts.push((-1, 0));
        }
        if open_e && self.ram.gdx != -1 {
            opts.push((1, 0));
        }

        if opts.is_empty() {
            self.ram.gdx = -self.ram.gdx;
            self.ram.gdy = -self.ram.gdy;
            return;
        }
        let i = self.rng.range(0, opts.len() as i32 - 1) as usize;
        let (dx, dy) = opts[i];
        self.ram.gdx = dx;
        self.ram.gdy = dy;
    }

    fn tick(&mut self, keyboard: &sdl2::keyboard::KeyboardState) {
        self.ram.frame = self.ram.frame.wrapping_add(1);

        // Get desired direction from input
        let mut wantdx = self.ram.pdx;
        let mut wantdy = self.ram.pdy;

        if keyboard.is_scancode_pressed(Scancode::Up) {
            wantdx = 0; wantdy = -1;
        }
        if keyboard.is_scancode_pressed(Scancode::Down) {
            wantdx = 0; wantdy = 1;
        }
        if keyboard.is_scancode_pressed(Scancode::Left) {
            wantdx = -1; wantdy = 0;
        }
        if keyboard.is_scancode_pressed(Scancode::Right) {
            wantdx = 1; wantdy = 0;
        }

        // Check if we can change direction
        // Allow direction change if the next tile in desired direction is not a wall
        // and we're aligned to the grid (can turn at any grid-aligned position)
        let is_aligned = self.player_sub == 0;
        let can_turn = (wantdx != self.ram.pdx || wantdy != self.ram.pdy) && 
                       !is_wall(self.ram.px + wantdx, self.ram.py + wantdy);
        
        // Change direction immediately if:
        // - We're aligned to grid AND the desired direction is valid, OR
        // - We can reverse direction (180-degree turn allowed anytime)
        let can_reverse = wantdx == -self.ram.pdx && wantdy == -self.ram.pdy;
        
        if can_turn && (is_aligned || can_reverse) {
            self.ram.pdx = wantdx;
            self.ram.pdy = wantdy;
        }

        // player moves every N sub-frames (integer speed) - slower for Atari 2600 feel
        self.player_sub += 1;
        if self.player_sub >= 4 {
            self.player_sub = 0;
            
            // Check input again right before moving (for queued direction changes)
            // This allows pressing a direction key just before reaching a corner
            let mut final_dx = self.ram.pdx;
            let mut final_dy = self.ram.pdy;
            
            if keyboard.is_scancode_pressed(Scancode::Up) {
                final_dx = 0; final_dy = -1;
            }
            if keyboard.is_scancode_pressed(Scancode::Down) {
                final_dx = 0; final_dy = 1;
            }
            if keyboard.is_scancode_pressed(Scancode::Left) {
                final_dx = -1; final_dy = 0;
            }
            if keyboard.is_scancode_pressed(Scancode::Right) {
                final_dx = 1; final_dy = 0;
            }
            
            // Apply direction change if valid
            if !is_wall(self.ram.px + final_dx, self.ram.py + final_dy) {
                self.ram.pdx = final_dx;
                self.ram.pdy = final_dy;
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
                // Hit a wall, stop
                self.ram.pdx = 0;
                self.ram.pdy = 0;
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

    fn draw(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) -> Result<(), String> {
        // clear
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        // compute integer scaling - account for score area at top
        let (ww, wh) = canvas.window().size();
        let ww = ww as i32;
        let wh = wh as i32;
        let total_view_h = VIEW_H + SCORE_AREA;
        let sx = ww as f32 / VIEW_W as f32;
        let sy = wh as f32 / total_view_h as f32;
        let s = sx.min(sy);
        let sw = (VIEW_W as f32 * s).floor() as i32;
        let total_sh = (total_view_h as f32 * s).floor() as i32;
        let ox = (ww - sw) / 2;
        let oy = (wh - total_sh) / 2;

        // Draw score first at the top (before creating closure that borrows canvas)
        let score_area_scaled = (SCORE_AREA as f32 * s).floor() as i32;
        let game_start_y = oy + score_area_scaled;
        self.draw_score_simple(canvas, ox, oy, s, score_area_scaled, sw);
        
        let mut draw_rect = |x: i32, y: i32, w: i32, h: i32, color: Color| {
            let rx = ox + ((x as f32) * s) as i32;
            let ry = game_start_y + ((y as f32) * s) as i32; // Offset by score area
            let rw = (w as f32 * s).ceil() as i32;
            let rh = (h as f32 * s).ceil() as i32;
            canvas.set_draw_color(color);
            let _ = canvas.fill_rect(Rect::new(rx, ry, rw as u32, rh as u32));
        };

        // walls & pellets (pellet render is cosmetic; removal uses shadow map)
        for y in 0..GRID_H {
            let bytes = MAZE[y as usize].as_bytes();
            for x in 0..GRID_W {
                let c = bytes[x as usize];
                match c {
                    b'#' => {
                        // Atari 2600: Blue walls (or cyan in some versions)
                        draw_rect(x * TILE, y * TILE, TILE, TILE, Color::RGB(0, 100, 255));
                    }
                    b'.' => {
                        // Regular pellet: small white/yellow dot
                        let idx = Game::pellet_idx(x, y);
                        if !self.eaten[idx] {
                            draw_rect(
                                x * TILE + TILE / 2 - 1,
                                y * TILE + TILE / 2 - 1,
                                2,
                                2,
                                Color::RGB(255, 255, 255), // White pellets (Atari 2600 style)
                            );
                        }
                    }
                    b'*' => {
                        // Power pellet: larger, flashing white/cyan
                        let idx = Game::pellet_idx(x, y);
                        if !self.eaten[idx] {
                            let flash = (self.ram.frame / 15) % 2 == 0; // Flash every ~0.25 seconds
                            let color = if flash { 
                                Color::RGB(0, 255, 255) // Cyan when flashing
                            } else { 
                                Color::RGB(255, 255, 255) // White
                            };
                            draw_rect(
                                x * TILE + TILE / 2 - 2,
                                y * TILE + TILE / 2 - 2,
                                4,
                                4,
                                color,
                            );
                        }
                    }
                    _ => {}
                }
            }
        }

        // player (yellow Pacman - Atari 2600 style)
        draw_rect(
            self.ram.px * TILE,
            self.ram.py * TILE,
            TILE,
            TILE,
            Color::RGB(255, 255, 0), // Yellow
        );

        // ghost (red normally, blue when vulnerable - Atari 2600 style)
        let ghost_color = if self.ram.ghost_vulnerable {
            // Flash between blue and white when timer is running low
            if self.ram.power_pellet_timer < 120 && (self.ram.frame / 10) % 2 == 0 {
                Color::RGB(255, 255, 255) // White (flashing when about to expire)
            } else {
                Color::RGB(0, 100, 255) // Blue (vulnerable)
            }
        } else {
            Color::RGB(255, 0, 0) // Red (normal)
        };
        draw_rect(
            self.ram.gx * TILE,
            self.ram.gy * TILE,
            TILE,
            TILE,
            ghost_color,
        );

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
        for e in event_pump.poll_iter() {
            match e {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { scancode: Some(Scancode::Escape), .. } => break 'running,
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
