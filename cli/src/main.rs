use std::env;
use std::path::PathBuf;
use std::process;

// Import the core library
use chronork_core::storage::StorageManager;

mod commands;
mod utils;

/// Resolves the base storage path depending on the runtime environment.
fn resolve_cli_path() -> PathBuf {
    // 1. Check for the PREFIX environment variable (Termux ecosystem)
    if let Ok(prefix) = env::var("PREFIX") {
        if !prefix.is_empty() {
            return PathBuf::from(format!("{}/var/lib/chronork/logs/", prefix));
        }
    }
    
    // 2. Fallback for standard Linux distributions (XDG Base Directory Specification)
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(format!("{}/.local/share/chronork/logs/", home))
}

fn main() {
    // Determine the environment's storage path and initialize the core manager
    let base_path = resolve_cli_path();
    let storage_manager = StorageManager::new(base_path);

    // Collect raw arguments. args[0] is the binary path, args[1] is the first real argument.
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        utils::print_usage();
        process::exit(1);
    }

    let first_arg = &args[1];
    
    let command: &str;
    let cmd_args: &[String]; // Using slices prevents allocating a new Vec just to pass arguments

    // UX Feature: If the first argument is a flag (e.g., "-y" or "-f"), 
    // implicitly route it to the 'dump' command.
    if first_arg.starts_with('-') {
        command = "dump";
        cmd_args = &args[1..];
    } else {
        command = first_arg.as_str();
        // Safe slice extraction. If args.len() == 2, this safely results in an empty slice [].
        cmd_args = &args[2..];
    }

    // Command Routing Logic
    // We now pass the environment-aware `storage_manager` directly into the handlers
    match command {
        "dump" => commands::handle_dump(cmd_args, &storage_manager),
        "store" => commands::handle_store(cmd_args, &storage_manager),
        _ => {
            eprintln!("{}Error: Unknown command '{}'{}", utils::RED, command, utils::RESET);
            utils::print_usage();
            process::exit(1);
        }
    }
}