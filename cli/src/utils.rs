// --- ANSI Color Codes for Termux/Linux Console ---
pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const CYAN: &str = "\x1b[36m";

// Returns a color based on the category string for human console layout
pub fn get_color(category: &str) -> &'static str {
    match category {
        "Mistake" => RED,
        "Achievement" => GREEN,
        "Learning" => YELLOW,
        "Idea" => CYAN,
        _ => RESET,
    }
}

// --- CLI Help Menu ---
pub fn print_usage() {
    println!("{}{}chronork v0.1.0{}\n", BOLD, CYAN, RESET);
    println!("{}Usage:{} chronork [command] <options>", BOLD, RESET);
    println!("       chronork [flags] (implicitly dumps logs)\n");
    
    println!("{}Commands:{}", BOLD, RESET);
    println!("  store                Interactively store multiple logs per category");
    println!("  dump [flags]         Explicitly query and filter stored logs\n");
    
    println!("{}Filtering Flags:{}", BOLD, RESET);
    println!("  -f <categories...>   Filter by category name (achievements, learnings, mistakes, ideas)");
    println!("  -t <tags...>         Filter logs containing any of these explicit #tags");
    println!("  -d <YYYY.MM.DD>      Target a specific exact date");
    println!("  -m <MM or YYYY-MM>   Target a specific month (defaults to current year if only MM given)");
    println!("  -y <YYYY>            Target an entire specific year\n");
    
    println!("{}Examples:{}", BOLD, RESET);
    println!("  chronork store                    # Open interactive prompt. Leave empty + Enter to skip.");
    println!("  chronork -f mistakes              # Implicit dump: displays today's mistakes");
    println!("  chronork -y 2026                  # Scans and displays the entire year of 2026");
    println!("  chronork -m 07 -f mistakes        # Displays mistakes made in July of the current year");
    println!("  chronork -d 2026.07.09            # Targets logs from that exact date string");
    println!("  chronork -f mistakes ideas -t ai  # Stacks filters: show mistakes/ideas tagged with #ai");
    println!("{}", "-".repeat(80));
}