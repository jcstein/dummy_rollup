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
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, WebSocketStream, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use tokio::sync::Mutex;
use std::net::SocketAddr;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GameState {
    fen: String,
    last_move: Option<String>,
    game_over: bool,
    winner: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct WebSocketMessage {
    #[serde(rename = "type")]
    msg_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    move_str: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<GameState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
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

type PeerMap = Arc<Mutex<HashMap<SocketAddr, futures_util::stream::SplitSink<WebSocketStream<TcpStream>, Message>>>>;

async fn handle_connection(
    peer_map: PeerMap,
    raw_stream: TcpStream,
    addr: SocketAddr,
    client: Arc<Client>,
    namespace: Arc<Namespace>,
    game: Arc<Mutex<Game>>,
) {
    let ws_stream = match tokio_tungstenite::accept_async(raw_stream).await {
        Ok(stream) => stream,
        Err(e) => {
            log_with_timestamp(&format!("Failed to accept websocket connection: {}", e));
            return;
        }
    };
    
    let (ws_sender, mut ws_receiver) = ws_stream.split();
    
    // Send initial game state
    let current_game = game.lock().await;
    let game_state = GameState::from_game(&current_game, None);
    let initial_state = WebSocketMessage {
        msg_type: "gameState".to_string(),
        move_str: None,
        state: Some(game_state),
        message: None,
    };
    
    {
        let mut peer_map = peer_map.lock().await;
        peer_map.insert(addr, ws_sender);
    }
    
    let peer_map_clone = peer_map.clone();
    if let Some(sender) = peer_map_clone.lock().await.get_mut(&addr) {
        if let Err(e) = sender.send(Message::Text(
            serde_json::to_string(&initial_state).unwrap()
        )).await {
            log_with_timestamp(&format!("Failed to send initial state: {}", e));
            return;
        }
    }
    drop(current_game);
    
    while let Some(result) = ws_receiver.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                log_with_timestamp(&format!("Error receiving message: {}", e));
                break;
            }
        };
        
        if let Ok(text) = msg.to_text() {
            if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(text) {
                match ws_msg.msg_type.as_str() {
                    "move" => {
                        if let Some(move_str) = ws_msg.move_str {
                            if let Some(chess_move) = parse_uci(&move_str) {
                                let mut current_game = game.lock().await;
                                if current_game.make_move(chess_move) {
                                    let move_str = move_str.clone();
                                    let game_state = GameState::from_game(&current_game, Some(move_str.clone()));
                                    
                                    // Submit move to Celestia
                                    let height = client.header_network_head().await.unwrap().height();
                                    let blob = Blob::new(
                                        (*namespace).clone(),
                                        serde_json::to_vec(&game_state).unwrap(),
                                        AppVersion::V2,
                                    ).unwrap();
                                    
                                    match client.blob_submit(&[blob], TxConfig::default()).await {
                                        Ok(_) => {
                                            let celestia_msg = WebSocketMessage {
                                                msg_type: "celestiaUpdate".to_string(),
                                                move_str: None,
                                                state: None,
                                                message: Some(format!("✅ Move submitted to Celestia at height {}", height)),
                                            };
                                            
                                            if let Some(sender) = peer_map_clone.lock().await.get_mut(&addr) {
                                                if let Err(e) = sender.send(Message::Text(
                                                    serde_json::to_string(&celestia_msg).unwrap()
                                                )).await {
                                                    log_with_timestamp(&format!("Failed to send Celestia update: {}", e));
                                                    continue;
                                                }
                                            }

                                            // Wait for the next block and verify the blob was included
                                            let verify_msg = WebSocketMessage {
                                                msg_type: "celestiaUpdate".to_string(),
                                                move_str: None,
                                                state: None,
                                                message: Some("🔍 Verifying move inclusion in Celestia...".to_string()),
                                            };
                                            if let Some(sender) = peer_map_clone.lock().await.get_mut(&addr) {
                                                let _ = sender.send(Message::Text(
                                                    serde_json::to_string(&verify_msg).unwrap()
                                                )).await;
                                            }

                                            // Wait for a few seconds to allow the block to be created
                                            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                                            
                                            // Get the current height and check the last few blocks
                                            let current_height = client.header_network_head().await.unwrap().height();
                                            let mut found = false;
                                            for h in (u64::from(height)..=u64::from(current_height)).rev() {
                                                if let Ok(Some(blobs)) = client.blob_get_all(h, &[(*namespace).clone()]).await {
                                                    for blob in blobs {
                                                        if let Ok(state) = serde_json::from_slice::<GameState>(&blob.data) {
                                                            if state.last_move == Some(move_str.clone()) {
                                                                found = true;
                                                                let verify_success_msg = WebSocketMessage {
                                                                    msg_type: "celestiaUpdate".to_string(),
                                                                    move_str: None,
                                                                    state: None,
                                                                    message: Some(format!("✅ Move verified in Celestia at height {}", h)),
                                                                };
                                                                if let Some(sender) = peer_map_clone.lock().await.get_mut(&addr) {
                                                                    let _ = sender.send(Message::Text(
                                                                        serde_json::to_string(&verify_success_msg).unwrap()
                                                                    )).await;
                                                                }
                                                                break;
                                                            }
                                                        }
                                                    }
                                                }
                                                if found {
                                                    break;
                                                }
                                            }

                                            if !found {
                                                let verify_fail_msg = WebSocketMessage {
                                                    msg_type: "celestiaUpdate".to_string(),
                                                    move_str: None,
                                                    state: None,
                                                    message: Some("⚠️ Move submission verified but inclusion not yet confirmed".to_string()),
                                                };
                                                if let Some(sender) = peer_map_clone.lock().await.get_mut(&addr) {
                                                    let _ = sender.send(Message::Text(
                                                        serde_json::to_string(&verify_fail_msg).unwrap()
                                                    )).await;
                                                }
                                            }
                                            
                                            // Broadcast new game state to all connected clients
                                            let state_update = WebSocketMessage {
                                                msg_type: "gameState".to_string(),
                                                move_str: Some(move_str.clone()),
                                                state: Some(game_state),
                                                message: None,
                                            };
                                            let state_msg = Message::Text(
                                                serde_json::to_string(&state_update).unwrap()
                                            );
                                            
                                            let mut peer_map = peer_map_clone.lock().await;
                                            for (_, sender) in peer_map.iter_mut() {
                                                if let Err(e) = sender.send(state_msg.clone()).await {
                                                    log_with_timestamp(&format!("Failed to broadcast game state: {}", e));
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            let error_msg = WebSocketMessage {
                                                msg_type: "celestiaUpdate".to_string(),
                                                move_str: None,
                                                state: None,
                                                message: Some(format!("❌ Failed to submit move to Celestia: {}", e)),
                                            };
                                            
                                            if let Some(sender) = peer_map_clone.lock().await.get_mut(&addr) {
                                                if let Err(e) = sender.send(Message::Text(
                                                    serde_json::to_string(&error_msg).unwrap()
                                                )).await {
                                                    log_with_timestamp(&format!("Failed to send error message: {}", e));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    let mut peer_map = peer_map.lock().await;
    peer_map.remove(&addr);
}

#[tokio::main]
async fn main() -> Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    // Initialize Celestia client with local light node
    let client = Arc::new(Client::new("http://localhost:26658", None).await?);
    let namespace = Arc::new(Namespace::new(0, &create_namespace("chess"))?);
    
    // Try to load existing game state from latest height
    let height: u64 = client.header_network_head().await?.height().into();
    let initial_game = if let Ok(Some(state)) = get_latest_game_state(&client, &namespace, height).await {
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
    
    let game = Arc::new(Mutex::new(initial_game));
    let peer_map = Arc::new(Mutex::new(HashMap::new()));
    
    let addr = "127.0.0.1:8080".to_string();
    let listener = TcpListener::bind(&addr).await?;
    log_with_timestamp(&format!("WebSocket server listening on: {}", addr));
    
    while running.load(Ordering::SeqCst) {
        tokio::select! {
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((stream, addr)) => {
                        log_with_timestamp(&format!("New WebSocket connection: {}", addr));
                        
                        let peer_map = peer_map.clone();
                        let game = game.clone();
                        let client = client.clone();
                        let namespace = namespace.clone();
                        
                        tokio::spawn(async move {
                            handle_connection(peer_map, stream, addr, client, namespace, game).await;
                        });
                    }
                    Err(e) => {
                        log_with_timestamp(&format!("Failed to accept connection: {}", e));
                    }
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                // Check running flag periodically
                if !running.load(Ordering::SeqCst) {
                    log_with_timestamp("Received shutdown signal, stopping server...");
                    break;
                }
            }
        }
    }

    log_with_timestamp("Server shutdown complete");
    Ok(())
}
