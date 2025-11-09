// Maze logic and tile checking functions

use crate::constants::{GRID_W, GRID_H, MAZE};

#[inline]
pub fn is_wall(x: i32, y: i32) -> bool {
    if x < 0 || x >= GRID_W || y < 0 || y >= GRID_H {
        return true;
    }
    MAZE[y as usize].as_bytes()[x as usize] == b'#'
}

#[inline]
pub fn is_pellet(x: i32, y: i32) -> bool {
    if x < 0 || x >= GRID_W || y < 0 || y >= GRID_H {
        return false;
    }
    let c = MAZE[y as usize].as_bytes()[x as usize];
    c == b'.' || c == b'*'
}

#[inline]
pub fn is_power_pellet(x: i32, y: i32) -> bool {
    if x < 0 || x >= GRID_W || y < 0 || y >= GRID_H {
        return false;
    }
    MAZE[y as usize].as_bytes()[x as usize] == b'*'
}

#[inline]
pub fn is_empty(x: i32, y: i32) -> bool {
    if x < 0 || x >= GRID_W || y < 0 || y >= GRID_H {
        return false;
    }
    MAZE[y as usize].as_bytes()[x as usize] != b'#'
}

pub fn count_pellets() -> i32 {
    let mut count = 0;
    for y in 0..GRID_H {
        for x in 0..GRID_W {
            if is_pellet(x, y) {
                count += 1;
            }
        }
    }
    count
}

