// Main entry point

use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use std::time::{Duration, Instant};
use paclike_2600_rs::game::Game;
use paclike_2600_rs::constants::{VIEW_W, VIEW_H, SCORE_AREA, WINDOW_SCALE, DT};

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
    let mut game = Game::new();
    let mut acc = 0.0f64;
    let mut prev = Instant::now();
    let dt = DT;

    'running: loop {
        // Process ALL events immediately - instantaneous input response
        // No queuing, no waiting - process input the moment the key is pressed
        for e in event_pump.poll_iter() {
            match e {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { scancode: Some(Scancode::Escape), .. } => break 'running,
                Event::KeyDown { scancode: Some(Scancode::Up), .. } => {
                    // Process input INSTANTLY - no delay, no queuing
                    game.process_input(0, -1);
                }
                Event::KeyDown { scancode: Some(Scancode::Down), .. } => {
                    game.process_input(0, 1);
                }
                Event::KeyDown { scancode: Some(Scancode::Left), .. } => {
                    game.process_input(-1, 0);
                }
                Event::KeyDown { scancode: Some(Scancode::Right), .. } => {
                    game.process_input(1, 0);
                }
                Event::Window { win_event, .. } => {
                    // Mark window size changed on resize
                    if matches!(win_event, sdl2::event::WindowEvent::Resized(_, _) | 
                                       sdl2::event::WindowEvent::SizeChanged(_, _)) {
                        game.render_cache.window_size_changed = true;
                    }
                }
                _ => {}
            }
        }

        // Fixed-step accumulator
        let now = Instant::now();
        let elapsed = now.duration_since(prev);
        prev = now;
        acc += (elapsed.as_secs_f64()).min(0.25); // avoid spiral

        let keys = event_pump.keyboard_state();
        while acc >= dt {
            if game.alive {
                game.tick(&keys);
            }
            acc -= dt;
        }

        game.draw(&mut canvas)?;
        // Small sleep to reduce CPU if vsync off
        std::thread::sleep(Duration::from_millis(1));
    }
    Ok(())
}
