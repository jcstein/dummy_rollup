# Celestia Chess Rollup

A decentralized chess game implementation that uses Celestia for move storage and game state management. This project demonstrates how to build a simple game rollup on Celestia, allowing for verifiable and transparent chess gameplay with all moves stored on-chain.

## Features

- Command-line chess interface
- Move validation and game state management
- Real-time move verification on Celestia
- Support for standard chess notation (UCI format)
- Automatic game state synchronization with the blockchain

## Prerequisites

- Rust
- Celestia light node running locally

### Setting up Celestia Light Node

Start a Celestia light node with:

```shell
celestia light start --core.ip rpc-mocha.pops.one --core.port 9090 --p2p.network mocha --rpc.skip-auth
```

## Quick Start

1. Build the project:
```shell
cargo build
```

2. Start a new game:
```shell
cargo run
```

3. Make moves using UCI format (e.g., "e2e4" to move pawn from e2 to e4)

## How It Works

1. Each move is validated locally using the chess engine
2. Valid moves are submitted as blobs to Celestia
3. The game state is retrieved and verified from Celestia after each move
4. All game states are stored in a custom namespace on Celestia

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

- `celestia_rpc`: Celestia node interaction
- `celestia_types`: Celestia data types
- `chess`: Chess game logic and validation
- `serde`: Game state serialization
- `chrono`: Timestamp logging

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
└── README.md
```

### Running Tests

```shell
cargo test
```

## Contributing

Feel free to submit issues and enhancement requests!
