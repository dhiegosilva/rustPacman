// Main entry point

use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use std::time::{Duration, Instant};
use paclike_2600_rs::game::Game;
use paclike_2600_rs::menu::{Menu, MenuAction};
use paclike_2600_rs::constants::{VIEW_W, VIEW_H, SCORE_AREA, WINDOW_SCALE, DT, MAZE_1, MAZE_2, CURRENT_MAZE};

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

    // Main loop
    let mut event_pump = sdl.event_pump()?;
    let mut menu = Menu::new();
    let mut game: Option<Game> = None;
    let mut acc = 0.0f64;
    let mut prev = Instant::now();
    let dt = DT;
    let mut in_menu = true;

    'running: loop {
        // Process ALL events immediately - instantaneous input response
        for e in event_pump.poll_iter() {
            match e {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { scancode: Some(Scancode::Escape), .. } => {
                    if in_menu {
                        break 'running;
                    } else {
                        in_menu = true;
                        game = None;
                    }
                }
                Event::KeyDown { scancode: Some(Scancode::Up), .. } => {
                    if in_menu {
                        menu.process_input(0, -1);
                    } else if let Some(ref mut g) = game {
                        g.process_input(0, -1);
                    }
                }
                Event::KeyDown { scancode: Some(Scancode::Down), .. } => {
                    if in_menu {
                        menu.process_input(0, 1);
                    } else if let Some(ref mut g) = game {
                        g.process_input(0, 1);
                    }
                }
                Event::KeyDown { scancode: Some(Scancode::Left), .. } => {
                    if !in_menu {
                        if let Some(ref mut g) = game {
                            g.process_input(-1, 0);
                        }
                    }
                }
                Event::KeyDown { scancode: Some(Scancode::Right), .. } => {
                    if !in_menu {
                        if let Some(ref mut g) = game {
                            g.process_input(1, 0);
                        }
                    }
                }
                Event::KeyDown { scancode: Some(Scancode::Return), .. } => {
                    if in_menu {
                        match menu.select() {
                            MenuAction::SelectMaze(idx) => {
                                unsafe {
                                    CURRENT_MAZE = match idx {
                                        0 => &MAZE_1 as *const _,
                                        1 => &MAZE_2 as *const _,
                                        _ => &MAZE_1 as *const _,
                                    };
                                }
                                game = Some(Game::new());
                                in_menu = false;
                            }
                            _ => {}
                        }
                    }
                }
                Event::Window { win_event, .. } => {
                    if matches!(win_event, sdl2::event::WindowEvent::Resized(_, _) | 
                                       sdl2::event::WindowEvent::SizeChanged(_, _)) {
                        if let Some(ref mut g) = game {
                            g.render_cache.window_size_changed = true;
                        }
                    }
                }
                _ => {}
            }
        }

        if in_menu {
            menu.draw(&mut canvas)?;
        } else {
            // Fixed-step accumulator
            let now = Instant::now();
            let elapsed = now.duration_since(prev);
            prev = now;
            acc += (elapsed.as_secs_f64()).min(0.25); // avoid spiral

            let keys = event_pump.keyboard_state();
            if let Some(ref mut g) = game {
                while acc >= dt {
                    if g.alive {
                        g.tick(&keys);
                    }
                    acc -= dt;
                }
                g.draw(&mut canvas)?;
            }
        }
        // Small sleep to reduce CPU if vsync off
        std::thread::sleep(Duration::from_millis(1));
    }
    Ok(())
}
