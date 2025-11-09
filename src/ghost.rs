//! Ghost AI and movement
//! 
//! This module manages ghost behavior, including:
//! - AI decision making (normal and vulnerable states)
//! - Movement with sub-frame precision
//! - Tunnel wrapping

use crate::constants::{
    GRID_W, TUNNEL_ROW, 
    GHOST_START_X, GHOST_START_Y,
    GHOST_MOVE_SUBFRAMES, GHOST_THINK_INTERVAL
};
use crate::maze::is_wall;
use crate::rng::Lfsr;

/// All possible movement directions (up, down, left, right)
const MOVEMENT_DIRECTIONS: [(i32, i32); 4] = [
    (0, -1),  // Up
    (0, 1),   // Down
    (-1, 0),  // Left
    (1, 0),   // Right
];

/// Represents a ghost in the game
pub struct Ghost {
    /// Current X position on the grid
    pub x: i32,
    /// Current Y position on the grid
    pub y: i32,
    /// Current X direction (-1 = left, 0 = none, 1 = right)
    pub dx: i32,
    /// Current Y direction (-1 = up, 0 = none, 1 = down)
    pub dy: i32,
    /// Sub-frame counter: increments each frame, resets when ghost moves
    pub sub_frame_counter: i32,
    /// Timer that counts up to GHOST_THINK_INTERVAL before making AI decision
    pub think_timer: i32,
    /// Whether the ghost is vulnerable (can be eaten by player)
    pub vulnerable: bool,
    /// Reusable buffer for AI pathfinding (avoids allocations)
    pub options_buffer: Vec<(i32, i32, i32)>,  // (dx, dy, priority)
}

impl Ghost {
    /// Creates a new ghost at the default starting position
    pub fn new() -> Self {
        Self::new_at(GHOST_START_X, GHOST_START_Y)
    }

    /// Creates a new ghost at the specified position
    /// 
    /// # Arguments
    /// * `x` - Starting X position
    /// * `y` - Starting Y position
    pub fn new_at(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            dx: 0,
            dy: -1,  // Start moving up
            sub_frame_counter: 0,
            think_timer: 0,
            vulnerable: false,
            options_buffer: Vec::with_capacity(4),  // Max 4 directions
        }
    }

    /// Makes an AI decision about which direction to move
    /// 
    /// When vulnerable: tries to flee from the player
    /// When normal: randomly chooses a valid direction (avoids reversing unless stuck)
    /// 
    /// # Arguments
    /// * `player_x` - Player's X position
    /// * `player_y` - Player's Y position
    /// * `rng` - Random number generator for decision making
    pub fn think(&mut self, player_x: i32, player_y: i32, rng: &mut Lfsr) {
        self.options_buffer.clear();
        
        if self.vulnerable {
            // FLEE MODE: Try to move away from the player
            self.think_flee_mode(player_x, player_y, rng);
        } else {
            // NORMAL MODE: Randomly choose a direction
            self.think_normal_mode(rng);
        }
    }

    /// AI logic for when ghost is vulnerable (fleeing from player)
    fn think_flee_mode(&mut self, player_x: i32, player_y: i32, rng: &mut Lfsr) {
        let distance_to_player_x = player_x - self.x;
        let distance_to_player_y = player_y - self.y;
        
        // Check all possible directions
        for (dx, dy) in MOVEMENT_DIRECTIONS.iter() {
            let new_x = self.x + dx;
            let new_y = self.y + dy;
            
            // Skip if it's a wall or would reverse direction
            if is_wall(new_x, new_y) || (*dx, *dy) == (-self.dx, -self.dy) {
                continue;
            }
            
            // Calculate priority: higher priority for directions that move away from player
            let priority = if (*dx == 0 && distance_to_player_y.signum() == -*dy) ||
                              (*dy == 0 && distance_to_player_x.signum() == -*dx) {
                10  // High priority: moves directly away from player
            } else {
                1   // Low priority: other valid directions
            };
            
            self.options_buffer.push((*dx, *dy, priority));
        }
        
        // If no valid directions, reverse (only option)
        if self.options_buffer.is_empty() {
            self.dx = -self.dx;
            self.dy = -self.dy;
            return;
        }
        
        // Sort by priority (highest first)
        self.options_buffer.sort_by(|a, b| b.2.cmp(&a.2));
        
        // Choose randomly from the best priority options
        let best_priority = self.options_buffer[0].2;
        let best_options_count = self.options_buffer.iter()
            .take_while(|option| option.2 == best_priority)
            .count();
        
        let random_index = rng.range(0, best_options_count as i32 - 1) as usize;
        let (dx, dy, _) = self.options_buffer[random_index];
        self.dx = dx;
        self.dy = dy;
    }

    /// AI logic for when ghost is normal (random movement)
    fn think_normal_mode(&mut self, rng: &mut Lfsr) {
        // Check all possible directions
        for (dx, dy) in MOVEMENT_DIRECTIONS.iter() {
            let new_x = self.x + dx;
            let new_y = self.y + dy;
            
            // Skip if it's a wall or would reverse direction (unless stuck)
            if !is_wall(new_x, new_y) && (*dx, *dy) != (-self.dx, -self.dy) {
                self.options_buffer.push((*dx, *dy, 0));  // All have same priority
            }
        }

        // If no valid directions, reverse (only option when stuck)
        if self.options_buffer.is_empty() {
            self.dx = -self.dx;
            self.dy = -self.dy;
            return;
        }
        
        // Choose a random valid direction
        let random_index = rng.range(0, self.options_buffer.len() as i32 - 1) as usize;
        let (dx, dy, _) = self.options_buffer[random_index];
        self.dx = dx;
        self.dy = dy;
    }

    /// Updates the ghost each frame
    /// 
    /// This function:
    /// 1. Updates the think timer and makes AI decisions periodically
    /// 2. Moves the ghost when enough sub-frames have passed
    /// 3. Handles tunnel wrapping
    /// 4. Re-thinks if hitting a wall
    /// 
    /// # Arguments
    /// * `rng` - Random number generator
    /// * `player_x` - Player's X position (for AI)
    /// * `player_y` - Player's Y position (for AI)
    pub fn update(&mut self, rng: &mut Lfsr, player_x: i32, player_y: i32) {
        // Update AI decision timer
        self.think_timer += 1;
        if self.think_timer >= GHOST_THINK_INTERVAL {
            self.think(player_x, player_y, rng);
            self.think_timer = 0;
        }

        // Move ghost when enough sub-frames have passed
        self.sub_frame_counter += 1;
        if self.sub_frame_counter >= GHOST_MOVE_SUBFRAMES {
            self.sub_frame_counter = 0;
            
            // Calculate new position
            let mut new_x = self.x + self.dx;
            let mut new_y = self.y + self.dy;
            
            // Handle tunnel wrapping
            if new_y == TUNNEL_ROW && new_x < 0 {
                new_x = GRID_W - 1;
            }
            if new_y == TUNNEL_ROW && new_x >= GRID_W {
                new_x = 0;
            }
            
            // Move if there's no wall, otherwise stop and re-think
            if !is_wall(new_x, new_y) {
                self.x = new_x;
                self.y = new_y;
            } else {
                // Hit a wall, stop and make a new decision
                self.dx = 0;
                self.dy = 0;
                self.think(player_x, player_y, rng);
            }
        }
    }

    /// Resets the ghost to the center starting position
    /// 
    /// Called when the ghost is eaten by the player
    pub fn reset_to_center(&mut self) {
        self.x = GHOST_START_X;
        self.y = GHOST_START_Y;
        self.dx = 0;
        self.dy = -1;  // Start moving up
    }
    
    /// Processes input for player-controlled ghost
    /// 
    /// Similar to player input processing - allows direction changes
    pub fn process_input(&mut self, dx: i32, dy: i32) {
        // Check if we can change direction
        let can_turn = (dx != self.dx || dy != self.dy) && 
                       !is_wall(self.x + dx, self.y + dy);
        let is_reverse_turn = dx == -self.dx && dy == -self.dy;
        let is_aligned = self.sub_frame_counter == 0;
        
        // Allow turn if aligned and can turn, or reverse direction
        if can_turn && (is_aligned || is_reverse_turn) {
            self.dx = dx;
            self.dy = dy;
        }
    }
    
    /// Updates only movement for player-controlled ghost (no AI)
    pub fn update_movement_only(&mut self) {
        // Move ghost when enough sub-frames have passed
        self.sub_frame_counter += 1;
        if self.sub_frame_counter >= GHOST_MOVE_SUBFRAMES {
            self.sub_frame_counter = 0;
            
            // Calculate new position
            let mut new_x = self.x + self.dx;
            let mut new_y = self.y + self.dy;
            
            // Handle tunnel wrapping
            if new_y == TUNNEL_ROW && new_x < 0 {
                new_x = GRID_W - 1;
            }
            if new_y == TUNNEL_ROW && new_x >= GRID_W {
                new_x = 0;
            }
            
            // Move if there's no wall, otherwise stop
            if !is_wall(new_x, new_y) {
                self.x = new_x;
                self.y = new_y;
            } else {
                // Hit a wall, stop
                self.dx = 0;
                self.dy = 0;
            }
        }
    }
}

