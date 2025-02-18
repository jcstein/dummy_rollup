use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use serde_json;

use crate::{GameState, GameMessage, MoveRequest, MoveResponse};

type Clients = Arc<Mutex<HashMap<String, broadcast::Sender<Message>>>>;

pub struct GameServer {
    clients: Clients,
    game_state: Arc<Mutex<GameState>>,
}

impl GameServer {
    pub fn new() -> Self {
        GameServer {
            clients: Arc::new(Mutex::new(HashMap::new())),
            game_state: Arc::new(Mutex::new(GameState::new())),
        }
    }

    pub async fn run(&self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        println!("WebSocket server listening on: {}", addr);

        while let Ok((stream, _)) = listener.accept().await {
            let clients = self.clients.clone();
            let game_state = self.game_state.clone();
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(stream, clients, game_state).await {
                    eprintln!("Error handling connection: {}", e);
                }
            });
        }

        Ok(())
    }

    async fn handle_connection(
        stream: TcpStream,
        clients: Clients,
        game_state: Arc<Mutex<GameState>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ws_stream = tokio_tungstenite::accept_async(stream).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        
        let (tx, mut rx) = broadcast::channel(100);
        let client_id = uuid::Uuid::new_v4().to_string();
        
        // Add client to the list
        clients.lock().unwrap().insert(client_id.clone(), tx.clone());

        // Send current game state
        let current_state = game_state.lock().unwrap().clone();
        let state_msg = serde_json::to_string(&GameMessage::GameState(current_state))?;
        ws_sender.send(Message::Text(state_msg)).await?;

        loop {
            tokio::select! {
                Some(msg) = ws_receiver.next() => {
                    match msg? {
                        Message::Text(text) => {
                            if let Ok(msg) = serde_json::from_str::<GameMessage>(&text) {
                                match msg {
                                    GameMessage::Move(move_str) => {
                                        // Handle move
                                        let request = MoveRequest {
                                            move_str,
                                            game_id: None,
                                        };
                                        
                                        // TODO: Validate move and update game state
                                        // For now, just broadcast the move
                                        let response = MoveResponse {
                                            success: true,
                                            error: None,
                                            game_state: Some(game_state.lock().unwrap().clone()),
                                        };
                                        
                                        let response_msg = serde_json::to_string(&response)?;
                                        for (_, tx) in clients.lock().unwrap().iter() {
                                            let _ = tx.send(Message::Text(response_msg.clone()));
                                        }
                                    },
                                    GameMessage::NewGame => {
                                        let mut state = game_state.lock().unwrap();
                                        *state = GameState::new();
                                        let state_msg = serde_json::to_string(&GameMessage::GameState(state.clone()))?;
                                        for (_, tx) in clients.lock().unwrap().iter() {
                                            let _ = tx.send(Message::Text(state_msg.clone()));
                                        }
                                    },
                                    _ => {}
                                }
                            }
                        },
                        Message::Close(_) => break,
                        _ => {}
                    }
                },
                Ok(msg) = rx.recv() => {
                    ws_sender.send(msg).await?;
                }
            }
        }

        // Remove client when disconnected
        clients.lock().unwrap().remove(&client_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_tungstenite::connect_async;
    use url::Url;

    #[tokio::test]
    async fn test_server_connection() {
        let server = GameServer::new();
        let server_handle = tokio::spawn(async move {
            server.run("127.0.0.1:8080").await.unwrap();
        });

        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let url = Url::parse("ws://127.0.0.1:8080").unwrap();
        let (ws_stream, _) = connect_async(url).await.unwrap();
        let (mut write, mut read) = ws_stream.split();

        // Test new game message
        let new_game_msg = serde_json::to_string(&GameMessage::NewGame).unwrap();
        write.send(Message::Text(new_game_msg)).await.unwrap();

        // Should receive game state
        if let Some(Ok(msg)) = read.next().await {
            if let Message::Text(text) = msg {
                let game_msg: GameMessage = serde_json::from_str(&text).unwrap();
                match game_msg {
                    GameMessage::GameState(state) => {
                        assert_eq!(state.game_over, false);
                        assert_eq!(state.winner, None);
                    },
                    _ => panic!("Expected GameState message"),
                }
            }
        }

        server_handle.abort();
    }
}
