# dummy_rollup - tictactoe

This is a Tic-tac-toe game implementation using the Celestia blockchain.

## Prerequisites

- Rust
- Celestia light node running

### Setting up Celestia light node

To start a Celestia light node, use the following command:

```shell
celestia light start --core.ip rpc-mocha.pops.one --p2p.network mocha --rpc.skip-auth
```

## Dependencies

The project uses the following dependencies:

- `anyhow`: For error handling
- `celestia_rpc`: For interacting with the Celestia node
- `celestia_types`: For handling Celestia-specific types like `Namespace` and `Blob`
- `tokio`: For asynchronous runtime
- `hex`: For namespace encoding/decoding

## Game Rules

The game follows standard Tic-tac-toe rules:
- Players take turns placing X's and O's on a 3x3 grid
- First player uses X, second player uses O
- Three in a row (horizontally, vertically, or diagonally) wins
- If the board fills up with no winner, it's a draw

## Usage

### Build

To build the game:

```shell
cargo build
```

### Run

To start or join a game:

```shell
cargo run -- <namespace_plaintext> <start_height>
```

Where:
- `<namespace_plaintext>`: The game identifier that will be converted to a hexadecimal namespace
- `<start_height>`: The blockchain height where the game started

#### Example

```shell
cargo run -- game1 2870188
```

This command:
- Joins/creates a game called "game1"
- Looks for moves starting from height 2870188

### Gameplay

When you start the game, you'll see a board guide showing the position numbers:

```
=== Tic Tac Toe Board Positions ===
Use these numbers to make your move:

0 | 1 | 2
---------
3 | 4 | 5
---------
6 | 7 | 8

Enter 'q' to quit the game
================================
```

During gameplay:
- Enter a number 0-8 to make your move
- Enter 'q' to quit the game

#### Example gameplay

```
Current board state:
. | . | .
---------
. | . | .
---------
. | . | .

Player X's turn
Enter position (0-8) or 'q' to quit:
4

Move submitted at height: 2870190

Current board state:
. | . | .
---------
. | X | .
---------
. | . | .

Player O's turn
Enter position (0-8) or 'q' to quit:
```

The board shows:
- `.` for empty spaces
- `X` for player 1's moves
- `O` for player 2's moves

The game continues until:
- A player wins
- The game ends in a draw
- A player quits by entering 'q'
