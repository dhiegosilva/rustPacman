//! Game constants and configuration
//! 
//! This module contains all the game's configuration values, making it easy
//! to understand and modify game behavior.

/// Width of the game grid in tiles
pub const GRID_W: i32 = 28;

/// Height of the game grid in tiles
pub const GRID_H: i32 = 31;

/// Size of each tile in pixels (6 pixels per tile)
pub const TILE: i32 = 6;

/// Total width of the game view in pixels
pub const VIEW_W: i32 = GRID_W * TILE;

/// Total height of the game view in pixels
pub const VIEW_H: i32 = GRID_H * TILE;

/// Height of the score area at the top of the screen in pixels
pub const SCORE_AREA: i32 = 30;

/// Target frames per second
pub const FPS: u32 = 60;

/// Delta time (time per frame) in seconds
pub const DT: f64 = 1.0 / FPS as f64;

/// Scale factor for window size (makes window 4x larger than game view)
pub const WINDOW_SCALE: i32 = 4;

/// Row number where tunnels wrap around (0-indexed, row 14)
pub const TUNNEL_ROW: i32 = 14;

// ============================================================================
// Player Constants
// ============================================================================

/// Player's starting X position (center of maze)
pub const PLAYER_START_X: i32 = 13;

/// Player's starting Y position
pub const PLAYER_START_Y: i32 = 23;

/// Number of sub-frames before player moves (controls speed: higher = slower)
/// Player moves every 5 sub-frames, making them 20% slower than original
pub const PLAYER_MOVE_SUBFRAMES: i32 = 5;

// ============================================================================
// Ghost Constants
// ============================================================================

/// Ghost's starting X position (center of maze)
pub const GHOST_START_X: i32 = 13;

/// Ghost's starting Y position
pub const GHOST_START_Y: i32 = 14;

/// Number of sub-frames before ghost moves (controls speed: higher = slower)
/// Ghosts move every 6 sub-frames, making them 20% slower than original
pub const GHOST_MOVE_SUBFRAMES: i32 = 6;

/// Number of frames between ghost AI decisions
pub const GHOST_THINK_INTERVAL: i32 = 8;

// ============================================================================
// Scoring Constants
// ============================================================================

/// Points awarded for eating a regular pellet
pub const SCORE_PELLET: i32 = 10;

/// Points awarded for eating a power pellet
pub const SCORE_POWER_PELLET: i32 = 50;

/// Points awarded for eating ghosts (multiplier increases with each ghost)
/// First ghost: 200, Second: 400, Third: 800, Fourth: 1600
pub const SCORE_GHOST: [i32; 4] = [200, 400, 800, 1600];

/// Duration of power pellet effect in frames (900 frames = ~15 seconds at 60 FPS)
pub const POWER_PELLET_DURATION: i32 = 900;

/// Number of frames before power pellet expires when it starts flashing
pub const POWER_PELLET_FLASH_START: i32 = 120;

// ============================================================================
// Animation Constants
// ============================================================================

/// Number of frames per Pac-Man mouth animation frame
pub const PACMAN_MOUTH_ANIMATION_SPEED: u32 = 8;

/// Number of frames per ghost wavy bottom animation frame
pub const GHOST_WAVE_ANIMATION_SPEED: u32 = 10;

/// Number of frames per power pellet flash cycle
pub const POWER_PELLET_FLASH_SPEED: u32 = 15;

// ============================================================================
// Game Loop Constants
// ============================================================================

/// Maximum time step to prevent spiral of death (0.25 seconds)
pub const MAX_TIME_STEP: f64 = 0.25;

/// Sleep duration in milliseconds to reduce CPU usage when vsync is off
pub const SLEEP_DURATION_MS: u64 = 1;

// Maze 1: Original Atari 2600 layout
pub static MAZE_1: [&str; GRID_H as usize] = [
    "############################",
    "#............##............#",
    "#.####.#####.##.#####.####.#",
    "#*####.#####.##.#####.####*#",
    "#.####.#####.##.#####.####.#",
    "#..........................#",
    "#.####.##.########.##.####.#",
    "#.####.##.########.##.####.#",
    "#......##....##....##......#",
    "######.##### ## #####.######",
    "#####..##### ## #####..#####",
    "#####.##            ##.#####",
    "#......# ### ## ### #......#",
    "######.# #        # #.######",
    "     #.# #  ####  # #.#     ",
    "######.# #        # #.######",
    "#......# ########## #......#",
    "#####.##            ##.#####",
    "#####..##### ## #####..#####",
    "######.##### ## #####.######",
    "#......##....##....##......#",
    "#.####.##.########.##.####.#",
    "#.####.##.########.##.####.#",
    "#...##................##...#",
    "###.##.####.##.####.##.###.#",
    "#*..   ####.##.####   ..*..#",
    "###.##.####.##.####.##.###.#",
    "#...##................##...#",
    "#.##########.##.##########.#",
    "#..........................#",
    "############################",
];

// Maze 2: Simpler layout (based on classic, simplified)
pub static MAZE_2: [&str; GRID_H as usize] = [
    "############1###############",
    "#............##............#",
    "#.####.#####.##.#####.####.#",
    "#*####.#####.##.#####.####*#",
    "#.####.#####.##.#####.####.#",
    "#..........................#",
    "#.####.##.########.##.####.#",
    "#.####.##.########.##.####.#",
    "#......##....##....##......#",
    "######.##### ## #####.######",
    "#####..##### ## #####..#####",
    "#####.##            ##.#####",
    "#......#            #......#",
    "## ###.#            #.######",
    "2    #.#            #.#    2",
    "######.#            #.#### #",
    "#......#            #......#",
    "#####.##            ##.#####",
    "#####..##### ## #####..#####",
    "######.##### ## #####.######",
    "#......##....##....##......#",
    "#.####.##.########.##.####.#",
    "#.####.##.########.##.####.#",
    "#...##................##...#",
    "###.##.####.##.####.######.#",
    "#*..   ####.##.####   ..*..#",
    "###.##.####.##.####.######.#",
    "#...##................##...#",
    "#.##########.##.##########.#",
    "#..........................#",
    "############1###############",
];

// Current maze (will be set based on selection)
pub static mut CURRENT_MAZE: *const [&str; GRID_H as usize] = &MAZE_1 as *const _;

