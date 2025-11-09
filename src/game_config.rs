//! Game configuration for player modes and roles

/// Game mode: Single player or Multiplayer
#[derive(Clone, Copy, PartialEq)]
pub enum GameMode {
    SinglePlayer,
    Multiplayer,
}

/// Player role: Pac-Man or Ghost
#[derive(Clone, Copy, PartialEq)]
pub enum PlayerRole {
    PacMan,
    Ghost,
}

/// Game configuration
/// 
/// Determines how the game should behave based on player choices
pub struct GameConfig {
    /// Game mode (single or multiplayer)
    pub mode: GameMode,
    /// Player 1 role (always set)
    pub player1_role: PlayerRole,
    /// Player 2 role (only used in multiplayer)
    pub player2_role: Option<PlayerRole>,
}

impl GameConfig {
    /// Creates a new game configuration
    pub fn new(mode: GameMode, player1_role: PlayerRole, player2_role: Option<PlayerRole>) -> Self {
        Self {
            mode,
            player1_role,
            player2_role,
        }
    }
    
    /// Returns true if Pac-Man should be AI-controlled
    pub fn pacman_is_ai(&self) -> bool {
        match self.mode {
            GameMode::SinglePlayer => {
                // In single player, if player chose ghost, Pac-Man is AI
                self.player1_role == PlayerRole::Ghost
            }
            GameMode::Multiplayer => {
                // In multiplayer, if no one is Pac-Man, it's AI (shouldn't happen, but handle it)
                self.player1_role != PlayerRole::PacMan && 
                self.player2_role != Some(PlayerRole::PacMan)
            }
        }
    }
    
    /// Returns true if ghosts should be AI-controlled
    pub fn ghosts_are_ai(&self) -> bool {
        match self.mode {
            GameMode::SinglePlayer => {
                // In single player, if player chose Pac-Man, ghosts are AI
                self.player1_role == PlayerRole::PacMan
            }
            GameMode::Multiplayer => {
                // In multiplayer, if no one is a ghost, they're AI (shouldn't happen, but handle it)
                self.player1_role != PlayerRole::Ghost && 
                self.player2_role != Some(PlayerRole::Ghost)
            }
        }
    }
}

