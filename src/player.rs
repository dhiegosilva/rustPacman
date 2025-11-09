//! Player movement and input handling
//! 
//! This module manages the player's position, movement, and input processing.
//! It handles grid-based movement with sub-frame precision for smooth animation.

use crate::constants::{GRID_W, TUNNEL_ROW, PLAYER_START_X, PLAYER_START_Y, PLAYER_MOVE_SUBFRAMES};
use crate::maze::{is_wall, is_teleporter, find_other_teleporter};

/// Represents the player (Pac-Man) in the game
pub struct Player {
    /// Current X position on the grid
    pub x: i32,
    /// Current Y position on the grid
    pub y: i32,
    /// Current X direction (-1 = left, 0 = none, 1 = right)
    pub dx: i32,
    /// Current Y direction (-1 = up, 0 = none, 1 = down)
    pub dy: i32,
    /// Sub-frame counter: increments each frame, resets when player moves
    pub sub_frame_counter: i32,
    /// Queued X direction (for perpendicular turns that can't happen immediately)
    pub queued_dx: i32,
    /// Queued Y direction (for perpendicular turns that can't happen immediately)
    pub queued_dy: i32,
}

impl Player {
    /// Creates a new player at the starting position
    pub fn new() -> Self {
        Self {
            x: PLAYER_START_X,
            y: PLAYER_START_Y,
            dx: 0,
            dy: 0,
            sub_frame_counter: 0,
            queued_dx: 0,
            queued_dy: 0,
        }
    }

    /// Processes input from the keyboard
    /// 
    /// This function is called immediately when a key is pressed for instant response.
    /// It handles three types of turns:
    /// - Immediate turns: When aligned to grid and path is clear
    /// - Perpendicular turns: 90-degree turns (can happen even when not perfectly aligned)
    /// - Reverse turns: 180-degree turns (always allowed)
    /// 
    /// If a turn can't happen immediately, it's queued for when the player aligns to the grid.
    /// 
    /// # Arguments
    /// * `dx` - Desired X direction (-1 = left, 0 = none, 1 = right)
    /// * `dy` - Desired Y direction (-1 = up, 0 = none, 1 = down)
    pub fn process_input(&mut self, dx: i32, dy: i32) {
        // Always update queued direction (for perpendicular turns)
        if dx != self.dx || dy != self.dy {
            self.queued_dx = dx;
            self.queued_dy = dy;
        }

        // Check if we can change direction immediately
        let is_aligned_to_grid = self.sub_frame_counter == 0;
        let is_perpendicular_turn = (dx != 0 && self.dy != 0) || (dy != 0 && self.dx != 0);
        let can_turn = (dx != self.dx || dy != self.dy) && 
                       !is_wall(self.x + dx, self.y + dy);
        let is_reverse_turn = dx == -self.dx && dy == -self.dy;
        
        // Allow immediate turn if:
        // - Aligned to grid AND can turn, OR
        // - Perpendicular turn (90-degree) - allow even when not perfectly aligned, OR
        // - Reverse direction (180-degree) - always allowed
        if can_turn && (is_aligned_to_grid || is_perpendicular_turn || is_reverse_turn) {
            self.dx = dx;
            self.dy = dy;
            // Clear queue if we successfully turned
            if dx == self.queued_dx && dy == self.queued_dy {
                self.queued_dx = 0;
                self.queued_dy = 0;
            }
        }
    }

    /// Updates the player's position each frame
    /// 
    /// This function:
    /// 1. Increments the sub-frame counter
    /// 2. When enough sub-frames have passed, moves the player
    /// 3. Checks for queued direction changes
    /// 4. Handles tunnel wrapping
    /// 5. Handles teleportation (if on a '1' tile)
    /// 6. Stops movement if hitting a wall
    pub fn update(&mut self) {
        // Increment sub-frame counter
        self.sub_frame_counter += 1;
        
        // Only move when enough sub-frames have passed (controls movement speed)
        if self.sub_frame_counter >= PLAYER_MOVE_SUBFRAMES {
            self.sub_frame_counter = 0;
            
            // Check queued direction when aligned (for perpendicular turns that were queued)
            if self.queued_dx != 0 || self.queued_dy != 0 {
                if !is_wall(self.x + self.queued_dx, self.y + self.queued_dy) {
                    self.dx = self.queued_dx;
                    self.dy = self.queued_dy;
                    self.queued_dx = 0;
                    self.queued_dy = 0;
                } else {
                    // Can't turn to queued direction, clear queue
                    self.queued_dx = 0;
                    self.queued_dy = 0;
                }
            }
            
            // Calculate new position
            let mut new_x = self.x + self.dx;
            let mut new_y = self.y + self.dy;
            
            // Handle tunnel wrapping: if on tunnel row and going off-screen, wrap to other side
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
                
                // Check for teleportation: if player is on a teleporter, teleport to the other one
                if is_teleporter(self.x, self.y) {
                    if let Some((teleport_x, teleport_y)) = find_other_teleporter(self.x, self.y) {
                        self.x = teleport_x;
                        self.y = teleport_y;
                    }
                }
            } else {
                // Hit a wall, stop and clear queue
                self.dx = 0;
                self.dy = 0;
                self.queued_dx = 0;
                self.queued_dy = 0;
            }
        }
    }
}

