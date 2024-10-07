use anyhow::{Context, Result};
use celestia_rpc::{BlobClient, Client};
use celestia_types::{nmt::Namespace, Blob, TxConfig};
use rand::RngCore;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!(
            "Usage: cargo run -- <namespace_plaintext> <number_of_blobs> <blob_size_in_bytes>"
        );
        std::process::exit(1);
    }

    let namespace_plaintext = &args[1];
    let num_blobs = args[2]
        .parse::<usize>()
        .context("Invalid number of blobs")?;
    let blob_size = args[3].parse::<usize>().context("Invalid blob size")?;

    let namespace_hex = hex::encode(namespace_plaintext);
    let namespace = Namespace::new_v0(&hex::decode(&namespace_hex)?)?;

    let client = Client::new("http://localhost:26658", None)
        .await
        .context("Failed to connect to Celestia node")?;

    let blobs = generate_random_blobs(num_blobs, blob_size, &namespace)?;

    println!(
        "Posting {} blobs of size {} bytes each with namespace '{}'...",
        num_blobs, blob_size, namespace_plaintext
    );
    let result = BlobClient::blob_submit(&client, blobs.as_slice(), TxConfig::default()).await?;

    println!("Blobs submitted successfully!");
    println!("Result: {:?}", result);

    Ok(())
}

fn generate_random_blobs(
    num_blobs: usize,
    blob_size: usize,
    namespace: &Namespace,
) -> Result<Vec<Blob>> {
    let mut rng = rand::thread_rng();
    let mut blobs = Vec::with_capacity(num_blobs);

    for _ in 0..num_blobs {
        let mut random_data = vec![0u8; blob_size];
        rng.fill_bytes(&mut random_data);
        let blob = Blob::new(namespace.clone(), random_data)?;
        blobs.push(blob);
    }

    Ok(blobs)
}
