# dummy_rollup

This is a dummy rollup project to test the Celenium API.
## Prerequisites

- Rust
- Celestia light node running

```shell
celestia light start --core.ip rpc-mocha.pops.one --p2p.network mocha --rpc.skip-auth
```

## Usage

### Build

```shell
cargo build
```

### Run

#### Example

```shell
cargo run -- <namespace_plaintext> <number_of_blobs> <blob_size_in_bytes>
```

#### Working example

```shell
cargo run -- namespace 10 1000
```

This posts to the "namespace" namespace with 10 blobs of 1000 bytes each.
