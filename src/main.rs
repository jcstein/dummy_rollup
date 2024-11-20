use anyhow::{Context, Result};
use celestia_rpc::{BlobClient, Client};
use celestia_types::{nmt::Namespace, AppVersion, Blob, TxConfig};
use std::env;
use std::time::Duration;
use tokio::time::sleep;

const WINNING_COMBINATIONS: [[usize; 3]; 8] = [
    [0, 1, 2], // Top row
    [3, 4, 5], // Middle row
    [6, 7, 8], // Bottom row
    [0, 3, 6], // Left column
    [1, 4, 7], // Middle column
    [2, 5, 8], // Right column
    [0, 4, 8], // Diagonal
    [2, 4, 6], // Diagonal
];

async fn retrieve_blobs(client: &Client, height: u64, namespace: &Namespace) -> Result<Vec<Blob>> {
    let blobs = client
        .blob_get_all(height, &[namespace.clone()])
        .await
        .map_err(|e| anyhow::anyhow!("Failed to retrieve blobs: {}", e))?
        .unwrap_or_default();
    Ok(blobs)
}

async fn submit_move(client: &Client, namespace: &Namespace, position: u8) -> Result<u64> {
    let blob = Blob::new(namespace.clone(), vec![position], AppVersion::V2)?;
    Ok(client.blob_submit(&[blob], TxConfig::default()).await?)
}

// Check if a player has won
fn check_winner(board: &[char; 9]) -> Option<char> {
    for combo in WINNING_COMBINATIONS.iter() {
        if board[combo[0]] != '.'
            && board[combo[0]] == board[combo[1]]
            && board[combo[1]] == board[combo[2]]
        {
            return Some(board[combo[0]]);
        }
    }
    None
}

// Check if the board is full (draw)
fn is_board_full(board: &[char; 9]) -> bool {
    !board.contains(&'.')
}

// Display the board and check game status
fn display_board(moves: &Vec<Vec<u8>>) -> Option<char> {
    let mut board = ['.'; 9];

    // Fill board with X's and O's alternating
    for (i, move_data) in moves.iter().enumerate() {
        if !move_data.is_empty() {
            let position = move_data[0] as usize;
            if position < 9 {
                board[position] = if i % 2 == 0 { 'X' } else { 'O' };
            }
        }
    }

    // Print board
    for i in 0..3 {
        println!("{} {} {}", board[i * 3], board[i * 3 + 1], board[i * 3 + 2]);
    }

    // Check for winner
    if let Some(winner) = check_winner(&board) {
        println!("\nPlayer {} wins!", winner);
        return Some(winner);
    }

    // Check for draw
    if is_board_full(&board) {
        println!("\nGame is a draw!");
        return Some('D'); // 'D' for draw
    }

    None // Game is still ongoing
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: cargo run -- <namespace_plaintext> <start_height> <position(0-8)>");
        std::process::exit(1);
    }

    let namespace_plaintext = &args[1];
    let start_height = args[2].parse::<u64>()?;
    let position = args[3].parse::<u8>()?;

    let namespace_hex = hex::encode(namespace_plaintext);
    let namespace = Namespace::new_v0(&hex::decode(&namespace_hex)?)?;

    let client = Client::new("http://localhost:26658", None)
        .await
        .context("Failed to connect to Celestia node")?;

    // Submit move
    let result_height = submit_move(&client, &namespace, position).await?;
    println!("Move submitted at height: {}", result_height);

    // Wait a bit for ONLY the new block to be processed
    sleep(Duration::from_secs(2)).await;

    // Get all moves from start_height with no delays in between
    let mut all_moves = Vec::new();
    for height in start_height..=result_height {
        if let Ok(blobs) = retrieve_blobs(&client, height, &namespace).await {
            all_moves.extend(blobs.iter().map(|b| b.data.clone()));
        }
    }

    println!("\nCurrent board state:");
    if let Some(game_result) = display_board(&all_moves) {
        match game_result {
            'D' => println!("Game Over - It's a draw!"),
            winner => println!("Game Over - Player {} has won!", winner),
        }
    } else {
        println!("Game is still in progress...");
    }

    Ok(())
}
