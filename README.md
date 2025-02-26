# Celestia Database

A public database implementation on Celestia that allows storing and retrieving records using a namespace.

## Features

- Store and retrieve records using a key-value model
- Namespace-based organization
- Automatic metadata tracking
- Simple CLI interface

## Prerequisites

- Rust and Cargo installed
- A running Celestia node (light node or full node)
- Proper authentication token for the Celestia node

## Setup

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/celestia-database.git
   cd celestia-database
   ```

2. Set up your Celestia node:
   - Follow the [official Celestia documentation](https://docs.celestia.org/nodes/light-node) to set up a light node
   - Make sure to start your node with the `--rpc.enable` flag
   - For testing, you can use the `--rpc.skip-auth` flag to bypass authentication

3. Set the authentication token (if not using `--rpc.skip-auth`):
   ```
   export CELESTIA_NODE_AUTH_TOKEN=your_auth_token
   ```

## Usage

Run the application with a namespace parameter:

```
cargo run -- your_namespace
```

The namespace will be padded or truncated to exactly 8 bytes as required by Celestia.

### Available Commands

Once the application is running, you can use the following commands:

- `add <key> <value>` - Add a new record or update an existing one
- `get <key>` - Retrieve a record by key
- `list` - List all records
- `exit` or `quit` - Exit the application
- `help` - Show help message

## Example

```
$ cargo run -- testdb
[2025-02-25 21:03:07.775] Starting Celestia database application
[2025-02-25 21:03:07.776] Configuration - Namespace: testdb
[2025-02-25 21:03:07.776] Connecting to Celestia node...
[2025-02-25 21:03:07.777] Successfully connected to Celestia node
[2025-02-25 21:03:07.778] Database client initialized

Available commands:
  add <key> <value>  - Add a new record or update existing one
  get <key>          - Retrieve a record by key
  list               - List all records
  exit               - Exit the application
  help               - Show this help message

Enter commands below:
> add user1 {"name": "John Doe", "email": "john@example.com"}
[2025-02-25 21:03:15.123] Adding record with key 'user1'
[2025-02-25 21:03:15.456] Record added successfully
> get user1
[2025-02-25 21:03:20.789] Retrieving record with key 'user1'
Key: user1
Value: {"name": "John Doe", "email": "john@example.com"}
Created: 2025-02-25T21:03:15.123Z
> list
[2025-02-25 21:03:25.321] Listing all records
Key: user1
Value: {"name": "John Doe", "email": "john@example.com"}
Created: 2025-02-25T21:03:15.123Z
---
> exit
[2025-02-25 21:03:30.654] Exiting application
```

## Troubleshooting

- **Authentication errors**: Make sure you have set the `CELESTIA_NODE_AUTH_TOKEN` environment variable or started your node with `--rpc.skip-auth`
- **Connection errors**: Verify that your Celestia node is running and accessible at the default RPC endpoint (http://localhost:26658)
- **Namespace errors**: Ensure your namespace is valid (will be automatically padded/truncated to 8 bytes)

## License

This project is licensed under the MIT License - see the LICENSE file for details.
