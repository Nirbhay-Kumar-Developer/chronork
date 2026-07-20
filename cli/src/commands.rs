use std::io::{self, Write};

use chronork_core::models::{LogEntry, QueryFilter};
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

    for token in tokens {
        let hash_pos = token.find('#');
        let is_tag = match hash_pos {
            Some(0) => true,
            Some(1) => token.chars().next().map_or(false, |c| c.is_ascii_punctuation()),
            _ => false,
        };

        // PATCH: Removed hash_pos.unwrap() and fixed unsafe multi-byte byte slicing panic
        if is_tag {
            if let Some(pos) = hash_pos {
                let mut tag = String::new();
                // Safely skip past the '#' index character-by-character to preserve valid UTF-8 structures
                for c in token.chars().skip(pos + 1) {
                    tag.push(c);
                }
                while tag.ends_with(|c: char| c.is_ascii_punctuation()) {
                    tag.pop();
                }
                if !tag.is_empty() {
                    tags.push(tag);
                }
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

/// Standard Terminal Formatter (ANSI Colors, Space Padding)
fn print_category(date: &str, category_name: &str, entries: &[LogEntry]) {
    if entries.is_empty() {
        return;
    }
    let color = cli_utils::get_color(category_name);
    
    for entry in entries {
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

/// AI Context Formatter (Strict Markdown, Sanitized Strings)
fn print_markdown_category(category_name: &str, entries: &[LogEntry]) {
    if entries.is_empty() {
        return;
    }
    println!("### {}", category_name);
    
    for entry in entries {
        // Sanitize newlines to maintain clean bullet structures
        let sanitized = entry.content.replace('\n', " ").trim().to_string();
        
        let tag_str = if entry.tags.is_empty() {
            "none".to_string()
        } else {
            entry.tags.join(", ")
        };
        
        println!("- {} [Tags: {}]", sanitized, tag_str);
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
    let mut use_markdown = false;

    // State Machine Argument Parser
    for arg in args {
        match arg.as_str() {
            "-y" | "-m" | "-d" | "-f" | "-t" => {
                mode = arg;
                continue;
            }
            "--markdown" | "-md" => {
                use_markdown = true;
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
        
        // PATCH: Removed unwrap_or(1) parsing step, replaced with strict 01-12 value verification logic
        match sanitized_m.parse::<u8>() {
            Ok(val) if (1..=12).contains(&val) => {
                m = format!("{:02}", val);
            }
            _ => {
                eprintln!("{}Error: Invalid month value (must be between 01 and 12).{}", cli_utils::RED, cli_utils::RESET);
                return;
            }
        }
    }

    // Construct unified QueryFilter for the Core Engine
    let mut start_date = None;
    let mut end_date = None;
    let time_filter_applied = !y.is_empty() || !m.is_empty() || !d.is_empty();

    if !time_filter_applied {
        let today = core_utils::get_today_date();
        start_date = Some(today.clone());
        end_date = Some(today);
    } else if !d.is_empty() {
        if d.len() == 10 {
            start_date = Some(d.clone());
            end_date = Some(d);
        }
    } else {
        let target_year = if y.is_empty() { core_utils::get_current_year() } else { y.clone() };
        let start_m = if m.is_empty() { "01".to_string() } else { m.clone() };
        let end_m = if m.is_empty() { "12".to_string() } else { m.clone() };
        
        let year_num: i32 = target_year.parse().unwrap_or(2026);
        let month_num: u32 = end_m.parse().unwrap_or(12);

        // FIX: Replaced direct chrono calls with the core utility helper to avoid external unlinked dependency errors
        let last_day = core_utils::get_last_day_of_month(year_num, month_num);

        start_date = Some(format!("{}-{}-01", target_year, start_m));
        end_date = Some(format!("{}-{}-{:02}", target_year, end_m, last_day)); 
    }

    let filter = QueryFilter {
        start_date,
        end_date,
        categories: f,
        tags: t,
    };

    // Execute query via Core Indexing Engine
    match storage.scan_range(&filter) {
        Ok(logs) => {
            if logs.is_empty() {
                println!("{}No logs found matching your criteria.{}", cli_utils::YELLOW, cli_utils::RESET);
                return;
            }

            if use_markdown {
                println!("# Chronork Data Export for AI Analysis\n");
            }

            for log in logs {
                if use_markdown {
                    println!("## Date: {}", log.metadata.date);
                    print_markdown_category("Achievements", &log.logs.achievements);
                    print_markdown_category("Learnings", &log.logs.learnings);
                    print_markdown_category("Mistakes", &log.logs.mistakes);
                    print_markdown_category("Ideas", &log.logs.ideas);
                    println!("\n{}\n", "-".repeat(40));
                } else {
                    print_category(&log.metadata.date, "Achievement", &log.logs.achievements);
                    print_category(&log.metadata.date, "Learning", &log.logs.learnings);
                    print_category(&log.metadata.date, "Mistake", &log.logs.mistakes);
                    print_category(&log.metadata.date, "Idea", &log.logs.ideas);
                }
            }
        }
        Err(e) => eprintln!("{}Database Error: {}{}", cli_utils::RED, e, cli_utils::RESET),
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
                
                match io::stdin().read_line(&mut input) {
                    Ok(0) => {
                        println!("\n{}EOF detected. Exiting input sequence...{}", cli_utils::YELLOW, cli_utils::RESET);
                        break 'interactive;
                    }
                    Ok(_) => {} 
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