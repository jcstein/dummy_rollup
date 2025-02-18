use yew::prelude::*;
use web_sys::WebSocket;
use wasm_bindgen::prelude::*;
use serde_json;
use log::{info, error};

use dummy_rollup::{GameState, GameMessage};
use crate::board::ChessBoard;
use crate::game::GameControls;

pub enum Msg {
    Connect,
    WebSocketMessage(String),
    WebSocketError(String),
    WebSocketClosed,
    MakeMove(String),
    NewGame,
    UpdateGameState(GameState),
}

pub struct App {
    ws: Option<WebSocket>,
    game_state: GameState,
    error: Option<String>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::Connect);
        Self {
            ws: None,
            game_state: GameState::new(),
            error: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Connect => {
                let ws = WebSocket::new("ws://localhost:8080").unwrap();
                let link = ctx.link().clone();

                let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: web_sys::MessageEvent| {
                    if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                        link.send_message(Msg::WebSocketMessage(txt.as_string().unwrap()));
                    }
                });

                let onerror_callback = Closure::<dyn FnMut(_)>::new(move |e: web_sys::ErrorEvent| {
                    error!("WebSocket error: {:?}", e);
                    link.send_message(Msg::WebSocketError(e.message()));
                });

                let onclose_callback = Closure::<dyn FnMut(_)>::new(move |_| {
                    link.send_message(Msg::WebSocketClosed);
                });

                ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
                ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
                ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));

                // Keep callbacks alive
                onmessage_callback.forget();
                onerror_callback.forget();
                onclose_callback.forget();

                self.ws = Some(ws);
                true
            }
            Msg::WebSocketMessage(message) => {
                if let Ok(game_msg) = serde_json::from_str::<GameMessage>(&message) {
                    match game_msg {
                        GameMessage::GameState(state) => {
                            self.game_state = state;
                            self.error = None;
                        }
                        GameMessage::Error(error) => {
                            self.error = Some(error);
                        }
                        _ => {}
                    }
                }
                true
            }
            Msg::WebSocketError(error) => {
                self.error = Some(error);
                true
            }
            Msg::WebSocketClosed => {
                self.error = Some("WebSocket connection closed".to_string());
                true
            }
            Msg::MakeMove(move_str) => {
                if let Some(ws) = &self.ws {
                    let msg = GameMessage::Move(move_str);
                    if let Ok(msg_str) = serde_json::to_string(&msg) {
                        ws.send_with_str(&msg_str).unwrap();
                    }
                }
                false
            }
            Msg::NewGame => {
                if let Some(ws) = &self.ws {
                    let msg = GameMessage::NewGame;
                    if let Ok(msg_str) = serde_json::to_string(&msg) {
                        ws.send_with_str(&msg_str).unwrap();
                    }
                }
                false
            }
            Msg::UpdateGameState(state) => {
                self.game_state = state;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onmove = ctx.link().callback(Msg::MakeMove);
        let onnewgame = ctx.link().callback(|_| Msg::NewGame);

        html! {
            <div class="container mx-auto px-4 py-8">
                <h1 class="text-4xl font-bold mb-8 text-center">{"Chess Game"}</h1>
                
                if let Some(error) = &self.error {
                    <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative mb-4">
                        {error}
                    </div>
                }

                <div class="flex flex-col md:flex-row gap-8">
                    <div class="flex-1">
                        <ChessBoard
                            fen={self.game_state.fen.clone()}
                            onmove={onmove}
                        />
                    </div>
                    
                    <div class="flex-1">
                        <GameControls
                            game_state={self.game_state.clone()}
                            onnewgame={onnewgame}
                        />
                    </div>
                </div>
            </div>
        }
    }
}
