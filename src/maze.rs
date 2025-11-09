// Maze logic and tile checking functions

use crate::constants::{GRID_W, GRID_H};

#[inline]
pub fn get_maze() -> &'static [&'static str] {
    unsafe { 
        let arr_ref: &[&str; crate::constants::GRID_H as usize] = &*crate::constants::CURRENT_MAZE;
        arr_ref as &[&str]
    }
}

#[inline]
pub fn is_wall(x: i32, y: i32) -> bool {
    if x < 0 || x >= GRID_W || y < 0 || y >= GRID_H {
        return true;
    }
    let maze = get_maze();
    let y_idx = y as usize;
    if y_idx >= maze.len() {
        return true;
    }
    let row = maze[y_idx];
    if x as usize >= row.len() {
        return true;
    }
    row.as_bytes()[x as usize] == b'#'
}

#[inline]
pub fn is_pellet(x: i32, y: i32) -> bool {
    if x < 0 || x >= GRID_W || y < 0 || y >= GRID_H {
        return false;
    }
    let maze = get_maze();
    let y_idx = y as usize;
    if y_idx >= maze.len() {
        return false;
    }
    let row = maze[y_idx];
    if x as usize >= row.len() {
        return false;
    }
    let c = row.as_bytes()[x as usize];
    c == b'.' || c == b'*'
}

#[inline]
pub fn is_power_pellet(x: i32, y: i32) -> bool {
    if x < 0 || x >= GRID_W || y < 0 || y >= GRID_H {
        return false;
    }
    let maze = get_maze();
    let y_idx = y as usize;
    if y_idx >= maze.len() {
        return false;
    }
    let row = maze[y_idx];
    if x as usize >= row.len() {
        return false;
    }
    row.as_bytes()[x as usize] == b'*'
}

#[inline]
pub fn is_empty(x: i32, y: i32) -> bool {
    if x < 0 || x >= GRID_W || y < 0 || y >= GRID_H {
        return false;
    }
    let maze = get_maze();
    let y_idx = y as usize;
    if y_idx >= maze.len() {
        return false;
    }
    let row = maze[y_idx];
    if x as usize >= row.len() {
        return false;
    }
    row.as_bytes()[x as usize] != b'#'
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

