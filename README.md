# dummy_rollup

This is a dummy rollup project to test the Celenium API.

## Prerequisites

- Rust
- Celestia light node running

### Setting up Celestia Light Node

To start a Celestia light node, use the following command:

```shell
celestia light start --core.ip rpc-mocha.pops.one --p2p.network mocha --rpc.skip-auth
```

## Usage

### Build

To build the project, run:

```shell
cargo build
```

### Run

To run the project, use the following command:

```shell
cargo run -- <namespace_plaintext> <number_of_blobs> <blob_size_in_bytes>
```

#### Example

```shell
cargo run -- junkdata 10 1000
```

This command posts continuously to the "junkdata" namespace with 10 blobs of 1000 bytes each.

### Detailed Explanation

- `<namespace_plaintext>`: The plaintext string that will be converted to a hexadecimal namespace.
- `<number_of_blobs>`: The number of blobs to generate and submit in each batch.
- `<blob_size_in_bytes>`: The size of each blob in bytes.

### Continuous Blob Submission

The program will continuously submit batches of blobs to the Celestia node every 5 seconds. To stop the submission, press `Ctrl+C`. The program will handle the shutdown gracefully.

### Error Handling

If there are any issues with the submission, the program will print an error message and continue attempting to submit new batches.

## Dependencies

The project uses the following dependencies:

- `anyhow`: For error handling.
- `celestia_rpc`: For interacting with the Celestia node.
- `celestia_types`: For handling Celestia-specific types like `Namespace` and `Blob`.
- `rand`: For generating random data for the blobs.
- `tokio`: For asynchronous runtime.
- `ctrlc`: For handling `Ctrl+C` interrupts.

## Example Output

When running the example command, you should see output similar to the following:

```
Starting continuous blob submission. Press Ctrl+C to stop.
Submitting batches of 10 blobs, each 1000 bytes, with namespace 'junkdata'
Batch submitted successfully!
Result: <submission_result>
...
```

If there is an error, you will see:

```
Error submitting batch: <error_message>
```

## Notes

- Ensure that the Celestia light node is running and accessible at `http://localhost:26658`.
- Adjust the namespace, number of blobs, and blob size according to your requirements.
