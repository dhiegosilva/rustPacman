//! Maze logic and tile checking functions
//! 
//! This module provides functions to check what's at specific positions in the maze:
//! - Walls (#)
//! - Pellets (.)
//! - Power pellets (*)
//! - Empty spaces

use crate::constants::{GRID_W, GRID_H};

/// Gets a reference to the currently selected maze
/// 
/// Returns the maze as a slice of strings, where each string is a row.
/// Each character represents a tile: '#' = wall, '.' = pellet, '*' = power pellet
#[inline]
pub fn get_maze() -> &'static [&'static str] {
    unsafe { 
        let arr_ref: &[&str; crate::constants::GRID_H as usize] = &*crate::constants::CURRENT_MAZE;
        arr_ref as &[&str]
    }
}

/// Checks if the given position contains a wall
/// 
/// # Arguments
/// * `x` - X coordinate (0 to GRID_W-1)
/// * `y` - Y coordinate (0 to GRID_H-1)
/// 
/// # Returns
/// `true` if the position is out of bounds or contains a wall ('#'), `false` otherwise
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

/// Checks if the given position contains a pellet (regular or power)
/// 
/// # Arguments
/// * `x` - X coordinate (0 to GRID_W-1)
/// * `y` - Y coordinate (0 to GRID_H-1)
/// 
/// # Returns
/// `true` if the position contains '.' (pellet) or '*' (power pellet), `false` otherwise
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

/// Checks if the given position contains a power pellet
/// 
/// # Arguments
/// * `x` - X coordinate (0 to GRID_W-1)
/// * `y` - Y coordinate (0 to GRID_H-1)
/// 
/// # Returns
/// `true` if the position contains '*' (power pellet), `false` otherwise
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

/// Checks if the given position is empty (not a wall)
/// 
/// # Arguments
/// * `x` - X coordinate (0 to GRID_W-1)
/// * `y` - Y coordinate (0 to GRID_H-1)
/// 
/// # Returns
/// `true` if the position is not a wall, `false` if it's a wall or out of bounds
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

/// Counts the total number of pellets (regular + power) in the current maze
/// 
/// # Returns
/// The total number of pellets in the maze
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

