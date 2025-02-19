# dummy_rollup

This is a dummy rollup project to test posting and retrieving data from Celestia.

## Prerequisites

- Rust
- Celestia light node running

### Setting up Celestia light node

To start a Celestia light node, use the following command:

```shell
celestia light start --core.ip rpc-mocha.pops.one --core.port 9090 --p2p.network mocha --rpc.skip-auth
```

### Environment Variables

- `CELESTIA_NODE_AUTH_TOKEN`: Authentication token for the Celestia node
  - If not set, ensure you use `--rpc.skip-auth` when starting your Celestia node

## Dependencies

The project uses the following dependencies:

- `anyhow`: For error handling
- `celestia_rpc`: For interacting with the Celestia node
- `celestia_types`: For handling Celestia-specific types
- `rand`: For generating random data for the blobs
- `tokio`: For asynchronous runtime
- `ctrlc`: For handling `Ctrl+C` interrupts

## Usage

### Build

```shell
cargo build
```

### Run

```shell
cargo run -- <namespace_plaintext> <number_of_blobs> <blob_size_in_bytes>
```

Parameters:
- `<namespace_plaintext>`: The namespace for your blobs
- `<number_of_blobs>`: Number of blobs per batch
- `<blob_size_in_bytes>`: Size of each blob in bytes

#### Example

```shell
cargo run -- testing 9 197278
```

This will:
1. Connect to your local Celestia node
2. Create a namespace called "testing"
3. Submit batches of 9 blobs (197278 bytes each)
4. Verify each submission by retrieving and checking the blobs
5. Continue submitting new batches with a short delay between them

#### Example Output

```
[2025-02-19 12:11:18.965] Starting rollup application
[2025-02-19 12:11:18.967] Configuration - Namespace: testing, Number of blobs: 9, Blob size: 197278 bytes
[2025-02-19 12:11:18.973] Successfully connected to Celestia node
[2025-02-19 12:11:45.845] Batch #1 submitted successfully at height 4770599
[2025-02-19 12:11:53.321] Found 9 blobs at height 4770599
[2025-02-19 12:11:53.321] ✅ Blob 0 verified successfully
...
[2025-02-19 12:11:53.321] ✅ Blob 8 verified successfully
[2025-02-19 12:11:53.321] Batch #1 fully verified
```

The program will continue running and submitting batches until interrupted with Ctrl+C.
