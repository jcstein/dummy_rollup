# dummy_rollup

This is a chess rollup project that uses Celestia for move storage and game state management, with a web-based frontend for interactive gameplay.

## Prerequisites

- Rust
- Celestia light node running
- Trunk (for frontend development)
  ```shell
  cargo install trunk
  ```

### Setting up Celestia light node

To start a Celestia light node, use the following command:

```shell
celestia light start --core.ip rpc-mocha.pops.one --core.port 9090--p2p.network mocha --rpc.skip-auth
```

## Dependencies

The project uses the following dependencies:

Backend:
- `celestia_rpc`: For interacting with the Celestia node
- `celestia_types`: For handling Celestia-specific types
- `chess`: For chess game logic and move validation
- `serde`: For game state serialization
- `chrono`: For timestamp logging

Frontend:
- `yew`: Rust-based web framework
- `trunk`: For building and serving
- `chess.js`: Chess board visualization
- `tailwindcss`: Styling

## Usage

### Build

```shell
# Build backend
cargo build

# Build frontend
trunk build
```

### Development

```shell
# Run backend WebSocket server
cargo run

# Run frontend development server (in a separate terminal)
trunk serve
```

Then visit `http://localhost:8080` in your browser.

### Playing Chess

You can play chess in two ways:

1. Web Interface:
   - Visit `http://localhost:8080`
   - Click and drag pieces to make moves
   - Use the "New Game" button to start a new game

2. Command Line:
   ```shell
   # Start a new game
   cargo run -- chess new

   # Make moves using UCI format (e.g., moving pawn from e2 to e4)
   cargo run -- chess e2e4

   # Make moves using SAN format (e.g., moving knight to f3)
   cargo run -- chess Nf3
   ```

### Move Formats

1. UCI format: Specifies the source and target squares
   - Example: `e2e4` (move from e2 to e4)
   - Example: `g1f3` (move from g1 to f3)

2. SAN format: Standard chess notation
   - Example: `e4` (pawn to e4)
   - Example: `Nf3` (knight to f3)
   - Example: `O-O` (kingside castling)

### Game State

The game state is stored on Celestia and includes:
- Current board position (FEN format)
- Last move played
- Game status (ongoing/finished)
- Winner (if game is over)

Each move is verified for legality before being submitted to the chain.

### Example Game

Here's an example of how to play the famous Scholar's Mate:

```shell
# Start new game
cargo run -- chess new

# White's first move (e4)
cargo run -- chess e2e4

# Black's response (e5)
cargo run -- chess e7e5

# White's second move (Qh5)
cargo run -- chess d1h5

# Black's response (Nc6)
cargo run -- chess b8c6

# White's third move (Bc4)
cargo run -- chess f1c4

# Black's response (Nf6??)
cargo run -- chess g8f6

# White's checkmate (Qxf7#)
cargo run -- chess h5f7
```

## Development

### Project Structure

```
/
├── src/
│   ├── main.rs        (backend game logic)
│   ├── lib.rs         (shared types)
│   ├── server.rs      (WebSocket server)
│   └── frontend/
│       ├── main.rs    (frontend entry)
│       ├── app.rs     (main app component)
│       ├── board.rs   (chess board component)
│       └── game.rs    (game state management)
├── static/
│   ├── index.html
│   └── assets/
│       └── pieces/    (chess piece SVGs)
├── Cargo.toml
└── README.md
```

### Running Tests

```shell
# Run backend tests
cargo test

# Run frontend tests
wasm-pack test --chrome
```
