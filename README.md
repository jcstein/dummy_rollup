# Celestia Chess Rollup

A decentralized chess game implementation that uses Celestia for move storage and game state management. This project demonstrates how to build a simple game rollup on Celestia, allowing for verifiable and transparent chess gameplay with all moves stored on-chain.

## Features

- Web-based chess interface
- Move validation and game state management
- Real-time move verification on Celestia
- Support for standard chess notation (UCI format)
- Automatic game state synchronization with the blockchain
- Real-time game updates across multiple clients

## Prerequisites

- Rust
- Node.js (v16 or later)
- Celestia light node running locally

### Setting up Celestia Light Node

Start a Celestia light node with:

```shell
celestia light start --core.ip rpc-mocha.pops.one --core.port 9090 --p2p.network mocha --rpc.skip-auth
```

## Quick Start

1. Build and start the backend:
```shell
cargo build
cargo run
```

2. Install frontend dependencies:
```shell
cd frontend
npm install
```

3. Start the frontend development server:
```shell
npm start
```

4. Open your browser and navigate to `http://localhost:3000`

## How It Works

1. The frontend provides a visual chess board interface
2. Each move is validated locally using the chess engine
3. Valid moves are submitted as blobs to Celestia
4. The game state is retrieved and verified from Celestia after each move
5. All game states are stored in a custom namespace on Celestia
6. Real-time updates are broadcasted to all connected clients

## Move Format

The game uses UCI (Universal Chess Interface) format for moves:
- Format: `[source square][target square]`
- Examples:
  - `e2e4` (moves pawn from e2 to e4)
  - `g1f3` (moves knight from g1 to f3)
  - `e1g1` (kingside castling)

For pawn promotion, add the piece letter:
- `e7e8q` (promotes pawn to queen)
- `e7e8r` (promotes pawn to rook)
- `e7e8b` (promotes pawn to bishop)
- `e7e8n` (promotes pawn to knight)

## Technical Details

### Dependencies

Backend:
- `celestia_rpc`: Celestia node interaction
- `celestia_types`: Celestia data types
- `chess`: Chess game logic and validation
- `serde`: Game state serialization
- `tokio-tungstenite`: WebSocket server
- `chrono`: Timestamp logging

Frontend:
- `react`: UI framework
- `react-chessboard`: Chess board component
- `chess.js`: Chess logic
- `tailwindcss`: Styling
- `websocket`: WebSocket client

### Game State Structure

Each game state stored on Celestia includes:
- Board position (FEN format)
- Last move played
- Game status
- Winner (if game is over)

## Development

### Project Structure

```
/
├── src/
│   └── main.rs        (game logic and Celestia integration)
├── frontend/
│   ├── src/
│   │   ├── App.tsx    (main React component)
│   │   └── index.tsx  (React entry point)
│   └── public/
│       └── index.html
└── README.md
```

### Running Tests

Backend:
```shell
cargo test
```

Frontend:
```shell
cd frontend
npm test
```

## Contributing

Feel free to submit issues and enhancement requests!
