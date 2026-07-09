#ifndef COMMANDS_HPP
#define COMMANDS_HPP

#include <iostream>
#include <vector>
#include <string>
#include <iomanip>
#include <sstream>
#include <filesystem>
#include <cctype>
#include "storage.hpp"
#include "models.hpp"
#include "utils.hpp"

namespace fs = std::filesystem;

namespace Commands {
    
    inline void parseInput(const std::string& rawInput, std::string& content, std::vector<std::string>& tags) {
        content.clear();
        tags.clear();
        
        std::vector<std::string> tokens;
        std::string currentToken;
        bool inQuotes = false;
        
        // 1. Tokenize the string, respecting spaces inside quotes
        for (char c : rawInput) {
            if (c == '"') {
                inQuotes = !inQuotes;
                currentToken += c;
            } else if (std::isspace(c) && !inQuotes) {
                // Only split on space if we are NOT inside quotes
                if (!currentToken.empty()) {
                    tokens.push_back(currentToken);
                    currentToken.clear();
                }
            } else {
                currentToken += c;
            }
        }
        if (!currentToken.empty()) {
            tokens.push_back(currentToken);
        }

        // 2. Process the tokens to separate tags from content
        for (const auto& token : tokens) {
            size_t hashPos = token.find('#');
            
            // It's a tag if it starts with '#' OR starts with punctuation (like ") then '#'
            if (hashPos != std::string::npos && (hashPos == 0 || (hashPos == 1 && std::ispunct(token[0])))) {
                std::string tag = token.substr(hashPos + 1);
                
                // Strip trailing punctuation (like closing quotes or commas)
                while (!tag.empty() && std::ispunct(tag.back())) {
                    tag.pop_back();
                }
                
                if (!tag.empty()) {
                    tags.push_back(tag);
                }
            } else {
                // Not a tag, add it back to the content string
                if (!content.empty()) content += " ";
                content += token; 
            }
        }
        
        // Fallback to general if no valid tags were found
        if (tags.empty()) {
            tags.push_back("general");
        }
    }


    // Helper function to print a specific category of logs
        // Helper function to print a specific category, filtered by requested tags
    inline void printCategory(const std::string& date, const std::string& categoryName, 
                              const std::vector<Models::LogEntry>& entries, 
                              const std::vector<std::string>& targetTags) {
        std::string color = Utils::getColor(categoryName);
        for (const auto& entry : entries) {
            // Tag filtering logic (OR match: if it has ANY of the target tags)
            bool match = targetTags.empty();
            if (!match) {
                for (const auto& target : targetTags) {
                    for (const auto& tag : entry.tags) {
                        if (target == tag) {
                            match = true;
                            break;
                        }
                    }
                    if (match) break;
                }
            }
            
            if (!match) continue; // Skip if no tags matched

            std::cout << "[" << date << "] " << color << std::left << std::setw(12) 
                      << categoryName << Utils::RESET << ": " << entry.content;
            
            std::cout << Utils::BLUE << " [";
            for (size_t i = 0; i < entry.tags.size(); ++i) {
                std::cout << "#" << entry.tags[i] << (i < entry.tags.size() - 1 ? " " : "");
            }
            std::cout << "]" << Utils::RESET << std::endl;
        }
    }

    // --- DUMP COMMAND ---
    inline void handleDump(const std::vector<std::string>& args) {
        std::string y, m, d;
        std::vector<std::string> f, t;
        std::string mode = "";

        // State Machine Argument Parser
        for (const auto& arg : args) {
            if (arg == "-y" || arg == "-m" || arg == "-d" || arg == "-f" || arg == "-t") {
                mode = arg;
                continue;
            }
            if (mode == "-y") y = arg;
            else if (mode == "-m") m = arg;
            else if (mode == "-d") {
                d = arg;
                std::replace(d.begin(), d.end(), '.', '-'); // Convert 2026.07.09 to 2026-07-09
            }
            else if (mode == "-f") f.push_back(arg);
            else if (mode == "-t") t.push_back(arg);
        }

        // Determine which categories to show
        bool showA = false, showL = false, showM = false, showI = false;
        if (f.empty()) {
            showA = showL = showM = showI = true;
        } else {
            for (std::string& cat : f) {
                char c = std::tolower(cat[0]);
                if (c == 'a') showA = true;
                else if (c == 'l') showL = true;
                else if (c == 'm') showM = true;
                else if (c == 'i') showI = true;
            }
        }

        std::vector<std::string> datesToProcess;
        bool timeFilterApplied = !y.empty() || !m.empty() || !d.empty();

        if (!timeFilterApplied) {
            // Default to today if no time flags are passed
            datesToProcess.push_back(Utils::getTodayDate());
        } else if (!d.empty()) {
            // Specific exact date requested
            datesToProcess.push_back(d);
        } else {
            // Range requested via year and/or month
            const char* prefix = std::getenv("PREFIX");
            std::string basePath = prefix ? std::string(prefix) + "/var/lib/chronork/logs/" : "/var/lib/chronork/logs/";
            std::string year = y.empty() ? Utils::getCurrentYear() : y;
            fs::path targetDir;

            if (!m.empty()) {
                std::string month = m;
                if (month.length() >= 7) { // Handle "2026-07"
                    year = month.substr(0, 4);
                    month = month.substr(5, 2);
                } else if (month.length() == 1) { // Handle "7" -> "07"
                    month = "0" + month;
                }
                targetDir = fs::path(basePath) / year / month;
            } else {
                targetDir = fs::path(basePath) / year; // Traverse entire year
            }

            // Recursively scan the directory for logs
            if (fs::exists(targetDir) && fs::is_directory(targetDir)) {
                for (const auto& entry : fs::recursive_directory_iterator(targetDir)) {
                    if (entry.is_regular_file() && entry.path().extension() == ".json") {
                        datesToProcess.push_back(entry.path().stem().string());
                    }
                }
                std::sort(datesToProcess.begin(), datesToProcess.end()); // Chronological order
            } else {
                std::cout << Utils::YELLOW << "No logs found for the specified range." << Utils::RESET << std::endl;
                return;
            }
        }

        // Execute Output
        for (const auto& date : datesToProcess) {
            try {
                Models::DailyLog log = StorageManager::load(date);
                if (log.metadata.updated_at == 0) continue; 

                if (showA && !log.logs.achievements.empty()) printCategory(date, "Achievement", log.logs.achievements, t);
                if (showL && !log.logs.learnings.empty()) printCategory(date, "Learning", log.logs.learnings, t);
                if (showM && !log.logs.mistakes.empty()) printCategory(date, "Mistake", log.logs.mistakes, t);
                if (showI && !log.logs.ideas.empty()) printCategory(date, "Idea", log.logs.ideas, t);
            } catch (const std::exception& e) {
                std::cerr << Utils::RED << "Error reading log for " << date << ": " << e.what() << Utils::RESET << std::endl;
            }
        }
    }

    // --- STORE COMMAND ---
    inline void handleStore(const std::vector<std::string>& args) {
        std::string date = Utils::getTodayDate();
        Models::DailyLog log;
        
        try {
            log = StorageManager::load(date);
        } catch (const std::exception& e) {
            std::cerr << Utils::RED << "Error loading log: " << e.what() << Utils::RESET << std::endl;
            return;
        }

        log.metadata.updated_at = Utils::getCurrentTimestamp();

        bool interactive = true;
        for (const auto& arg : args) {
            if (arg == "-a" || arg == "-l" || arg == "-m" || arg == "-i") {
                interactive = false;
                break;
            }
        }

        if (interactive) {
            std::vector<std::pair<std::string, std::vector<Models::LogEntry>*>> categories = {
                {"Achievement", &log.logs.achievements},
                {"Learning", &log.logs.learnings},
                {"Mistake", &log.logs.mistakes},
                {"Idea", &log.logs.ideas}
            };

            std::cout << Utils::CYAN << "--- Recording Worklog for " << date << " ---" << Utils::RESET << std::endl;
            std::cout << Utils::YELLOW << "Tip: Leave empty and press Enter to move to the next category." << Utils::RESET << std::endl;

            for (auto& cat : categories) {
                std::cout << "\n" << Utils::BOLD << cat.first << "s:" << Utils::RESET << std::endl;
                int counter = 1;
                
                while (true) {
                    std::string input;
                    std::cout << "  " << counter << ". ";
                    std::getline(std::cin, input);
                    
                    // Break the loop if the user inputs nothing
                    if (input.empty() || input == "null") {
                        break;
                    }

                    Models::LogEntry entry;
                    char prefix = static_cast<char>(tolower(cat.first[0]));
                    entry.id = Utils::generateId(std::string(1, prefix));
                    entry.timestamp = Utils::getCurrentTimestamp();
                    
                    // Parse content and extract #tags
                    parseInput(input, entry.content, entry.tags);
                    
                    cat.second->push_back(entry);
                    counter++;
                }
            }
        } else {
            for (size_t i = 0; i < args.size(); i++) {
                if (i + 1 >= args.size()) break; 
                
                Models::LogEntry entry;
                entry.timestamp = Utils::getCurrentTimestamp();
                
                // Process the CLI string for tags as well
                parseInput(args[i + 1], entry.content, entry.tags);

                if (args[i] == "-a") {
                    entry.id = Utils::generateId("a");
                    log.logs.achievements.push_back(entry);
                } else if (args[i] == "-l") {
                    entry.id = Utils::generateId("l");
                    log.logs.learnings.push_back(entry);
                } else if (args[i] == "-m") {
                    entry.id = Utils::generateId("m");
                    log.logs.mistakes.push_back(entry);
                } else if (args[i] == "-i") {
                    entry.id = Utils::generateId("i");
                    log.logs.ideas.push_back(entry);
                } else {
                    continue; 
                }
                i++; 
            }
            std::cout << Utils::GREEN << "✔ Quick-store complete." << Utils::RESET << std::endl;
        }

        try {
            StorageManager::save(log);
            if (interactive) std::cout << "\n" << Utils::GREEN << "✔ Data committed to " << date << Utils::RESET << std::endl;
        } catch (const std::exception& e) {
            std::cerr << Utils::RED << "Failed to save log: " << e.what() << Utils::RESET << std::endl;
        }
    }
}

#endif // COMMANDS_HPP