use anyhow::{Context, Result};
use celestia_rpc::{BlobClient, Client};
use celestia_types::{nmt::Namespace, AppVersion, Blob};
use celestia_rpc::TxConfig;
use rand::RngCore;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// Retrieves blobs from the Celestia network for a specific height and namespace
/// Returns a Result containing a vector of retrieved blobs or an error
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
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!(
            "Usage: cargo run -- <namespace_plaintext> <number_of_blobs> <blob_size_in_bytes>"
        );
        std::process::exit(1);
    }

    // Extract and parse command line arguments
    let namespace_plaintext = &args[1];
    let num_blobs = args[2]
        .parse::<usize>()
        .context("Invalid number of blobs")?;
    let blob_size = args[3].parse::<usize>().context("Invalid blob size")?;

    // Check for authentication token in environment variables
    let token = match env::var("CELESTIA_NODE_AUTH_TOKEN") {
        Ok(token) => Some(token),
        Err(_) => {
            println!("Note: CELESTIA_NODE_AUTH_TOKEN not set. Make sure to either:");
            println!("  1. Set CELESTIA_NODE_AUTH_TOKEN environment variable, or");
            println!("  2. Use --rpc.skip-auth when starting your Celestia node");
            None
        }
    };

    // Create namespace from plaintext input
    let namespace_hex = hex::encode(namespace_plaintext);
    let namespace = Namespace::new_v0(&hex::decode(&namespace_hex)?)?;

    // Initialize Celestia client with WebSocket connection
    let client = Client::new("ws://localhost:26658", token.as_deref())
        .await
        .context("Failed to connect to Celestia node")?;

    println!("Starting continuous blob submission. Press Ctrl+C to stop.");
    println!(
        "Submitting batches of {} blobs, each {} bytes, with namespace '{}'",
        num_blobs, blob_size, namespace_plaintext
    );

    // Set up graceful shutdown handler for Ctrl+C
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        println!("\nShutting down gracefully...");
    })
    .expect("Error setting Ctrl-C handler");

    // Main submission loop
    while running.load(Ordering::SeqCst) {
        // Generate and submit random blobs
        let blobs = generate_random_blobs(num_blobs, blob_size, &namespace)?;
        let submitted_data: Vec<Vec<u8>> = blobs.iter().map(|b| b.data.clone()).collect();

        // Attempt to submit blobs to the network
        match client.blob_submit(&blobs, TxConfig::default()).await {
            Ok(result_height) => {
                println!("Batch submitted successfully!");
                println!("Result height: {}", result_height);

                // Wait for a few blocks to ensure the transaction is processed
                sleep(Duration::from_secs(6)).await;

                println!("Checking height {}...", result_height);
                // Add retry logic for verification
                let mut retry_count = 0;
                let max_retries = 3;
                
                while retry_count < max_retries {
                    match retrieve_blobs(&client, result_height, &namespace).await {
                        Ok(retrieved_blobs) => {
                            if !retrieved_blobs.is_empty() {
                                println!(
                                    "Found {} blobs at height {}",
                                    retrieved_blobs.len(),
                                    result_height
                                );

                                let retrieved_data: Vec<Vec<u8>> =
                                    retrieved_blobs.iter().map(|b| b.data.clone()).collect();

                                let mut all_verified = true;
                                for (i, submitted) in submitted_data.iter().enumerate() {
                                    if retrieved_data.iter().any(|retrieved| retrieved == submitted) {
                                        println!("✅ Blob {} verified successfully", i);
                                    } else {
                                        println!("❌ Blob {} verification failed", i);
                                        all_verified = false;
                                    }
                                }

                                if all_verified {
                                    break;
                                }
                            }
                            retry_count += 1;
                            if retry_count < max_retries {
                                sleep(Duration::from_secs(3)).await;
                            }
                        }
                        Err(e) => {
                            eprintln!("Error verifying batch: {:?}", e);
                            retry_count += 1;
                            if retry_count < max_retries {
                                sleep(Duration::from_secs(3)).await;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error submitting batch: {:?}", e);
                // Add delay before retrying submission
                sleep(Duration::from_secs(3)).await;
            }
        }
        
        // Add delay between batch submissions
        sleep(Duration::from_secs(3)).await;
    }

    println!("Blob submission stopped.");
    Ok(())
}

/// Generates a vector of random blobs with specified parameters
/// Parameters:
/// - num_blobs: Number of blobs to generate
/// - blob_size: Size of each blob in bytes
/// - namespace: Namespace for the blobs
fn generate_random_blobs(
    num_blobs: usize,
    blob_size: usize,
    namespace: &Namespace,
) -> Result<Vec<Blob>> {
    let mut rng = rand::thread_rng();
    let mut blobs = Vec::with_capacity(num_blobs);

    // Generate random data for each blob
    for _ in 0..num_blobs {
        let mut random_data = vec![0u8; blob_size];
        rng.fill_bytes(&mut random_data);
        let blob = Blob::new(namespace.clone(), random_data, AppVersion::V2)
            .map_err(|e| anyhow::anyhow!("Failed to create blob: {}", e))?;
        blobs.push(blob);
    }

    Ok(blobs)
}
