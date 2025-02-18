use yew::prelude::*;
use dummy_rollup::GameState;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub game_state: GameState,
    pub onnewgame: Callback<()>,
}

pub struct GameControls;

impl Component for GameControls {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let game_state = &ctx.props().game_state;
        let onnewgame = ctx.props().onnewgame.clone();

        html! {
            <div class="p-4 bg-gray-100 rounded-lg">
                <h2 class="text-2xl font-bold mb-4">{"Game Status"}</h2>
                
                // Game status
                <div class="mb-4">
                    if game_state.game_over {
                        <div class="text-lg font-semibold text-red-600">
                            {"Game Over"}
                            if let Some(winner) = &game_state.winner {
                                <span class="ml-2">
                                    {"Winner: "}{winner}
                                </span>
                            } else {
                                <span class="ml-2">{"Draw"}</span>
                            }
                        </div>
                    } else {
                        <div class="text-lg font-semibold text-green-600">
                            {"Game in progress"}
                        </div>
                    }
                </div>

                // Last move
                if let Some(last_move) = &game_state.last_move {
                    <div class="mb-4">
                        <h3 class="text-lg font-semibold mb-2">{"Last Move"}</h3>
                        <div class="p-2 bg-white rounded">
                            {last_move}
                        </div>
                    </div>
                }

                // Controls
                <div class="mt-6">
                    <button
                        class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
                        onclick={move |_| onnewgame.emit(())}
                    >
                        {"New Game"}
                    </button>
                </div>
            </div>
        }
    }
}
