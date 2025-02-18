use wasm_bindgen::prelude::*;
use yew::prelude::*;
use wasm_logger;

mod app;
mod board;
mod game;

use app::App;

#[wasm_bindgen(start)]
pub fn run_app() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
    Ok(())
}
