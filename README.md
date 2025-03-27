# Celestia Database

A public database implementation on Celestia that allows storing and retrieving records using a namespace.

## Features

- Store and retrieve records using a key-value model
- Namespace-based organization
- Automatic metadata tracking
- Simple CLI interface
- Configurable block search limit
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

Run the application with a namespace parameter and an optional search limit:

```
cargo run -- <namespace> [search_limit_blocks]
```

The namespace will be padded or truncated to exactly 10 bytes as required by Celestia.

### Search Limit Parameter

The search limit parameter controls how many blocks back the database will search to discover existing data:

- When specified, the database will search up to this many blocks back from the current height to find existing data
- If it finds an existing database, it will use that database's starting block for all operations
- If no existing database is found, it creates a new one at the current height
- This parameter is only used during database initialization and not for subsequent operations

Benefits of using search limit:
- Controls how far back to look for your existing database when restarting
- Reduces startup time by limiting the initial search range
- Once your database is found, operations are fast regardless of search limit

### Available Commands

Once the application is running, you can use the following commands:

- `add <key> <value>` - Add a new record or update an existing one
- `get <key>` - Retrieve a record by key
- `list` - List all records
- `exit` or `quit` - Exit the application
- `help` - Show help message

## Example

```
$ cargo run -- testdb 1000
[2025-02-25 21:03:07.775] Starting Celestia database application
[2025-02-25 21:03:07.776] Block search limit: 1000 blocks
[2025-02-25 21:03:07.776] Configuration - Namespace: testdb
[2025-02-25 21:03:07.776] Connecting to Celestia node...
[2025-02-25 21:03:07.777] Successfully connected to Celestia node
[2025-02-25 21:03:07.778] Searching for existing database (blocks 123789..124789)
[2025-02-25 21:03:07.779] Found existing database at height 123500
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
[2025-02-25 21:03:20.789] Searching for record with key 'user1' (database start: 123500, current height: 124789)
[2025-02-25 21:03:20.790] Found record with key 'user1' at height 124001
Key: user1
Value: {"name": "John Doe", "email": "john@example.com"}
Created: 2025-02-25T21:03:15.123Z
> list
[2025-02-25 21:03:25.321] Listing all records
[2025-02-25 21:03:25.322] Listing all records (database start: 123500, current height: 124789)
[2025-02-25 21:03:25.323] Found 1 records
Key: user1
Value: {"name": "John Doe", "email": "john@example.com"}
Created: 2025-02-25T21:03:15.123Z
---
> exit
[2025-02-25 21:03:30.654] Exiting application
```

Here's another example of starting a brand new database:

```
$ cargo run -- newdb 1000
[2025-02-25 21:04:07.775] Starting Celestia database application
[2025-02-25 21:04:07.776] Block search limit: 1000 blocks
[2025-02-25 21:04:07.776] Configuration - Namespace: newdb
[2025-02-25 21:04:07.776] Connecting to Celestia node...
[2025-02-25 21:04:07.777] Successfully connected to Celestia node
[2025-02-25 21:04:07.778] Searching for existing database (blocks 124000..125000)
[2025-02-25 21:04:07.779] No existing database found within search range
[2025-02-25 21:04:07.780] Creating new database (estimating height 125000)
[2025-02-25 21:04:07.781] Submit response: {"height":5334862}
[2025-02-25 21:04:07.782] Using height: 5334862
[2025-02-25 21:04:07.783] Database created at height 5334862
[2025-02-25 21:04:07.784] Database client initialized
```

## How It Works

1. **Database Initialization**:
   - When you start the database with a namespace, it searches for existing database metadata
   - The search_limit parameter controls how many blocks back to search for existing database metadata
   - If existing metadata is found, the database uses that metadata for all operations
   - If no existing metadata is found, a new database is created at the current block height

2. **Record Storage**:
   - Records are stored as blobs in the Celestia blockchain
   - Each record includes a key, value, creation timestamp, and unique ID
   - Records are serialized to JSON before being stored
   - Metadata is maintained to track the number of records and update timestamps

3. **Record Retrieval**:
   - When retrieving records, the database searches from the database's start height to the current height
   - For key-based lookups, it returns the first matching record found
   - For listing all records, it collects the most recent version of each record
   - Only blocks that could contain your data are searched, making operations efficient

## Troubleshooting

- **Authentication errors**: Make sure you have set the `CELESTIA_NODE_AUTH_TOKEN` environment variable or started your node with `--rpc.skip-auth`
- **Connection errors**: Verify that your Celestia node is running and accessible at the default RPC endpoint (http://localhost:26658)
- **Namespace errors**: Ensure your namespace is valid (will be automatically padded/truncated to 10 bytes)
- **Performance issues**: If searching for records is slow, use a smaller search_limit value

## License

This project is licensed under the MIT License - see the LICENSE file for details.
