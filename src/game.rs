// Main game state and logic

use crate::constants::{GRID_W, GRID_H};
use crate::maze::{is_pellet, is_power_pellet, count_pellets};
use crate::player::Player;
use crate::ghost::Ghost;
use crate::rng::Lfsr;
use crate::render::{RenderCache, draw_score, draw_game};
use sdl2::keyboard::Scancode;

pub struct Game {
    pub player: Player,
    pub ghost: Ghost,
    pub eaten: Vec<bool>, // Shadow pellet map
    pub rng: Lfsr,
    pub frame: u32,
    pub pellets: i32,
    pub score: i32,
    pub alive: bool,
    pub power_pellet_timer: i32,
    pub ghost_eaten_count: i32, // For scoring multiplier
    pub render_cache: RenderCache,
}

impl Game {
    pub fn new() -> Self {
        let pellets = count_pellets();
        Self {
            player: Player::new(),
            ghost: Ghost::new(),
            eaten: vec![false; (GRID_W * GRID_H) as usize],
            rng: Lfsr::new(0xACE1),
            frame: 0,
            pellets,
            score: 0,
            alive: true,
            power_pellet_timer: 0,
            ghost_eaten_count: 0,
            render_cache: RenderCache::new(),
        }
    }

    fn pellet_idx(x: i32, y: i32) -> usize {
        (y * GRID_W + x) as usize
    }

    // Process input immediately when key is pressed (instantaneous response)
    pub fn process_input(&mut self, dx: i32, dy: i32) {
        self.player.process_input(dx, dy);
    }

    pub fn tick(&mut self, keyboard: &sdl2::keyboard::KeyboardState) {
        self.frame = self.frame.wrapping_add(1);

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

        // Update player
        self.player.update();

        // Pellet eating
        if is_pellet(self.player.x, self.player.y) {
            let idx = Game::pellet_idx(self.player.x, self.player.y);
            if !self.eaten[idx] {
                self.eaten[idx] = true;
                self.pellets -= 1;
                
                // Power pellet (marked with *)
                if is_power_pellet(self.player.x, self.player.y) {
                    self.score += 50; // Power pellets worth 50 points
                    self.power_pellet_timer = 900; // ~15 seconds at 60 FPS
                    self.ghost.vulnerable = true;
                    self.ghost_eaten_count = 0; // Reset multiplier
                } else {
                    self.score += 10; // Regular pellets worth 10 points
                }
            }
        }
        
        // Update power pellet timer
        if self.power_pellet_timer > 0 {
            self.power_pellet_timer -= 1;
            if self.power_pellet_timer == 0 {
                self.ghost.vulnerable = false;
            }
        }

        // Update ghost
        self.ghost.update(&mut self.rng, self.player.x, self.player.y);

        // Collision (tile-precise)
        if self.player.x == self.ghost.x && self.player.y == self.ghost.y {
            if self.ghost.vulnerable {
                // Eat the ghost! Atari 2600 scoring: 200, 400, 800, 1600
                let points = [200, 400, 800, 1600];
                let multiplier_idx = self.ghost_eaten_count.min(3) as usize;
                self.score += points[multiplier_idx];
                self.ghost_eaten_count += 1;
                self.ghost.reset_to_center();
            } else {
                self.alive = false;
            }
        }
    }

    pub fn draw(&mut self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) -> Result<(), String> {
        let (ww, wh) = canvas.window().size();
        
        // Update cache first
        self.render_cache.update_cache(ww as i32, wh as i32);
        
        // Draw game (this clears the canvas)
        draw_game(
            canvas,
            &mut self.render_cache,
            &self.eaten,
            self.player.x,
            self.player.y,
            self.ghost.x,
            self.ghost.y,
            self.ghost.vulnerable,
            self.power_pellet_timer,
            self.frame,
            self.alive,
        )?;
        
        // Draw score after game (so it appears on top)
        draw_score(canvas, self.score, self.render_cache.ox, self.render_cache.oy, self.render_cache.sw);
        
        // Present the frame after everything is drawn
        canvas.present();
        Ok(())
    }
}

