use std::io::{self, Write};

use chronork_core::models::LogEntry;
use chronork_core::storage::StorageManager;
use chronork_core::utils as core_utils;
use crate::utils as cli_utils;

/// 1. Tokenize the string, respecting spaces inside quotes
/// 2. Process the tokens to separate tags from content
pub fn parse_input(raw_input: &str) -> (String, Vec<String>) {
    let mut content_parts = Vec::new();
    let mut tags = Vec::new();
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut in_quotes = false;

    // Tokenize
    for c in raw_input.chars() {
        if c == '"' {
            in_quotes = !in_quotes;
            current_token.push(c);
        } else if c.is_whitespace() && !in_quotes {
            if !current_token.is_empty() {
                tokens.push(current_token.clone());
                current_token.clear();
            }
        } else {
            current_token.push(c);
        }
    }
    if !current_token.is_empty() {
        tokens.push(current_token);
    }

    // Process tokens
    for token in tokens {
        let hash_pos = token.find('#');
        
        let is_tag = match hash_pos {
            Some(0) => true,
            Some(1) => token.chars().next().map_or(false, |c| c.is_ascii_punctuation()),
            _ => false,
        };

        if is_tag {
            let mut tag = token[hash_pos.unwrap() + 1..].to_string();
            
            while tag.ends_with(|c: char| c.is_ascii_punctuation()) {
                tag.pop();
            }
            
            if !tag.is_empty() {
                tags.push(tag);
            }
        } else {
            content_parts.push(token);
        }
    }

    if tags.is_empty() {
        tags.push("general".to_string());
    }

    (content_parts.join(" "), tags)
}

fn print_category(
    date: &str,
    category_name: &str,
    entries: &[LogEntry],
    target_tags: &[String],
) {
    let color = cli_utils::get_color(category_name);
    
    for entry in entries {
        let mut match_found = target_tags.is_empty();
        
        if !match_found {
            for target in target_tags {
                if entry.tags.iter().any(|tag| tag == target) {
                    match_found = true;
                    break;
                }
            }
        }

        if !match_found {
            continue;
        }

        print!("[{}] {}{:<12}{}: {}", date, color, category_name, cli_utils::RESET, entry.content);
        
        print!("{} [", cli_utils::BLUE);
        for (i, tag) in entry.tags.iter().enumerate() {
            print!("#{}", tag);
            if i < entry.tags.len() - 1 {
                print!(" ");
            }
        }
        println!("]{}", cli_utils::RESET);
    }
}

// --- DUMP COMMAND ---
pub fn handle_dump(args: &[String], storage: &StorageManager) {
    let mut y = String::new();
    let mut m = String::new();
    let mut d = String::new();
    let mut f = Vec::new();
    let mut t = Vec::new();
    let mut mode = "";

    // State Machine Argument Parser
    for arg in args {
        match arg.as_str() {
            "-y" | "-m" | "-d" | "-f" | "-t" => {
                mode = arg;
                continue;
            }
            _ => {}
        }

        match mode {
            "-y" => y = arg.clone(),
            "-m" => m = arg.clone(),
            "-d" => d = arg.replace('.', "-"),
            "-f" => f.push(arg.clone()),
            "-t" => t.push(arg.clone()),
            _ => {}
        }
    }

    // Path Traversal Sanitization: Enforce strict digit boundaries
    if !y.is_empty() {
        if y.len() != 4 || !y.chars().all(|c| c.is_ascii_digit()) {
            eprintln!("{}Error: Year must be exactly 4 digits.{}", cli_utils::RED, cli_utils::RESET);
            return;
        }
    }

    if !m.is_empty() {
        let sanitized_m: String = m.chars().filter(|c| c.is_ascii_digit()).collect();
        if sanitized_m.is_empty() || sanitized_m.len() > 2 {
            eprintln!("{}Error: Month must be 1 or 2 digits.{}", cli_utils::RED, cli_utils::RESET);
            return;
        }
        // Normalize to two digits ("7" -> "07")
        m = format!("{:02}", sanitized_m.parse::<u8>().unwrap_or(1));
    }

    // Determine which categories to show
    let mut show_a = false;
    let mut show_l = false;
    let mut show_m = false;
    let mut show_i = false;

    if f.is_empty() {
        show_a = true; show_l = true; show_m = true; show_i = true;
    } else {
        for cat in &f {
            if let Some(c) = cat.chars().next().map(|ch| ch.to_ascii_lowercase()) {
                match c {
                    'a' => show_a = true,
                    'l' => show_l = true,
                    'm' => show_m = true,
                    'i' => show_i = true,
                    _ => {}
                }
            }
        }
    }

    // Delegate directory traversal logic entirely to the Storage Manager by generating the requested dates.
    let mut dates_to_process = Vec::new();
    let time_filter_applied = !y.is_empty() || !m.is_empty() || !d.is_empty();

    if !time_filter_applied {
        dates_to_process.push(core_utils::get_today_date());
    } else if !d.is_empty() {
        // Sanitize exact date input length before pushing
        if d.len() == 10 {
            dates_to_process.push(d);
        }
    } else {
        let target_year = if y.is_empty() { core_utils::get_current_year() } else { y };
        let months = if m.is_empty() { 1..=12 } else { m.parse::<u8>().unwrap()..=m.parse::<u8>().unwrap() };
        
        for month in months {
            for day in 1..=31 {
                dates_to_process.push(format!("{}-{:02}-{:02}", target_year, month, day));
            }
        }
    }

    // Execute Output via Core Storage Engine
    for date in dates_to_process {
        match storage.load(&date) {
            Ok(log) => {
                if log.metadata.updated_at == 0 {
                    continue; // Skip silently; file does not exist
                }

                if show_a && !log.logs.achievements.is_empty() {
                    print_category(&date, "Achievement", &log.logs.achievements, &t);
                }
                if show_l && !log.logs.learnings.is_empty() {
                    print_category(&date, "Learning", &log.logs.learnings, &t);
                }
                if show_m && !log.logs.mistakes.is_empty() {
                    print_category(&date, "Mistake", &log.logs.mistakes, &t);
                }
                if show_i && !log.logs.ideas.is_empty() {
                    print_category(&date, "Idea", &log.logs.ideas, &t);
                }
            }
            Err(e) => {
                eprintln!("{}Error reading log for {}: {}{}", cli_utils::RED, date, e, cli_utils::RESET);
            }
        }
    }
}

// --- STORE COMMAND ---
pub fn handle_store(args: &[String], storage: &StorageManager) {
    let date = core_utils::get_today_date();
    
    let mut log = match storage.load(&date) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("{}Error loading log: {}{}", cli_utils::RED, e, cli_utils::RESET);
            return;
        }
    };

    log.metadata.updated_at = core_utils::get_current_timestamp();

    let mut interactive = true;
    for arg in args {
        if matches!(arg.as_str(), "-a" | "-l" | "-m" | "-i") {
            interactive = false;
            break;
        }
    }

    if interactive {
        println!("{}--- Recording Worklog for {} ---{}", cli_utils::CYAN, date, cli_utils::RESET);
        println!("{}Tip: Leave empty and press Enter to move to the next category. Press Ctrl+D to exit early.{}", cli_utils::YELLOW, cli_utils::RESET);

        let chronork_core::models::LogCategories {
            achievements,
            learnings,
            mistakes,
            ideas,
        } = &mut log.logs;

        let mut categories = vec![
            ("Achievement", achievements),
            ("Learning", learnings),
            ("Mistake", mistakes),
            ("Idea", ideas),
        ];

        'interactive: for (cat_name, cat_vec) in categories.iter_mut() {
            println!("\n{}{}{}:", cli_utils::BOLD, cat_name, cli_utils::RESET);
            let mut counter = 1;
            
            loop {
                print!("  {}. ", counter);
                io::stdout().flush().unwrap();
                
                let mut input = String::new();
                
                // Graceful Panic Handling for EOF (Ctrl+D)
                match io::stdin().read_line(&mut input) {
                    Ok(0) => {
                        println!("\n{}EOF detected. Exiting input sequence...{}", cli_utils::YELLOW, cli_utils::RESET);
                        break 'interactive;
                    }
                    Ok(_) => {} // Continue processing input
                    Err(e) => {
                        eprintln!("{}Input error: {}{}", cli_utils::RED, e, cli_utils::RESET);
                        break 'interactive;
                    }
                }
                
                let input = input.trim();
                
                if input.is_empty() || input == "null" {
                    break;
                }

                let prefix = cat_name.chars().next().unwrap().to_ascii_lowercase();
                let (content, tags) = parse_input(input);
                
                let entry = LogEntry {
                    id: core_utils::generate_id(&prefix.to_string()),
                    timestamp: core_utils::get_current_timestamp(),
                    content,
                    tags,
                };
                
                cat_vec.push(entry);
                counter += 1;
            }
        }
    } else {
        let mut i = 0;
        while i < args.len() {
            if i + 1 >= args.len() {
                break;
            }

            let flag = &args[i];
            let raw_content = &args[i + 1];
            let (content, tags) = parse_input(raw_content);
            
            let mut entry = LogEntry {
                id: String::new(),
                timestamp: core_utils::get_current_timestamp(),
                content,
                tags,
            };

            match flag.as_str() {
                "-a" => {
                    entry.id = core_utils::generate_id("a");
                    log.logs.achievements.push(entry);
                }
                "-l" => {
                    entry.id = core_utils::generate_id("l");
                    log.logs.learnings.push(entry);
                }
                "-m" => {
                    entry.id = core_utils::generate_id("m");
                    log.logs.mistakes.push(entry);
                }
                "-i" => {
                    entry.id = core_utils::generate_id("i");
                    log.logs.ideas.push(entry);
                }
                _ => {
                    i += 1;
                    continue;
                }
            }
            i += 2;
        }
        println!("{}✔ Quick-store complete.{}", cli_utils::GREEN, cli_utils::RESET);
    }

    match storage.save(&log) {
        Ok(_) => {
            if interactive {
                println!("\n{}✔ Data committed to {}{}", cli_utils::GREEN, date, cli_utils::RESET);
            }
        }
        Err(e) => eprintln!("{}Failed to save log: {}{}", cli_utils::RED, e, cli_utils::RESET),
    }
}