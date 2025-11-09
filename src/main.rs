//! Main entry point for the Pac-Man game
//! 
//! This module handles:
//! - SDL2 initialization
//! - Window creation
//! - Event loop (input handling)
//! - Game loop with fixed timestep
//! - Menu and game state management

use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use std::time::{Duration, Instant};
use paclike_2600_rs::game::Game;
use paclike_2600_rs::menu::{Menu, MenuAction};
use paclike_2600_rs::audio::AudioManager;
use paclike_2600_rs::game_config::{GameConfig, GameMode, PlayerRole};
use paclike_2600_rs::constants::{
    VIEW_W, VIEW_H, SCORE_AREA, WINDOW_SCALE, DT, 
    MAZE_1, MAZE_2, CURRENT_MAZE,
    MAX_TIME_STEP, SLEEP_DURATION_MS
};

fn main() -> Result<(), String> {
    // Init SDL
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    
    // Calculate window size to fit game content with minimal borders
    let window_w = (VIEW_W * WINDOW_SCALE) as u32;
    let window_h = ((VIEW_H + SCORE_AREA) * WINDOW_SCALE) as u32;
    
    let window = video
        .window("Pacman - Atari 2600 Style (Rust)", window_w, window_h)
        .position_centered()
        .opengl()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window
        .into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    // Initialize audio
    let _audio_manager = AudioManager::new(&sdl)?;
    
    // Initialize game state
    let mut event_pump = sdl.event_pump()?;
    let mut menu = Menu::new();
    let mut game: Option<Game> = None;
    let mut time_accumulator = 0.0f64;  // Accumulates time for fixed timestep
    let mut previous_frame_time = Instant::now();
    let delta_time = DT;
    let mut in_menu = true;

    'main_loop: loop {
        // Process ALL events immediately - instantaneous input response
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main_loop,
                
                // Escape key: exit game or return to menu
                Event::KeyDown { scancode: Some(Scancode::Escape), .. } => {
                    if in_menu {
                        break 'main_loop;  // Exit game
                    } else {
                        in_menu = true;  // Return to menu
                        game = None;
                    }
                }
                
                // Arrow keys: navigate menu or move player
                Event::KeyDown { scancode: Some(Scancode::Up), .. } => {
                    if in_menu {
                        menu.process_input(0, -1);  // Move menu selection up
                    } else if let Some(ref mut current_game) = game {
                        current_game.process_input(0, -1);  // Move player up
                    }
                }
                Event::KeyDown { scancode: Some(Scancode::Down), .. } => {
                    if in_menu {
                        menu.process_input(0, 1);  // Move menu selection down
                    } else if let Some(ref mut current_game) = game {
                        current_game.process_input(0, 1);  // Move player down
                    }
                }
                Event::KeyDown { scancode: Some(Scancode::Left), .. } => {
                    if !in_menu {
                        if let Some(ref mut current_game) = game {
                            current_game.process_input(-1, 0);  // Move player left
                        }
                    }
                }
                Event::KeyDown { scancode: Some(Scancode::Right), .. } => {
                    if !in_menu {
                        if let Some(ref mut current_game) = game {
                            current_game.process_input(1, 0);  // Move player right
                        }
                    }
                }
                
                // Backspace: go back in menu
                Event::KeyDown { scancode: Some(Scancode::Backspace), .. } => {
                    if in_menu {
                        menu.back();
                    }
                }
                
                // Enter key: select menu option
                Event::KeyDown { scancode: Some(Scancode::Return), .. } => {
                    if in_menu {
                        match menu.select() {
                            MenuAction::SelectMaze(maze_index) => {
                                // Switch to selected maze
                                unsafe {
                                    CURRENT_MAZE = match maze_index {
                                        0 => &MAZE_1 as *const _,
                                        1 => &MAZE_2 as *const _,
                                        _ => &MAZE_1 as *const _,  // Default to maze 1
                                    };
                                }
                                
                                // Create game config from menu selections
                                let config = GameConfig::new(
                                    menu.game_mode.unwrap_or(GameMode::SinglePlayer),
                                    menu.player1_role.unwrap_or(PlayerRole::PacMan),
                                    menu.player2_role,
                                );
                                
                                game = Some(Game::new(config));
                                in_menu = false;
                            }
                            _ => {}
                        }
                    }
                }
                
                // Window resize: update render cache
                Event::Window { win_event, .. } => {
                    if matches!(win_event, sdl2::event::WindowEvent::Resized(_, _) | 
                                       sdl2::event::WindowEvent::SizeChanged(_, _)) {
                        if let Some(ref mut current_game) = game {
                            current_game.render_cache.window_size_changed = true;
                        }
                    }
                }
                _ => {}
            }
        }

        if in_menu {
            // Draw menu screen
            menu.draw(&mut canvas)?;
        } else {
            // Fixed timestep game loop
            // This ensures the game runs at a consistent speed regardless of frame rate
            let current_time = Instant::now();
            let frame_duration = current_time.duration_since(previous_frame_time);
            previous_frame_time = current_time;
            
            // Add elapsed time to accumulator (clamped to prevent spiral of death)
            time_accumulator += (frame_duration.as_secs_f64()).min(MAX_TIME_STEP);

            // Get keyboard state for held keys (fallback input)
            let keyboard_state = event_pump.keyboard_state();
            
            // Check for player 2 input (WASD keys for multiplayer)
            let player2_input = if game.as_ref().map(|g| g.config.mode == GameMode::Multiplayer).unwrap_or(false) {
                let mut input = None;
                if keyboard_state.is_scancode_pressed(Scancode::W) {
                    input = Some((0, -1));
                } else if keyboard_state.is_scancode_pressed(Scancode::S) {
                    input = Some((0, 1));
                } else if keyboard_state.is_scancode_pressed(Scancode::A) {
                    input = Some((-1, 0));
                } else if keyboard_state.is_scancode_pressed(Scancode::D) {
                    input = Some((1, 0));
                }
                input
            } else {
                None
            };
            
            if let Some(ref mut current_game) = game {
                // Run game updates until we've caught up with real time
                while time_accumulator >= delta_time {
                    if current_game.alive {
                        current_game.tick(&keyboard_state, player2_input);
                    }
                    time_accumulator -= delta_time;
                }

                // Draw the game
                current_game.draw(&mut canvas)?;
            }
        }
        
        // Small sleep to reduce CPU usage when vsync is off
        std::thread::sleep(Duration::from_millis(SLEEP_DURATION_MS));
    }
    Ok(())
}
