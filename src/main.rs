mod database;
mod schema;

use anyhow::{Context, Result};
use celestia_rpc::Client;
use celestia_types::nmt::Namespace;
use chrono::Local;
use database::{Database, DatabaseClient};
use schema::DatabaseError;
use std::env;
use std::str::FromStr;
use hex;

/// Helper function to get current timestamp for logging
fn log_with_timestamp(message: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] {}", timestamp, message);
}

#[derive(Debug)]
enum Command {
    Create(Vec<u8>),
    Read(String),
    Update(String, Vec<u8>),
    Delete(String),
    List,
}

impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        match parts.get(0).map(|s| *s) {
            Some("create") => {
                if parts.len() != 2 {
                    return Err("Usage: create <data>".to_string());
                }
                Ok(Command::Create(parts[1].as_bytes().to_vec()))
            }
            Some("read") => {
                if parts.len() != 2 {
                    return Err("Usage: read <id>".to_string());
                }
                Ok(Command::Read(parts[1].to_string()))
            }
            Some("update") => {
                if parts.len() != 3 {
                    return Err("Usage: update <id> <new_data>".to_string());
                }
                Ok(Command::Update(parts[1].to_string(), parts[2].as_bytes().to_vec()))
            }
            Some("delete") => {
                if parts.len() != 2 {
                    return Err("Usage: delete <id>".to_string());
                }
                Ok(Command::Delete(parts[1].to_string()))
            }
            Some("list") => Ok(Command::List),
            _ => Err("Unknown command. Available commands: create, read, update, delete, list".to_string()),
        }
    }
}

async fn handle_command(db: &impl Database, cmd: Command) -> Result<(), DatabaseError> {
    match cmd {
        Command::Create(data) => {
            let id = db.create_record(data).await?;
            log_with_timestamp(&format!("Created record with ID: {}", id));
        }
        Command::Read(id) => {
            let record = db.read_record(&id).await?;
            log_with_timestamp(&format!(
                "Record {}: created={}, updated={}, data={}",
                record.id,
                record.created_at,
                record.updated_at,
                String::from_utf8_lossy(&record.data)
            ));
        }
        Command::Update(id, data) => {
            db.update_record(&id, data).await?;
            log_with_timestamp(&format!("Updated record {}", id));
        }
        Command::Delete(id) => {
            db.delete_record(&id).await?;
            log_with_timestamp(&format!("Deleted record {}", id));
        }
        Command::List => {
            let records = db.list_records().await?;
            log_with_timestamp(&format!("Found {} records:", records.len()));
            for record in records {
                println!(
                    "  {} (created={}, updated={}): {}",
                    record.id,
                    record.created_at,
                    record.updated_at,
                    String::from_utf8_lossy(&record.data)
                );
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    log_with_timestamp("Starting Celestia database application");
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        log_with_timestamp("Error: Invalid number of arguments");
        eprintln!("Usage: cargo run -- <namespace_plaintext>");
        std::process::exit(1);
    }

    // Extract and parse command line arguments
    let namespace_plaintext = &args[1];

    log_with_timestamp(&format!(
        "Configuration - Namespace: {}",
        namespace_plaintext
    ));

    // Check for authentication token in environment variables
    let token = match env::var("CELESTIA_NODE_AUTH_TOKEN") {
        Ok(token) => {
            log_with_timestamp("Found CELESTIA_NODE_AUTH_TOKEN in environment");
            Some(token)
        }
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

    // Create database client
    let db = DatabaseClient::new(client, namespace).await?;
    log_with_timestamp("Database client initialized");

    println!("\nAvailable commands:");
    println!("  create <data>      - Create a new record");
    println!("  read <id>          - Read a record by ID");
    println!("  update <id> <data> - Update a record");
    println!("  delete <id>        - Delete a record");
    println!("  list              - List all records");
    println!("  exit              - Exit the application");

    // Main command loop
    loop {
        print!("\n> ");
        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input == "exit" {
            break;
        }

        match Command::from_str(input) {
            Ok(cmd) => {
                if let Err(e) = handle_command(&db, cmd).await {
                    log_with_timestamp(&format!("Error: {}", e));
                }
            }
            Err(e) => {
                log_with_timestamp(&format!("Error: {}", e));
            }
        }
    }

    log_with_timestamp("Database application stopped");
    Ok(())
}
