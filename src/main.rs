use anyhow::Result;
use celestia_rpc::{BlobClient, Client, HeaderClient};
use celestia_types::{nmt::Namespace, AppVersion, Blob};
use celestia_rpc::TxConfig;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use chrono::Local;
use chess::{Game, ChessMove, MoveGen, Color, Square, Piece, Rank, File};
use serde::{Serialize, Deserialize};
use std::str::FromStr;
use ctrlc;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GameState {
    fen: String,
    last_move: Option<String>,
    game_over: bool,
    winner: Option<String>,
}

impl GameState {
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
    log_with_timestamp(&format!("🔍 Starting game state retrieval from height {}", height));
    
    // Look back through the last 10 blocks
    for h in (height.saturating_sub(10)..=height).rev() {
        log_with_timestamp(&format!("📦 Checking block height {} for game state", h));
        let blobs = client
            .blob_get_all(h, &[namespace.clone()])
            .await
            .map_err(|e| anyhow::anyhow!("Failed to retrieve blobs: {}", e))?;
            
        match &blobs {
            Some(b) if !b.is_empty() => {
                log_with_timestamp(&format!("✅ Found {} blob(s) at height {}", b.len(), h));
                // Get the latest blob
                let latest_blob = b.last().unwrap();
                let state: GameState = serde_json::from_slice(&latest_blob.data)
                    .map_err(|e| anyhow::anyhow!("Failed to deserialize game state: {}", e))?;
                
                log_with_timestamp(&format!("🎮 Retrieved game state: FEN={}, last_move={:?}", 
                    state.fen, state.last_move));
                return Ok(Some(state));
            }
            Some(_) => {
                log_with_timestamp(&format!("ℹ️  No game blobs found at height {}", h));
            }
            None => {
                log_with_timestamp(&format!("ℹ️  No blobs at all at height {}", h));
            }
        }
    }
    
    log_with_timestamp("❌ No game state found in the last 10 blocks");
    Ok(None)
}

fn print_board(game: &Game) {
    println!("\n  Current board position:");
    println!("    a b c d e f g h");
    println!("    ---------------");
    
    let board = game.current_position();
    for rank in (0..8).rev() {
        print!("{}   ", rank + 1);
        for file in 0..8 {
            let square = Square::make_square(Rank::from_index(rank), File::from_index(file));
            match board.piece_on(square) {
                Some(piece) => {
                    let symbol = match piece {
                        Piece::Pawn => if board.color_on(square) == Some(Color::White) { "♙" } else { "♟" },
                        Piece::Knight => if board.color_on(square) == Some(Color::White) { "♘" } else { "♞" },
                        Piece::Bishop => if board.color_on(square) == Some(Color::White) { "♗" } else { "♝" },
                        Piece::Rook => if board.color_on(square) == Some(Color::White) { "♖" } else { "♜" },
                        Piece::Queen => if board.color_on(square) == Some(Color::White) { "♕" } else { "♛" },
                        Piece::King => if board.color_on(square) == Some(Color::White) { "♔" } else { "♚" },
                    };
                    print!("{} ", symbol);
                }
                None => print!(". "),
            }
        }
        println!(" {}", rank + 1);
    }
    println!("    ---------------");
    println!("    a b c d e f g h\n");
}

#[tokio::main]
async fn main() -> Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    // Initialize Celestia client with local light node
    let client = Client::new("http://localhost:26658", None).await?;
    let namespace = Namespace::new(0, &create_namespace("chess"))?;
    
    // Try to load existing game state from latest height
    let height: u64 = client.header_network_head().await?.height().into();
    let mut game = if let Ok(Some(state)) = get_latest_game_state(&client, &namespace, height).await {
        match Game::from_str(&state.fen) {
            Ok(g) => g,
            Err(e) => {
                log_with_timestamp(&format!("Failed to load game state: {}", e));
                Game::new()
            }
        }
    } else {
        Game::new()
    };
    
    println!("Welcome to CLI Chess!");
    println!("Enter moves in UCI format (e.g., 'e2e4') or 'quit' to exit");
    
    while running.load(Ordering::SeqCst) {
        print_board(&game);
        
        if game.result().is_some() {
            println!("Game Over! Result: {:?}", game.result().unwrap());
            break;
        }
        
        println!("Current turn: {}", if game.side_to_move() == Color::White { "White" } else { "Black" });
        print!("Enter your move: ");
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.eq_ignore_ascii_case("quit") {
            break;
        }
        
        match parse_uci(input) {
            Some(chess_move) => {
                let legal_moves: Vec<ChessMove> = MoveGen::new_legal(&game.current_position()).collect();
                if legal_moves.contains(&chess_move) {
                    game.make_move(chess_move);
                    println!("Move played: {}", chess_move);
                    
                    // Create and submit game state to Celestia
                    let state = GameState::from_game(&game, Some(chess_move.to_string()));
                    let state_json = serde_json::to_vec(&state)?;
                    let blob = Blob::new(namespace.clone(), state_json, AppVersion::V2)?;
                    
                    match client.blob_submit(&[blob], TxConfig::default()).await {
                        Ok(_) => {
                            log_with_timestamp("✅ Game state submitted to Celestia");
                            
                            // Get the latest height and verify the state immediately
                            let height: u64 = client.header_network_head().await?.height().into();
                            match get_latest_game_state(&client, &namespace, height).await {
                                Ok(Some(celestia_state)) => {
                                    // Verify the state matches what we submitted
                                    if celestia_state.fen != state.fen {
                                        log_with_timestamp("⚠️  Warning: Celestia state doesn't match local state!");
                                        log_with_timestamp(&format!("Local FEN: {}", state.fen));
                                        log_with_timestamp(&format!("Celestia FEN: {}", celestia_state.fen));
                                        // Revert to Celestia's state
                                        match Game::from_str(&celestia_state.fen) {
                                            Ok(g) => {
                                                game = g;
                                                log_with_timestamp("✅ Reverted to Celestia state");
                                            }
                                            Err(e) => {
                                                log_with_timestamp(&format!("❌ Failed to load Celestia state: {}", e));
                                            }
                                        }
                                    } else {
                                        log_with_timestamp("✅ Move verified on Celestia");
                                    }
                                }
                                Ok(None) => {
                                    log_with_timestamp("⚠️  Warning: Move not found on Celestia after submission!");
                                }
                                Err(e) => {
                                    log_with_timestamp(&format!("❌ Failed to verify move on Celestia: {}", e));
                                }
                            }
                        }
                        Err(e) => log_with_timestamp(&format!("❌ Failed to submit game state: {}", e)),
                    }
                } else {
                    println!("Illegal move! Try again.");
                }
            }
            None => {
                println!("Invalid move format! Use UCI format (e.g., 'e2e4')");
            }
        }
    }
    
    println!("Thanks for playing!");
    Ok(())
}
