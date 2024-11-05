use anyhow::{Context, Result};
use celestia_rpc::{BlobClient, Client};
use celestia_types::{nmt::Namespace, AppVersion, Blob, TxConfig};
use rand::RngCore;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

async fn retrieve_blobs(client: &Client, height: u64, namespace: &Namespace) -> Result<Vec<Blob>> {
    let blobs = client
        .blob_get_all(height, &[namespace.clone()])
        .await
        .map_err(|e| anyhow::anyhow!("Failed to retrieve blobs: {}", e))?
        .unwrap_or_default();
    Ok(blobs)
}

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

    println!("Starting continuous blob submission. Press Ctrl+C to stop.");
    println!(
        "Submitting batches of {} blobs, each {} bytes, with namespace '{}'",
        num_blobs, blob_size, namespace_plaintext
    );

    // Set up Ctrl+C handler
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        println!("\nShutting down gracefully...");
    })
    .expect("Error setting Ctrl-C handler");

    while running.load(Ordering::SeqCst) {
        let blobs = generate_random_blobs(num_blobs, blob_size, &namespace)?;
        let submitted_data: Vec<Vec<u8>> = blobs.iter().map(|b| b.data.clone()).collect();

        match client.blob_submit(&blobs, TxConfig::default()).await {
            Ok(result_height) => {
                println!("Batch submitted successfully!");
                println!("Result height: {}", result_height);

                // Wait a bit for the block to be processed
                sleep(Duration::from_secs(2)).await;

                println!("Checking height {}...", result_height);
                match retrieve_blobs(&client, result_height, &namespace).await {
                    Ok(retrieved_blobs) => {
                        if !retrieved_blobs.is_empty() {
                            println!(
                                "Found {} blobs at height {}",
                                retrieved_blobs.len(),
                                result_height
                            );

                            // Verify the retrieved blobs match what we submitted
                            let retrieved_data: Vec<Vec<u8>> = retrieved_blobs
                                .iter()
                                .map(|b| b.data.clone())
                                .collect();

                            for (i, (submitted, retrieved)) in submitted_data
                                .iter()
                                .zip(retrieved_data.iter())
                                .enumerate()
                            {
                                if submitted == retrieved {
                                    println!("✅ Blob {} verified successfully", i);
                                } else {
                                    println!("❌ Blob {} verification failed", i);
                                }
                            }
                        } else {
                            println!("No blobs found at height {}", result_height);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error retrieving blobs at height {}: {:?}", result_height, e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error submitting batch: {:?}", e);
            }
        }

        sleep(Duration::from_secs(5)).await;
    }

    println!("Blob submission stopped.");
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
        let blob = Blob::new(namespace.clone(), random_data, AppVersion::V2)
            .map_err(|e| anyhow::anyhow!("Failed to create blob: {}", e))?;
        blobs.push(blob);
    }

    Ok(blobs)
}
