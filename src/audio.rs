//! Audio module for background music
//! 
//! Generates simple chiptune music combining Tetris-style melodies with Zelda-style harmonies

use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};

/// Simple audio callback that generates chiptune music
/// 
/// Combines Tetris-style melody with Zelda-style chord progressions
struct MusicGenerator {
    sample_rate: i32,
    time: f32,
    melody_time: f32,
    bass_time: f32,
}

impl AudioCallback for MusicGenerator {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Tetris theme notes (Korobeiniki melody) - simplified
        // C, E, G, C, G, E, C, G, E, C, G, E, C, E, G, C
        let tetris_melody = [
            261.63, 329.63, 392.00, 523.25, 392.00, 329.63, 261.63, 392.00,
            329.63, 261.63, 392.00, 329.63, 261.63, 329.63, 392.00, 523.25,
        ];
        
        // Zelda-style chord progression (I-V-vi-IV)
        // C major, G major, A minor, F major
        let zelda_chords = [
            (261.63, 329.63, 392.00),  // C major
            (392.00, 493.88, 587.33),  // G major
            (220.00, 261.63, 329.63),  // A minor
            (174.61, 220.00, 261.63),  // F major
        ];
        
        let melody_speed = 0.3;  // How fast the melody plays
        let bass_speed = 0.15;   // How fast the bass/chords play
        
        for x in out.iter_mut() {
            // Melody (Tetris theme)
            let melody_index = ((self.melody_time * melody_speed) as usize) % tetris_melody.len();
            let melody_freq = tetris_melody[melody_index];
            let melody_phase = self.time * melody_freq * 2.0 * std::f32::consts::PI;
            let melody = (melody_phase.sin() * 0.3).max(-1.0).min(1.0);
            
            // Bass/Chord (Zelda progression)
            let chord_index = ((self.bass_time * bass_speed) as usize) % zelda_chords.len();
            let (bass_freq, mid_freq, high_freq) = zelda_chords[chord_index];
            
            let bass_phase = self.time * bass_freq * 2.0 * std::f32::consts::PI;
            let mid_phase = self.time * mid_freq * 2.0 * std::f32::consts::PI;
            let high_phase = self.time * high_freq * 2.0 * std::f32::consts::PI;
            
            let bass = (bass_phase.sin() * 0.2).max(-1.0).min(1.0);
            let mid = (mid_phase.sin() * 0.15).max(-1.0).min(1.0);
            let high = (high_phase.sin() * 0.1).max(-1.0).min(1.0);
            
            // Combine all layers
            *x = (melody + bass + mid + high).max(-1.0).min(1.0);
            
            self.time += 1.0 / (self.sample_rate as f32);
            self.melody_time += 1.0 / (self.sample_rate as f32);
            self.bass_time += 1.0 / (self.sample_rate as f32);
        }
    }
}

/// Audio manager for the game
pub struct AudioManager {
    _device: AudioDevice<MusicGenerator>,
}

impl AudioManager {
    /// Creates a new audio manager and starts playing background music
    pub fn new(sdl: &sdl2::Sdl) -> Result<Self, String> {
        let audio_subsystem = sdl.audio()?;
        
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),  // Mono
            samples: None,     // Default sample size
        };
        
        let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
            MusicGenerator {
                sample_rate: spec.freq,
                time: 0.0,
                melody_time: 0.0,
                bass_time: 0.0,
            }
        })?;
        
        // Start playing
        device.resume();
        
        Ok(AudioManager {
            _device: device,
        })
    }
}

