// Player movement and input handling

use crate::constants::GRID_W;
use crate::maze::is_wall;
use crate::constants::TUNNEL_ROW;

pub struct Player {
    pub x: i32,
    pub y: i32,
    pub dx: i32,
    pub dy: i32,
    pub sub: i32, // Sub-frame counter for movement speed
    pub queued_dx: i32,
    pub queued_dy: i32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            x: 13,
            y: 23,
            dx: 0,
            dy: 0,
            sub: 0,
            queued_dx: 0,
            queued_dy: 0,
        }
    }

    // Process input immediately when key is pressed (instantaneous response)
    pub fn process_input(&mut self, dx: i32, dy: i32) {
        // Always update queued direction (for perpendicular turns)
        if dx != self.dx || dy != self.dy {
            self.queued_dx = dx;
            self.queued_dy = dy;
        }

        // Check if we can change direction immediately
        let is_aligned = self.sub == 0;
        let is_perpendicular = (dx != 0 && self.dy != 0) || (dy != 0 && self.dx != 0);
        let can_turn = (dx != self.dx || dy != self.dy) && 
                       !is_wall(self.x + dx, self.y + dy);
        let can_reverse = dx == -self.dx && dy == -self.dy;
        
        // Allow immediate turn if:
        // - Aligned to grid AND can turn, OR
        // - Perpendicular turn (90-degree) - allow even when not perfectly aligned, OR
        // - Reverse direction (180-degree) - always allowed
        if can_turn && (is_aligned || is_perpendicular || can_reverse) {
            self.dx = dx;
            self.dy = dy;
            // Clear queue if we successfully turned
            if dx == self.queued_dx && dy == self.queued_dy {
                self.queued_dx = 0;
                self.queued_dy = 0;
            }
        }
    }

    pub fn update(&mut self) {
        // Player moves every N sub-frames (integer speed) - 20% slower
        self.sub += 1;
        if self.sub >= 5 {
            self.sub = 0;
            
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
            
            let mut nx = self.x + self.dx;
            let ny = self.y + self.dy;
            // wrap tunnels on tunnel row
            if ny == TUNNEL_ROW && nx < 0 {
                nx = GRID_W - 1;
            }
            if ny == TUNNEL_ROW && nx >= GRID_W {
                nx = 0;
            }
            if !is_wall(nx, ny) {
                self.x = nx;
                self.y = ny;
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

