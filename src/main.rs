use anyhow::{Context, Result};
use celestia_rpc::{BlobClient, Client};
use celestia_types::{nmt::Namespace, AppVersion, Blob};
use celestia_rpc::TxConfig;
use rand::RngCore;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use chrono::Local;
use hex;

/// Helper function to get current timestamp for logging
fn log_with_timestamp(message: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] {}", timestamp, message);
}

/// Retrieves blobs from the Celestia network for a specific height and namespace
/// Returns a Result containing a vector of retrieved blobs or an error
async fn retrieve_blobs(client: &Client, height: u64, namespace: &Namespace) -> Result<Vec<Blob>> {
    log_with_timestamp(&format!("Retrieving blobs at height {}", height));
    let blobs = client
        .blob_get_all(height, &[namespace.clone()])
        .await
        .map_err(|e| anyhow::anyhow!("Failed to retrieve blobs: {}", e))?
        .unwrap_or_default();
    log_with_timestamp(&format!("Retrieved {} blobs", blobs.len()));
    Ok(blobs)
}

#[tokio::main]
async fn main() -> Result<()> {
    log_with_timestamp("Starting rollup application");
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        log_with_timestamp("Error: Invalid number of arguments");
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

    log_with_timestamp(&format!(
        "Configuration - Namespace: {}, Number of blobs: {}, Blob size: {} bytes",
        namespace_plaintext, num_blobs, blob_size
    ));

    // Check for authentication token in environment variables
    let token = match env::var("CELESTIA_NODE_AUTH_TOKEN") {
        Ok(token) => {
            log_with_timestamp("Found CELESTIA_NODE_AUTH_TOKEN in environment");
            Some(token)
        },
        Err(_) => {
            log_with_timestamp("Warning: CELESTIA_NODE_AUTH_TOKEN not set");
            println!("Note: CELESTIA_NODE_AUTH_TOKEN not set. Make sure to either:");
            println!("  1. Set CELESTIA_NODE_AUTH_TOKEN environment variable, or");
            println!("  2. Use --rpc.skip-auth when starting your Celestia node");
            None
        }
    };

    // Create namespace from plaintext input
    let namespace_hex = hex::encode(namespace_plaintext);
    let namespace = Namespace::new_v0(&hex::decode(&namespace_hex)?)?;
    log_with_timestamp(&format!("Created namespace '{}'", namespace_plaintext));

    // Initialize Celestia client with WebSocket connection
    log_with_timestamp("Connecting to Celestia node at ws://localhost:26658");
    let client = Client::new("ws://localhost:26658", token.as_deref())
        .await
        .context("Failed to connect to Celestia node")?;
    log_with_timestamp("Successfully connected to Celestia node");

    log_with_timestamp("Starting continuous blob submission. Press Ctrl+C to stop.");
    log_with_timestamp(&format!(
        "Submitting batches of {} blobs, each {} bytes, with namespace '{}'",
        num_blobs, blob_size, namespace_plaintext
    ));

    // Set up graceful shutdown handler for Ctrl+C
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        log_with_timestamp("Received shutdown signal, preparing to stop...");
    })
    .expect("Error setting Ctrl-C handler");

    // Main submission loop
    let mut batch_counter = 0;
    while running.load(Ordering::SeqCst) {
        batch_counter += 1;
        log_with_timestamp(&format!("Starting batch #{} submission to namespace '{}'", batch_counter, namespace_plaintext));
        
        // Generate and submit random blobs
        log_with_timestamp(&format!("Generating random blobs for namespace '{}'", namespace_plaintext));
        let blobs = generate_random_blobs(num_blobs, blob_size, &namespace)?;
        let submitted_data: Vec<Vec<u8>> = blobs.iter().map(|b| b.data.clone()).collect();
        log_with_timestamp(&format!("Generated {} random blobs", blobs.len()));

        // Attempt to submit blobs to the network
        log_with_timestamp("Submitting blobs to network");
        match client.blob_submit(&blobs, TxConfig::default()).await {
            Ok(result_height) => {
                log_with_timestamp(&format!("Batch #{} submitted successfully at height {}", batch_counter, result_height));

                // Wait for a few blocks to ensure the transaction is processed
                log_with_timestamp("Waiting for transaction processing");
                sleep(Duration::from_secs(6)).await;

                log_with_timestamp(&format!("Verifying submission at height {}", result_height));
                // Add retry logic for verification
                let mut retry_count = 0;
                let max_retries = 3;
                
                while retry_count < max_retries {
                    log_with_timestamp(&format!("Verification attempt {} of {}", retry_count + 1, max_retries));
                    match retrieve_blobs(&client, result_height, &namespace).await {
                        Ok(retrieved_blobs) => {
                            if !retrieved_blobs.is_empty() {
                                log_with_timestamp(&format!(
                                    "Found {} blobs at height {}",
                                    retrieved_blobs.len(),
                                    result_height
                                ));

                                let retrieved_data: Vec<Vec<u8>> =
                                    retrieved_blobs.iter().map(|b| b.data.clone()).collect();

                                let mut all_verified = true;
                                for (i, submitted) in submitted_data.iter().enumerate() {
                                    if retrieved_data.iter().any(|retrieved| retrieved == submitted) {
                                        log_with_timestamp(&format!("✅ Blob {} verified successfully", i));
                                    } else {
                                        log_with_timestamp(&format!("❌ Blob {} verification failed", i));
                                        all_verified = false;
                                    }
                                }

                                if all_verified {
                                    log_with_timestamp(&format!("Batch #{} fully verified", batch_counter));
                                    break;
                                }
                            }
                            retry_count += 1;
                            if retry_count < max_retries {
                                log_with_timestamp("Verification incomplete, retrying in 3 seconds");
                                sleep(Duration::from_secs(3)).await;
                            }
                        }
                        Err(e) => {
                            log_with_timestamp(&format!("Error verifying batch: {:?}", e));
                            retry_count += 1;
                            if retry_count < max_retries {
                                log_with_timestamp("Retrying verification in 3 seconds");
                                sleep(Duration::from_secs(3)).await;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                log_with_timestamp(&format!("Error submitting batch #{}: {:?}", batch_counter, e));
                log_with_timestamp("Retrying submission in 3 seconds");
                sleep(Duration::from_secs(3)).await;
            }
        }
        
        log_with_timestamp("Waiting before next batch submission");
        sleep(Duration::from_secs(3)).await;
    }

    log_with_timestamp("Blob submission stopped gracefully");
    Ok(())
}

/// Generates a vector of random blobs with specified parameters
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
