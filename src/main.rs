use anyhow::{Context, Result};
use celestia_rpc::{BlobClient, Client, HeaderClient};
use celestia_types::{nmt::Namespace, AppVersion, Blob};
use celestia_rpc::TxConfig;
use rand::RngCore;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use chrono::Local;
use hex;
use chess::{Game, ChessMove, Board, MoveGen, Color, Square, Piece};
use serde::{Serialize, Deserialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GameState {
    fen: String,
    last_move: Option<String>,
    game_over: bool,
    winner: Option<String>,
}

impl GameState {
    fn new() -> Self {
        GameState {
            fen: Board::default().to_string(),
            last_move: None,
            game_over: false,
            winner: None,
        }
    }

    fn from_game(game: &Game, last_move: Option<String>) -> Self {
        let winner = if game.result().is_some() {
            match game.result().unwrap() {
                chess::GameResult::WhiteCheckmates => Some("White".to_string()),
                chess::GameResult::BlackCheckmates => Some("Black".to_string()),
                chess::GameResult::WhiteResigns => Some("Black".to_string()),
                chess::GameResult::BlackResigns => Some("White".to_string()),
                chess::GameResult::Stalemate => Some("Draw".to_string()),
                _ => Some("Draw".to_string()),
            }
        } else {
            None
        };

        GameState {
            fen: game.current_position().to_string(),
            last_move,
            game_over: game.result().is_some(),
            winner,
        }
    }
}

/// Helper function to get current timestamp for logging
fn log_with_timestamp(message: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] {}", timestamp, message);
}

/// Parse a move in UCI format (e.g., "e2e4")
fn parse_uci(move_str: &str) -> Option<ChessMove> {
    if move_str.len() != 4 && move_str.len() != 5 {
        return None;
    }

    let from = Square::from_str(&move_str[0..2]).ok()?;
    let to = Square::from_str(&move_str[2..4]).ok()?;
    
    let promotion = if move_str.len() == 5 {
        match move_str.chars().last()? {
            'q' => Some(Piece::Queen),
            'r' => Some(Piece::Rook),
            'b' => Some(Piece::Bishop),
            'n' => Some(Piece::Knight),
            _ => None,
        }
    } else {
        None
    };

    Some(ChessMove::new(from, to, promotion))
}

/// Create a valid 8-byte namespace from a string
fn create_namespace(input: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(8);
    for (i, chunk) in input.as_bytes().chunks(2).enumerate() {
        if i >= 4 {
            break;
        }
        let mut value = chunk[0] as u16;
        if chunk.len() > 1 {
            value = (value << 8) | (chunk[1] as u16);
        }
        bytes.extend_from_slice(&value.to_be_bytes());
    }
    while bytes.len() < 8 {
        bytes.push(0);
    }
    bytes
}

/// Retrieves the latest game state from the Celestia network
async fn get_latest_game_state(client: &Client, namespace: &Namespace, height: u64) -> Result<Option<GameState>> {
    // Look back through the last 10 blocks
    for h in (height.saturating_sub(10)..=height).rev() {
        log_with_timestamp(&format!("Retrieving game state at height {}", h));
        let blobs = client
            .blob_get_all(h, &[namespace.clone()])
            .await
            .map_err(|e| anyhow::anyhow!("Failed to retrieve blobs: {}", e))?
            .unwrap_or_default();

        if !blobs.is_empty() {
            // Get the latest blob
            let latest_blob = blobs.last().unwrap();
            let state: GameState = serde_json::from_slice(&latest_blob.data)
                .map_err(|e| anyhow::anyhow!("Failed to deserialize game state: {}", e))?;
            
            return Ok(Some(state));
        }
    }
    
    Ok(None)
}

#[tokio::main]
async fn main() -> Result<()> {
    log_with_timestamp("Starting chess rollup");
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        log_with_timestamp("Error: Invalid number of arguments");
        eprintln!(
            "Usage: cargo run -- <namespace_id> <move_in_uci_format>"
        );
        eprintln!("Example: cargo run -- chess e2e4");
        eprintln!("Use 'new' as move to start a new game");
        std::process::exit(1);
    }

    let namespace_id = &args[1];
    let chess_move = &args[2];

    // Create namespace
    let namespace_bytes = create_namespace(namespace_id);
    let namespace = Namespace::new_v0(&namespace_bytes)?;
    log_with_timestamp(&format!("Using namespace '{}'", namespace_id));

    // Initialize Celestia client
    log_with_timestamp("Connecting to Celestia node at ws://localhost:26658");
    let client = Client::new("ws://localhost:26658", None)
        .await
        .context("Failed to connect to Celestia node")?;
    log_with_timestamp("Successfully connected to Celestia node");

    // Get current height
    let header = client.header_network_head().await?;
    let height: u64 = header.height().into();
    
    // Get current game state
    let mut game = Game::new();
    let current_state = get_latest_game_state(&client, &namespace, height).await?;
    
    if let Some(state) = current_state {
        // Load the existing game state
        game = Game::from_str(&state.fen).map_err(|e| anyhow::anyhow!("Invalid FEN string: {}", e))?;
        log_with_timestamp("Loaded existing game state");
        log_with_timestamp(&format!("Current position:\n{}", game.current_position()));
        if let Some(last_move) = state.last_move {
            log_with_timestamp(&format!("Last move: {}", last_move));
        }
    } else {
        log_with_timestamp("No existing game found, starting new game");
    }

    // Handle the move
    if chess_move == "new" {
        game = Game::new();
        log_with_timestamp("Started new game");
    } else {
        // Parse and validate the move
        let chess_move = ChessMove::from_san(&game.current_position(), chess_move)
            .or_else(|_| parse_uci(chess_move).ok_or_else(|| anyhow::anyhow!("Invalid move format")))
            .map_err(|_| anyhow::anyhow!("Invalid move format. Use UCI (e.g., e2e4) or SAN (e.g., Nf3)"))?;

        // Verify move is legal
        let legal_moves: Vec<ChessMove> = MoveGen::new_legal(&game.current_position()).collect();
        if !legal_moves.contains(&chess_move) {
            return Err(anyhow::anyhow!("Illegal move"));
        }

        // Make the move
        game.make_move(chess_move);
        log_with_timestamp(&format!("Move made: {}", chess_move));
    }

    // Create new game state
    let new_state = GameState::from_game(&game, Some(chess_move.to_string()));
    let state_json = serde_json::to_vec(&new_state)?;

    // Submit the new state
    log_with_timestamp("Submitting new game state");
    let blob = Blob::new(namespace.clone(), state_json, AppVersion::V2)?;
    let result_height = client.blob_submit(&[blob], TxConfig::default()).await?;
    log_with_timestamp(&format!("Game state submitted at height {}", result_height));

    // Print current board state
    log_with_timestamp(&format!("Current position:\n{}", game.current_position()));
    
    // Check game status
    if new_state.game_over {
        match new_state.winner {
            Some(ref winner) => log_with_timestamp(&format!("Game Over! Winner: {}", winner)),
            None => log_with_timestamp("Game Over! It's a draw!"),
        }
    } else {
        log_with_timestamp(&format!("Next move: {}", if game.side_to_move() == Color::White { "White" } else { "Black" }));
    }

    Ok(())
}

/// Generates a vector of random blobs with specified parameters
fn generate_random_blobs(
    num_blobs: usize,
    blob_size: usize,
    namespace: &Namespace,
) -> Result<Vec<Blob>> {
    let mut rng = rand::thread_rng();
    let mut blobs = Vec::with_capacity(num_blobs);

    // Generate random data for each blob
    for _ in 0..num_blobs {
        let mut random_data = vec![0u8; blob_size];
        rng.fill_bytes(&mut random_data);
        let blob = Blob::new(namespace.clone(), random_data, AppVersion::V2)
            .map_err(|e| anyhow::anyhow!("Failed to create blob: {}", e))?;
        blobs.push(blob);
    }

    Ok(blobs)
}
