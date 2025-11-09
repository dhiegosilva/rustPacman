// Ghost AI and movement

use crate::constants::{GRID_W, TUNNEL_ROW};
use crate::maze::is_wall;
use crate::rng::Lfsr;

const DIRECTIONS: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];

pub struct Ghost {
    pub x: i32,
    pub y: i32,
    pub dx: i32,
    pub dy: i32,
    pub sub: i32, // Sub-frame counter for movement speed
    pub think_timer: i32,
    pub vulnerable: bool,
    pub opts_buffer: Vec<(i32, i32, i32)>, // Reusable buffer for AI
}

impl Ghost {
    pub fn new() -> Self {
        Self {
            x: 13,
            y: 14,
            dx: 0,
            dy: -1,
            sub: 0,
            think_timer: 0,
            vulnerable: false,
            opts_buffer: Vec::with_capacity(4),
        }
    }

    pub fn think(&mut self, player_x: i32, player_y: i32, rng: &mut Lfsr) {
        self.opts_buffer.clear();
        
        if self.vulnerable {
            // Flee: prefer direction away from player
            let dx_to_player = player_x - self.x;
            let dy_to_player = player_y - self.y;
            
            for (dx, dy) in DIRECTIONS.iter() {
                if !is_wall(self.x + dx, self.y + dy) && (*dx, *dy) != (-self.dx, -self.dy) {
                    let priority = if (*dx == 0 && dy_to_player.signum() == -*dy) ||
                                      (*dy == 0 && dx_to_player.signum() == -*dx) {
                        10
                    } else {
                        1
                    };
                    self.opts_buffer.push((*dx, *dy, priority));
                }
            }
            
            if self.opts_buffer.is_empty() {
                self.dx = -self.dx;
                self.dy = -self.dy;
                return;
            }
            
            self.opts_buffer.sort_by(|a, b| b.2.cmp(&a.2));
            let best_priority = self.opts_buffer[0].2;
            let best_count = self.opts_buffer.iter()
                .take_while(|opt| opt.2 == best_priority)
                .count();
            let i = rng.range(0, best_count as i32 - 1) as usize;
            let (dx, dy, _) = self.opts_buffer[i];
            self.dx = dx;
            self.dy = dy;
            return;
        }
        
        // Normal AI: choose a direction at junctions; avoid immediate reversal unless stuck
        for (dx, dy) in DIRECTIONS.iter() {
            if !is_wall(self.x + dx, self.y + dy) && (*dx, *dy) != (-self.dx, -self.dy) {
                self.opts_buffer.push((*dx, *dy, 0));
            }
        }

        if self.opts_buffer.is_empty() {
            self.dx = -self.dx;
            self.dy = -self.dy;
            return;
        }
        let i = rng.range(0, self.opts_buffer.len() as i32 - 1) as usize;
        let (dx, dy, _) = self.opts_buffer[i];
        self.dx = dx;
        self.dy = dy;
    }

    pub fn update(&mut self, rng: &mut Lfsr, player_x: i32, player_y: i32) {
        // Ghost AI
        self.think_timer += 1;
        if self.think_timer >= 8 {
            self.think(player_x, player_y, rng);
            self.think_timer = 0;
        }

        // Ghost moves - slower for Atari 2600 feel
        self.sub += 1;
        if self.sub >= 5 {
            self.sub = 0;
            let mut nx = self.x + self.dx;
            let ny = self.y + self.dy;
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
                self.dx = 0;
                self.dy = 0;
                self.think(player_x, player_y, rng);
            }
        }
    }

    pub fn reset_to_center(&mut self) {
        self.x = 13;
        self.y = 14;
        self.dx = 0;
        self.dy = -1;
    }
}

