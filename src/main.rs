use std::env;
use std::process;

mod commands;
mod utils;
mod models;
mod storage;

fn main() {
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
    match command {
        "dump" => commands::handle_dump(cmd_args),
        "store" => commands::handle_store(cmd_args),
        _ => {
            eprintln!("{}Error: Unknown command '{}'{}", utils::RED, command, utils::RESET);
            utils::print_usage();
            process::exit(1);
        }
    }
}