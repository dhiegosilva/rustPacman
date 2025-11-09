//! Maze logic and tile checking functions
//! 
//! This module provides functions to check what's at specific positions in the maze:
//! - Walls (#)
//! - Pellets (.)
//! - Power pellets (*)
//! - Teleporters (1)
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

/// Checks if the given position contains a teleporter ('1' through '9')
/// 
/// # Arguments
/// * `x` - X coordinate (0 to GRID_W-1)
/// * `y` - Y coordinate (0 to GRID_H-1)
/// 
/// # Returns
/// `true` if the position contains a teleporter digit ('1'-'9'), `false` otherwise
#[inline]
pub fn is_teleporter(x: i32, y: i32) -> bool {
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
    let tile = row.as_bytes()[x as usize];
    // Check if tile is a digit from '1' to '9'
    tile >= b'1' && tile <= b'9'
}

/// Gets the teleporter digit at the given position
/// 
/// # Arguments
/// * `x` - X coordinate (0 to GRID_W-1)
/// * `y` - Y coordinate (0 to GRID_H-1)
/// 
/// # Returns
/// `Some(digit)` if the position contains a teleporter ('1'-'9'), `None` otherwise
#[inline]
pub fn get_teleporter_digit(x: i32, y: i32) -> Option<u8> {
    if x < 0 || x >= GRID_W || y < 0 || y >= GRID_H {
        return None;
    }
    let maze = get_maze();
    let y_idx = y as usize;
    if y_idx >= maze.len() {
        return None;
    }
    let row = maze[y_idx];
    if x as usize >= row.len() {
        return None;
    }
    let tile = row.as_bytes()[x as usize];
    // Return the digit if it's between '1' and '9'
    if tile >= b'1' && tile <= b'9' {
        Some(tile)
    } else {
        None
    }
}

/// Finds the other teleporter position with the same digit
/// 
/// When the player is on a teleporter, this function finds the other teleporter
/// in the maze with the same digit (e.g., '1' teleports to '1', '2' to '2', etc.)
/// 
/// # Arguments
/// * `current_x` - Current X position (the teleporter the player is on)
/// * `current_y` - Current Y position (the teleporter the player is on)
/// 
/// # Returns
/// `Some((x, y))` if another teleporter with the same digit is found, `None` if not found
pub fn find_other_teleporter(current_x: i32, current_y: i32) -> Option<(i32, i32)> {
    // Get the digit of the current teleporter
    let current_digit = match get_teleporter_digit(current_x, current_y) {
        Some(d) => d,
        None => return None,  // Not on a teleporter
    };
    
    // Search for another teleporter with the same digit
    for y in 0..GRID_H {
        for x in 0..GRID_W {
            // Skip the current position
            if x == current_x && y == current_y {
                continue;
            }
            // Check if this position has the same teleporter digit
            if let Some(digit) = get_teleporter_digit(x, y) {
                if digit == current_digit {
                    return Some((x, y));
                }
            }
        }
    }
    None
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

