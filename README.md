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
## Dependencies

The project uses the following dependencies:

- `anyhow`: For error handling.
- [`celestia_rpc`](https://crates.io/crates/celestia-rpc): For interacting with the Celestia node.
- [`celestia_types`](https://crates.io/crates/celestia-types): For handling Celestia-specific types like `Namespace` and `Blob`.
- `rand`: For generating random data for the blobs.
- `tokio`: For asynchronous runtime.
- `ctrlc`: For handling `Ctrl+C` interrupts.

## Usage

### Build

To build the dummy_rollup:

```shell
cargo build
```

### Run

To run the dummy_rollup:

```shell
cargo run -- <namespace_plaintext> <number_of_blobs> <blob_size_in_bytes>
```

Where:
- `<namespace_plaintext>`: The plaintext string that will be converted to a hexadecimal namespace.
- `<number_of_blobs>`: The number of blobs to generate and submit in each batch.
- `<blob_size_in_bytes>`: The size of each blob in bytes.

The program will continuously submit batches of blobs to the Celestia node every 5 seconds. To stop the submission, press `Ctrl+C`. The program will handle the shutdown gracefully.

#### Example

```shell
cargo run -- junkdata 10 1000
```

This command posts continuously to the "junkdata" namespace with 10 blobs of 1000 bytes each.

If there are any issues with the submission, the program will print an error message and continue attempting to submit new batches.

#### Example output

When running the example command, you should see output similar to the following:

```
Starting continuous blob submission. Press Ctrl+C to stop.
Submitting batches of 10 blobs, each 197278 bytes, with namespace 'junkdata'
Batch submitted successfully!
Result height: 3216604
Checking height 3216604...
Found 9 blobs at height 3216604
✅ Blob 0 verified successfully
✅ Blob 1 verified successfully
✅ Blob 2 verified successfully
✅ Blob 3 verified successfully
✅ Blob 4 verified successfully
✅ Blob 5 verified successfully
✅ Blob 6 verified successfully
✅ Blob 7 verified successfully
✅ Blob 8 verified successfully
✅ Blob 9 verified successfully
...
```

If there is an error, you will see:

```
Error submitting batch: <error_message>
```
