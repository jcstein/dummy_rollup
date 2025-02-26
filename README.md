# Celestia Database

A public database implementation on Celestia that allows storing and retrieving records using a namespace.

## Features

- Store and retrieve records using a key-value model
- Namespace-based organization
- Automatic metadata tracking
- Simple CLI interface
- Configurable starting block height
- Efficient record search and retrieval

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

Run the application with a namespace parameter and an optional start height:

```
cargo run -- <namespace> [start_height]
```

The namespace will be padded or truncated to exactly 8 bytes as required by Celestia.

### Start Height Parameter

The start height parameter allows you to specify the block height from which the database should begin:

- If you provide a start height, the database will begin at that block height
- If you don't provide a start height, the database will start at the current block height of the Celestia chain
- When restarting the application with the same namespace, it will automatically find and use the original start height

This is useful for:
- Creating a database that includes historical data
- Ensuring consistent data access across restarts
- Setting up multiple databases with different starting points

### Available Commands

Once the application is running, you can use the following commands:

- `add <key> <value>` - Add a new record or update an existing one
- `get <key>` - Retrieve a record by key
- `list` - List all records
- `exit` or `quit` - Exit the application
- `help` - Show help message

## Example

```
$ cargo run -- testdb 100000
[2025-02-25 21:03:07.775] Starting Celestia database application
[2025-02-25 21:03:07.776] Configuration - Namespace: testdb, Start Height: 100000
[2025-02-25 21:03:07.776] Connecting to Celestia node...
[2025-02-25 21:03:07.777] Successfully connected to Celestia node
[2025-02-25 21:03:07.778] Checking for existing metadata at height 100000
[2025-02-25 21:03:07.779] No metadata found at height 100000, will create new
[2025-02-25 21:03:07.780] Creating new metadata with start height 100000
[2025-02-25 21:03:07.781] Database client initialized

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
[2025-02-25 21:03:20.789] Searching for record with key 'user1' (start height: 100000)
[2025-02-25 21:03:20.790] Found record with key 'user1' at height 100001
Key: user1
Value: {"name": "John Doe", "email": "john@example.com"}
Created: 2025-02-25T21:03:15.123Z
> list
[2025-02-25 21:03:25.321] Listing all records (start height: 100000)
[2025-02-25 21:03:25.322] Found 1 records
Key: user1
Value: {"name": "John Doe", "email": "john@example.com"}
Created: 2025-02-25T21:03:15.123Z
---
> exit
[2025-02-25 21:03:30.654] Exiting application
```

## How It Works

1. **Database Initialization**:
   - When you start the database with a namespace and optional start height, it first checks if metadata already exists for that namespace
   - If metadata exists, it loads the existing configuration
   - If no metadata is found, it creates new metadata with the specified start height (or current height if none specified)

2. **Record Storage**:
   - Records are stored as blobs in the Celestia blockchain
   - Each record includes a key, value, creation timestamp, and unique ID
   - Records are serialized to JSON before being stored

3. **Record Retrieval**:
   - When retrieving records, the database searches from the start height to the current height
   - For key-based lookups, it returns the first matching record found
   - For listing all records, it collects the most recent version of each record

## Troubleshooting

- **Authentication errors**: Make sure you have set the `CELESTIA_NODE_AUTH_TOKEN` environment variable or started your node with `--rpc.skip-auth`
- **Connection errors**: Verify that your Celestia node is running and accessible at the default RPC endpoint (http://localhost:26658)
- **Namespace errors**: Ensure your namespace is valid (will be automatically padded/truncated to 8 bytes)
- **Start height errors**: If you specify a start height, make sure it's a valid block height on the Celestia chain
- **Performance issues**: If searching for records is slow, try using a start height closer to when your records were created

## License

This project is licensed under the MIT License - see the LICENSE file for details.
