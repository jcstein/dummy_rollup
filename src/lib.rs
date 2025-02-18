use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub fen: String,
    pub last_move: Option<String>,
    pub game_over: bool,
    pub winner: Option<String>,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
            last_move: None,
            game_over: false,
            winner: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameMessage {
    NewGame,
    Move(String),
    GameState(GameState),
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveRequest {
    pub move_str: String,
    pub game_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveResponse {
    pub success: bool,
    pub error: Option<String>,
    pub game_state: Option<GameState>,
}
