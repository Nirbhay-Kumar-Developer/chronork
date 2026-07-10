use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::models::{LogEntry};
use crate::storage::StorageManager;
use crate::utils;

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
        
        // It's a tag if it starts with '#' OR starts with punctuation (like ") then '#'
        let is_tag = match hash_pos {
            Some(0) => true,
            Some(1) => token.chars().next().map_or(false, |c| c.is_ascii_punctuation()),
            _ => false,
        };

        if is_tag {
            let mut tag = token[hash_pos.unwrap() + 1..].to_string();
            
            // Strip trailing punctuation (like closing quotes or commas)
            while tag.ends_with(|c: char| c.is_ascii_punctuation()) {
                tag.pop();
            }
            
            if !tag.is_empty() {
                tags.push(tag);
            }
        } else {
            // Not a tag, add it back to the content string
            content_parts.push(token);
        }
    }

    // Fallback to general if no valid tags were found
    if tags.is_empty() {
        tags.push("general".to_string());
    }

    (content_parts.join(" "), tags)
}

// Helper function to print a specific category, filtered by requested tags
fn print_category(
    date: &str,
    category_name: &str,
    entries: &[LogEntry],
    target_tags: &[String],
) {
    let color = utils::get_color(category_name);
    
    for entry in entries {
        // Tag filtering logic (OR match: if it has ANY of the target tags)
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
            continue; // Skip if no tags matched
        }

        print!("[{}] {}{:<12}{}: {}", date, color, category_name, utils::RESET, entry.content);
        
        print!("{} [", utils::BLUE);
        for (i, tag) in entry.tags.iter().enumerate() {
            print!("#{}", tag);
            if i < entry.tags.len() - 1 {
                print!(" ");
            }
        }
        println!("]{}", utils::RESET);
    }
}

// Helper to recursively collect JSON files manually without extra dependencies
fn collect_json_files(dir: &Path, dates: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_json_files(&path, dates);
            } else if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    dates.push(stem.to_string());
                }
            }
        }
    }
}

// --- DUMP COMMAND ---
pub fn handle_dump(args: &[String]) {
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
            "-d" => {
                d = arg.replace('.', "-"); // Convert 2026.07.09 to 2026-07-09
            }
            "-f" => f.push(arg.clone()),
            "-t" => t.push(arg.clone()),
            _ => {}
        }
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

    let mut dates_to_process = Vec::new();
    let time_filter_applied = !y.is_empty() || !m.is_empty() || !d.is_empty();

    if !time_filter_applied {
        // Default to today if no time flags are passed
        dates_to_process.push(utils::get_today_date());
    } else if !d.is_empty() {
        // Specific exact date requested
        dates_to_process.push(d);
    } else {
        // Range requested via year and/or month
        let prefix = env::var("PREFIX").unwrap_or_else(|_| "".to_string());
        let base_path = if prefix.is_empty() {
            "/var/lib/chronork/logs/".to_string()
        } else {
            format!("{}/var/lib/chronork/logs/", prefix)
        };
        
        let mut year = if y.is_empty() { utils::get_current_year() } else { y };
        let target_dir: PathBuf;

        if !m.is_empty() {
            let mut month = m;
            if month.len() >= 7 { // Handle "2026-07"
                year = month[0..4].to_string();
                month = month[5..7].to_string();
            } else if month.len() == 1 { // Handle "7" -> "07"
                month = format!("0{}", month);
            }
            target_dir = Path::new(&base_path).join(&year).join(&month);
        } else {
            target_dir = Path::new(&base_path).join(&year); // Traverse entire year
        }

        // Recursively scan the directory for logs
        if target_dir.exists() && target_dir.is_dir() {
            collect_json_files(&target_dir, &mut dates_to_process);
            dates_to_process.sort(); // Chronological order
        } else {
            println!("{}No logs found for the specified range.{}", utils::YELLOW, utils::RESET);
            return;
        }
    }

    // Execute Output
    for date in dates_to_process {
        match StorageManager::load(&date) {
            Ok(log) => {
                if log.metadata.updated_at == 0 {
                    continue;
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
                eprintln!("{}Error reading log for {}: {}{}", utils::RED, date, e, utils::RESET);
            }
        }
    }
}

// --- STORE COMMAND ---
pub fn handle_store(args: &[String]) {
    let date = utils::get_today_date();
    
    let mut log = match StorageManager::load(&date) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("{}Error loading log: {}{}", utils::RED, e, utils::RESET);
            return;
        }
    };

    log.metadata.updated_at = utils::get_current_timestamp();

    let mut interactive = true;
    for arg in args {
        if matches!(arg.as_str(), "-a" | "-l" | "-m" | "-i") {
            interactive = false;
            break;
        }
    }

    if interactive {
        println!("{}--- Recording Worklog for {} ---{}", utils::CYAN, date, utils::RESET);
        println!("{}Tip: Leave empty and press Enter to move to the next category.{}", utils::YELLOW, utils::RESET);

        // Destructure log.logs safely to create an array of mutable references
        let crate::models::LogCategories {
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

        for (cat_name, cat_vec) in categories.iter_mut() {
            println!("\n{}{}{}:", utils::BOLD, cat_name, utils::RESET);
            let mut counter = 1;
            
            loop {
                print!("  {}. ", counter);
                io::stdout().flush().unwrap(); // Ensure prompt is printed before input
                
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let input = input.trim();
                
                // Break the loop if the user inputs nothing
                if input.is_empty() || input == "null" {
                    break;
                }

                let prefix = cat_name.chars().next().unwrap().to_ascii_lowercase();
                
                let (content, tags) = parse_input(input);
                
                let entry = LogEntry {
                    id: utils::generate_id(&prefix.to_string()),
                    timestamp: utils::get_current_timestamp(),
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
                id: String::new(), // Assigned below
                timestamp: utils::get_current_timestamp(),
                content,
                tags,
            };

            match flag.as_str() {
                "-a" => {
                    entry.id = utils::generate_id("a");
                    log.logs.achievements.push(entry);
                }
                "-l" => {
                    entry.id = utils::generate_id("l");
                    log.logs.learnings.push(entry);
                }
                "-m" => {
                    entry.id = utils::generate_id("m");
                    log.logs.mistakes.push(entry);
                }
                "-i" => {
                    entry.id = utils::generate_id("i");
                    log.logs.ideas.push(entry);
                }
                _ => {
                    i += 1;
                    continue;
                }
            }
            i += 2;
        }
        println!("{}✔ Quick-store complete.{}", utils::GREEN, utils::RESET);
    }

    if let Err(e) = StorageManager::save(&log) {
        eprintln!("{}Failed to save log: {}{}", utils::RED, e, utils::RESET);
    } else if interactive {
        println!("\n{}✔ Data committed to {}{}", utils::GREEN, date, utils::RESET);
    }
}