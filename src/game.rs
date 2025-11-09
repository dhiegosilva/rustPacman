//! Main game state and logic
//! 
//! This module manages the overall game state, including the player, ghosts,
//! pellets, scoring, and game loop updates.

use crate::constants::{
    GRID_W, GRID_H, 
    SCORE_PELLET, SCORE_POWER_PELLET, SCORE_GHOST,
    POWER_PELLET_DURATION, POWER_PELLET_FLASH_START
};
use crate::maze::{is_pellet, is_power_pellet, count_pellets};
use crate::player::Player;
use crate::ghost::Ghost;
use crate::rng::Lfsr;
use crate::render::{RenderCache, draw_score, draw_game};
use crate::game_config::{GameConfig, GameMode, PlayerRole};
use sdl2::keyboard::Scancode;

/// Main game state structure
/// 
/// Contains all the game's state: player, ghosts, pellets, score, etc.
pub struct Game {
    /// The player (Pac-Man)
    pub player: Player,
    /// Array of 3 ghosts
    pub ghosts: [Ghost; 3],
    /// Tracks which pellets have been eaten (true = eaten, false = not eaten)
    pub eaten: Vec<bool>,
    /// Random number generator for ghost AI
    pub rng: Lfsr,
    /// Current frame number (increments each frame)
    pub frame: u32,
    /// Number of pellets remaining
    pub pellets: i32,
    /// Current score
    pub score: i32,
    /// Whether the player is still alive
    pub alive: bool,
    /// Timer for power pellet effect (counts down from POWER_PELLET_DURATION)
    pub power_pellet_timer: i32,
    /// Number of ghosts eaten in current power pellet cycle (for scoring multiplier)
    pub ghost_eaten_count: i32,
    /// Rendering cache for performance optimization
    pub render_cache: RenderCache,
    /// Game configuration (player modes and roles)
    pub config: GameConfig,
    /// Which ghost is controlled by player (if any)
    pub player_ghost_index: Option<usize>,
}

impl Game {
    /// Creates a new game with initial state
    pub fn new(config: GameConfig) -> Self {
        let total_pellets = count_pellets();
        
        // Determine which ghost is player-controlled (if any)
        let player_ghost_index = if config.player1_role == PlayerRole::Ghost {
            Some(0)  // First ghost is player 1
        } else if config.mode == GameMode::Multiplayer && 
                  config.player2_role == Some(PlayerRole::Ghost) {
            Some(1)  // Second ghost is player 2
        } else {
            None
        };
        
        Self {
            player: Player::new(),
            ghosts: [
                Ghost::new_at(12, 14),  // Left ghost
                Ghost::new_at(13, 14),  // Center ghost
                Ghost::new_at(14, 14),  // Right ghost
            ],
            eaten: vec![false; (GRID_W * GRID_H) as usize],
            rng: Lfsr::new(0xACE1),  // Seed for random number generator
            frame: 0,
            pellets: total_pellets,
            score: 0,
            alive: true,
            power_pellet_timer: 0,
            ghost_eaten_count: 0,
            render_cache: RenderCache::new(),
            config,
            player_ghost_index,
        }
    }

    /// Converts grid coordinates (x, y) to an index in the eaten array
    fn pellet_index(x: i32, y: i32) -> usize {
        (y * GRID_W + x) as usize
    }

    /// Updates the game state for one frame
    /// 
    /// This is called every frame and handles:
    /// - Input processing
    /// - Player movement
    /// - Pellet collection
    /// - Power pellet effects
    /// - Ghost AI and movement
    /// - Collision detection
    /// 
    /// # Arguments
    /// * `keyboard` - Current keyboard state (for held keys as fallback)
    /// * `player2_input` - Optional input for player 2 (in multiplayer)
    pub fn tick(&mut self, keyboard: &sdl2::keyboard::KeyboardState, player2_input: Option<(i32, i32)>) {
        self.frame = self.frame.wrapping_add(1);

        // Handle player 1 input (Pac-Man or Ghost)
        let player1_is_pacman = self.config.player1_role == PlayerRole::PacMan;
        let player1_is_ghost = self.config.player1_role == PlayerRole::Ghost;
        
        if player1_is_pacman {
            // Player 1 controls Pac-Man
            const DIRECTION_KEYS: [(Scancode, (i32, i32)); 4] = [
                (Scancode::Up, (0, -1)),
                (Scancode::Down, (0, 1)),
                (Scancode::Left, (-1, 0)),
                (Scancode::Right, (1, 0)),
            ];
            if let Some((_, (dx, dy))) = DIRECTION_KEYS.iter().find(|(sc, _)| keyboard.is_scancode_pressed(*sc)) {
                self.process_input(*dx, *dy);
            }
        } else if player1_is_ghost && self.player_ghost_index == Some(0) {
            // Player 1 controls first ghost
            const DIRECTION_KEYS: [(Scancode, (i32, i32)); 4] = [
                (Scancode::Up, (0, -1)),
                (Scancode::Down, (0, 1)),
                (Scancode::Left, (-1, 0)),
                (Scancode::Right, (1, 0)),
            ];
            if let Some((_, (dx, dy))) = DIRECTION_KEYS.iter().find(|(sc, _)| keyboard.is_scancode_pressed(*sc)) {
                if let Some(ghost) = self.ghosts.get_mut(0) {
                    ghost.process_input(*dx, *dy);
                }
            }
        }
        
        // Handle player 2 input (in multiplayer)
        if let Some((dx, dy)) = player2_input {
            let player2_is_pacman = self.config.player2_role == Some(PlayerRole::PacMan);
            let player2_is_ghost = self.config.player2_role == Some(PlayerRole::Ghost);
            
            if player2_is_pacman {
                // Player 2 controls Pac-Man (in multiplayer)
                self.process_input(dx, dy);
            } else if player2_is_ghost && self.player_ghost_index == Some(1) {
                // Player 2 controls second ghost
                if let Some(ghost) = self.ghosts.get_mut(1) {
                    ghost.process_input(dx, dy);
                }
            }
        }

        // Update Pac-Man (player-controlled or AI)
        if self.config.pacman_is_ai() {
            // AI-controlled Pac-Man
            let ghost_data: Vec<(i32, i32, bool)> = self.ghosts.iter()
                .map(|ghost| (ghost.x, ghost.y, ghost.vulnerable))
                .collect();
            self.player.update_ai(&ghost_data, self.power_pellet_timer > 0, 
                                 &self.eaten, &mut self.rng);
        }
        self.player.update();

        // Check if player is on a pellet
        self.handle_pellet_collection();

        // Update power pellet timer and effects
        self.update_power_pellet_timer();

        // Update all ghosts (AI and movement)
        for (i, ghost) in self.ghosts.iter_mut().enumerate() {
            // Skip AI update if this ghost is player-controlled
            let is_player_controlled = self.player_ghost_index == Some(i);
            if !is_player_controlled {
                ghost.update(&mut self.rng, self.player.x, self.player.y);
            } else {
                // Player-controlled ghost: just update movement
                ghost.update_movement_only();
            }
        }

        // Check for collisions between player and ghosts
        self.check_collisions();
    }
    
    /// Processes input for Pac-Man (called from main loop)
    pub fn process_input(&mut self, dx: i32, dy: i32) {
        if !self.config.pacman_is_ai() {
            self.player.process_input(dx, dy);
        }
    }

    /// Handles pellet collection when player moves onto a pellet
    fn handle_pellet_collection(&mut self) {
        if is_pellet(self.player.x, self.player.y) {
            let pellet_index = Game::pellet_index(self.player.x, self.player.y);
            
            // Only collect if not already eaten
            if !self.eaten[pellet_index] {
                self.eaten[pellet_index] = true;
                self.pellets -= 1;
                
                // Check if it's a power pellet (marked with *)
                if is_power_pellet(self.player.x, self.player.y) {
                    self.score += SCORE_POWER_PELLET;
                    self.power_pellet_timer = POWER_PELLET_DURATION;
                    
                    // Make all ghosts vulnerable
                    for ghost in &mut self.ghosts {
                        ghost.vulnerable = true;
                    }
                    
                    // Reset ghost eaten counter for new power pellet cycle
                    self.ghost_eaten_count = 0;
                } else {
                    // Regular pellet
                    self.score += SCORE_PELLET;
                }
            }
        }
    }

    /// Updates the power pellet timer and removes vulnerability when it expires
    fn update_power_pellet_timer(&mut self) {
        if self.power_pellet_timer > 0 {
            self.power_pellet_timer -= 1;
            
            // When timer expires, make ghosts normal again
            if self.power_pellet_timer == 0 {
                for ghost in &mut self.ghosts {
                    ghost.vulnerable = false;
                }
            }
        }
    }

    /// Checks for collisions between player and ghosts
    /// 
    /// If player collides with a vulnerable ghost: eat it and score points
    /// If player collides with a normal ghost: player dies
    fn check_collisions(&mut self) {
        for ghost in &mut self.ghosts {
            // Check if player and ghost are on the same tile
            if self.player.x == ghost.x && self.player.y == ghost.y {
                if ghost.vulnerable {
                    // Eat the ghost! Score increases with each ghost eaten
                    let multiplier_index = self.ghost_eaten_count.min(3) as usize;
                    self.score += SCORE_GHOST[multiplier_index];
                    self.ghost_eaten_count += 1;
                    
                    // Reset ghost to center
                    ghost.reset_to_center();
                } else {
                    // Player hit a normal ghost - game over
                    self.alive = false;
                    break;
                }
            }
        }
    }

    /// Draws the entire game frame
    /// 
    /// This function:
    /// 1. Updates the render cache
    /// 2. Draws the game (maze, player, ghosts, pellets)
    /// 3. Draws the score
    /// 4. Presents the frame to the screen
    pub fn draw(&mut self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) -> Result<(), String> {
        let (window_width, window_height) = canvas.window().size();
        
        // Update render cache (handles window resizing)
        self.render_cache.update_cache(window_width as i32, window_height as i32);
        
        // Prepare ghost data for rendering (position and vulnerability state)
        let ghost_data: Vec<(i32, i32, bool)> = self.ghosts.iter()
            .map(|ghost| (ghost.x, ghost.y, ghost.vulnerable))
            .collect();

        // Draw game elements (this clears the canvas)
        draw_game(
            canvas,
            &mut self.render_cache,
            &self.eaten,
            self.player.x,
            self.player.y,
            &ghost_data,
            self.power_pellet_timer,
            self.frame,
            self.alive,
        )?;
        
        // Draw score after game (so it appears on top)
        draw_score(
            canvas, 
            self.score, 
            self.render_cache.ox, 
            self.render_cache.oy, 
            self.render_cache.sw
        );
        
        // Present the frame to the screen
        canvas.present();
        Ok(())
    }
}

