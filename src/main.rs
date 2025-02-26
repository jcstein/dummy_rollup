mod database;
mod schema;

use celestia_rpc::Client;
use chrono::Local;
use database::DatabaseClient;
use schema::{DatabaseError, Record};
use std::io::{self, BufRead, Write};
use std::str::FromStr;

/// Helper function to get current timestamp for logging
fn log_with_timestamp(message: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] {}", timestamp, message);
}

enum Command {
    Add(String, String),
    Get(String),
    List,
    Exit,
    Help,
}

impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.trim().split_whitespace().collect();
        if parts.is_empty() {
            return Err("Empty command".to_string());
        }

        match parts[0].to_lowercase().as_str() {
            "add" => {
                if parts.len() < 3 {
                    return Err("Usage: add <key> <value>".to_string());
                }
                let key = parts[1].to_string();
                let value = parts[2..].join(" ");
                Ok(Command::Add(key, value))
            }
            "get" => {
                if parts.len() != 2 {
                    return Err("Usage: get <key>".to_string());
                }
                Ok(Command::Get(parts[1].to_string()))
            }
            "list" => Ok(Command::List),
            "exit" | "quit" => Ok(Command::Exit),
            "help" => Ok(Command::Help),
            _ => Err(format!("Unknown command: {}", parts[0])),
        }
    }
}

async fn handle_command(db: &DatabaseClient, cmd: Command) -> Result<(), DatabaseError> {
    match cmd {
        Command::Add(key, value) => {
            log_with_timestamp(&format!("Adding record with key '{}'", key));
            let record = Record::new(key, value);
            db.add_record(record).await?;
            log_with_timestamp("Record added successfully");
        }
        Command::Get(key) => {
            log_with_timestamp(&format!("Retrieving record with key '{}'", key));
            match db.get_record(&key).await? {
                Some(record) => {
                    println!("Key: {}", record.key);
                    println!("Value: {}", record.value);
                    println!("Created: {}", record.created_at);
                    if let Some(updated) = record.updated_at {
                        println!("Updated: {}", updated);
                    }
                }
                None => log_with_timestamp(&format!("No record found with key '{}'", key)),
            }
        }
        Command::List => {
            log_with_timestamp("Listing all records");
            let records = db.list_records().await?;
            if records.is_empty() {
                println!("No records found");
            } else {
                for record in records {
                    println!("Key: {}", record.key);
                    println!("Value: {}", record.value);
                    println!("Created: {}", record.created_at);
                    if let Some(updated) = record.updated_at {
                        println!("Updated: {}", updated);
                    }
                    println!("---");
                }
            }
        }
        Command::Exit => {
            log_with_timestamp("Exiting application");
            std::process::exit(0);
        }
        Command::Help => {
            println!("\nAvailable commands:");
            println!("  add <key> <value>  - Add a new record or update existing one");
            println!("  get <key>          - Retrieve a record by key");
            println!("  list               - List all records");
            println!("  exit               - Exit the application");
            println!("  help               - Show this help message");
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_with_timestamp("Starting Celestia database application");
    
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        log_with_timestamp("Error: Invalid number of arguments");
        println!("Usage: cargo run -- <namespace_plaintext> [start_height]");
        return Ok(());
    }
    
    let namespace_plaintext = &args[1];
    let start_height = if args.len() == 3 {
        match args[2].parse::<u64>() {
            Ok(height) => {
                log_with_timestamp(&format!("Configuration - Namespace: {}, Start Height: {}", namespace_plaintext, height));
                Some(height)
            },
            Err(_) => {
                log_with_timestamp("Error: Invalid start height, must be a positive number");
                println!("Usage: cargo run -- <namespace_plaintext> [start_height]");
                return Ok(());
            }
        }
    } else {
        log_with_timestamp(&format!("Configuration - Namespace: {}", namespace_plaintext));
        None
    };
    
    // Ensure namespace is exactly 8 bytes (Celestia requirement)
    let mut namespace_bytes = namespace_plaintext.as_bytes().to_vec();
    if namespace_bytes.len() > 8 {
        namespace_bytes.truncate(8);
    } else {
        while namespace_bytes.len() < 8 {
            namespace_bytes.push(0);
        }
    }
    
    // Create database client
    let auth_token = std::env::var("CELESTIA_NODE_AUTH_TOKEN").ok();
    log_with_timestamp("Connecting to Celestia node...");
    let client = Client::new("http://localhost:26658", auth_token.as_deref())
        .await
        .map_err(|e| Box::new(DatabaseError::CelestiaError(e.to_string())) as Box<dyn std::error::Error>)?;
    log_with_timestamp("Successfully connected to Celestia node");
    
    let db_client = DatabaseClient::new(client, namespace_bytes, start_height).await?;
    log_with_timestamp("Database client initialized");

    println!("\nAvailable commands:");
    println!("  add <key> <value>  - Add a new record or update existing one");
    println!("  get <key>          - Retrieve a record by key");
    println!("  list               - List all records");
    println!("  exit               - Exit the application");
    println!("  help               - Show this help message");
    println!("\nEnter commands below:");

    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut input = String::new();

    loop {
        print!("> ");
        io::stdout().flush()?;
        input.clear();
        handle.read_line(&mut input)?;

        match Command::from_str(&input) {
            Ok(cmd) => {
                if let Err(e) = handle_command(&db_client, cmd).await {
                    log_with_timestamp(&format!("Error: {}", e));
                }
            }
            Err(e) => {
                log_with_timestamp(&format!("Command error: {}", e));
            }
        }
    }
}
