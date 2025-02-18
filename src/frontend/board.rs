use yew::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlElement, MouseEvent};
use gloo::utils::document;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window)]
    fn alert(s: &str);

    type Chess;

    #[wasm_bindgen(constructor)]
    fn new() -> Chess;

    #[wasm_bindgen(method)]
    fn load(this: &Chess, fen: &str) -> bool;

    #[wasm_bindgen(method)]
    fn move(this: &Chess, move_str: &str) -> bool;

    #[wasm_bindgen(method)]
    fn fen(this: &Chess) -> String;

    #[wasm_bindgen(method)]
    fn in_check(this: &Chess) -> bool;

    #[wasm_bindgen(method)]
    fn in_checkmate(this: &Chess) -> bool;

    #[wasm_bindgen(method)]
    fn in_stalemate(this: &Chess) -> bool;

    #[wasm_bindgen(method)]
    fn in_draw(this: &Chess) -> bool;
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub fen: String,
    pub onmove: Callback<String>,
}

pub struct ChessBoard {
    chess: Chess,
    selected_square: Option<String>,
    board_ref: NodeRef,
}

pub enum Msg {
    SquareClick(String, i32, i32),
    UpdateFen(String),
}

impl Component for ChessBoard {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            chess: Chess::new(),
            selected_square: None,
            board_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SquareClick(square, x, y) => {
                if let Some(selected) = &self.selected_square {
                    let move_str = format!("{}{}", selected, square);
                    if self.chess.move(&move_str) {
                        ctx.props().onmove.emit(move_str);
                        self.selected_square = None;
                        true
                    } else {
                        self.selected_square = Some(square);
                        true
                    }
                } else {
                    self.selected_square = Some(square);
                    true
                }
            }
            Msg::UpdateFen(fen) => {
                self.chess.load(&fen);
                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.chess.load(&ctx.props().fen);
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let board_ref = self.board_ref.clone();
        let onclick = ctx.link().callback(move |e: MouseEvent| {
            if let Some(board) = board_ref.cast::<HtmlElement>() {
                let rect = board.get_bounding_client_rect();
                let square_size = rect.width() / 8.0;
                let x = ((e.client_x() as f64 - rect.left()) / square_size) as i32;
                let y = 7 - ((e.client_y() as f64 - rect.top()) / square_size) as i32;
                let square = format!("{}{}", 
                    (('a' as u8) + x as u8) as char,
                    y + 1
                );
                Msg::SquareClick(square, x, y)
            } else {
                Msg::SquareClick("".to_string(), 0, 0)
            }
        });

        html! {
            <div class="relative w-full aspect-square" ref={self.board_ref.clone()}>
                <div 
                    class="absolute inset-0 grid grid-cols-8 grid-rows-8 bg-white cursor-pointer"
                    onclick={onclick}
                >
                    { self.render_squares() }
                    { self.render_pieces(&ctx.props().fen) }
                </div>
            </div>
        }
    }
}

impl ChessBoard {
    fn render_squares(&self) -> Html {
        let mut squares = Vec::new();
        for rank in (0..8).rev() {
            for file in 0..8 {
                let is_dark = (rank + file) % 2 == 1;
                let square = format!("{}{}", 
                    (('a' as u8) + file as u8) as char,
                    rank + 1
                );
                let is_selected = self.selected_square.as_ref() == Some(&square);
                let bg_color = if is_selected {
                    "bg-yellow-200"
                } else if is_dark {
                    "bg-gray-400"
                } else {
                    "bg-gray-200"
                };
                
                squares.push(html! {
                    <div class={classes!("border", "border-gray-300", bg_color)}></div>
                });
            }
        }
        html! { <>{squares}</> }
    }

    fn render_pieces(&self, fen: &str) -> Html {
        let mut pieces = Vec::new();
        let ranks: Vec<&str> = fen.split('/').collect();
        
        for (rank_idx, rank) in ranks.iter().enumerate() {
            let mut file_idx = 0;
            for c in rank.chars() {
                if c.is_digit(10) {
                    file_idx += c.to_digit(10).unwrap() as usize;
                } else {
                    let piece_type = match c.to_lowercase().next().unwrap() {
                        'p' => "pawn",
                        'n' => "knight",
                        'b' => "bishop",
                        'r' => "rook",
                        'q' => "queen",
                        'k' => "king",
                        _ => continue,
                    };
                    let color = if c.is_uppercase() { "white" } else { "black" };
                    let left = format!("{}%", (file_idx as f32 / 8.0) * 100.0);
                    let top = format!("{}%", (rank_idx as f32 / 8.0) * 100.0);
                    
                    pieces.push(html! {
                        <div 
                            class="absolute w-1/8 h-1/8 flex items-center justify-center"
                            style={format!("left: {}; top: {}", left, top)}
                        >
                            <img 
                                src={format!("/assets/pieces/{}-{}.svg", color, piece_type)}
                                alt={format!("{} {}", color, piece_type)}
                                class="w-full h-full pointer-events-none"
                            />
                        </div>
                    });
                    file_idx += 1;
                }
            }
        }
        html! { <>{pieces}</> }
    }
}
