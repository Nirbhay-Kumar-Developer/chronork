#ifndef UTILS_HPP
#define UTILS_HPP

#include <iostream>
#include <string>
#include <chrono>
#include <ctime>
#include <iomanip>
#include <sstream>
#include <cstdint>

namespace Utils {

    // --- ANSI Color Codes for Termux ---
    const std::string RESET  = "\033[0m";
    const std::string BOLD   = "\033[1m";
    const std::string RED    = "\033[31m";
    const std::string GREEN  = "\033[32m";
    const std::string YELLOW = "\033[33m";
    const std::string BLUE   = "\033[34m";
    const std::string CYAN   = "\033[36m";

    // Returns a color based on the category string
    inline std::string getColor(const std::string& category) {
        if (category == "Mistake") return RED;
        if (category == "Achievement") return GREEN;
        if (category == "Learning") return YELLOW;
        if (category == "Idea") return CYAN;
        return RESET;
    }

    // --- Date, Time, and ID Helpers ---

    // Returns exact 64-bit UNIX epoch timestamp (seconds since 1970)
    inline int64_t getCurrentTimestamp() {
        auto now = std::chrono::system_clock::now();
        return std::chrono::duration_cast<std::chrono::seconds>(now.time_since_epoch()).count();
    }

    // Generates a simple, robust unique ID based on the timestamp (e.g., "a_1720524000")
    inline std::string generateId(const std::string& prefix) {
        return prefix + "_" + std::to_string(getCurrentTimestamp());
    }

    // Returns today's date strictly as "YYYY-MM-DD" for filesystem sorting
    inline std::string getTodayDate() {
        auto now = std::chrono::system_clock::now();
        std::time_t now_time = std::chrono::system_clock::to_time_t(now);
        std::tm now_tm;
        
        // Use localtime_r for thread-safe memory handling (POSIX standard)
        localtime_r(&now_time, &now_tm);
        
        std::ostringstream oss;
        oss << (now_tm.tm_year + 1900) << "-"
            << std::setfill('0') << std::setw(2) << (now_tm.tm_mon + 1) << "-"
            << std::setfill('0') << std::setw(2) << now_tm.tm_mday;
        return oss.str();
    }

    // Returns "YYYY-MM" for standard directory parsing
    inline std::string getCurrentMonth() {
        auto now = std::chrono::system_clock::now();
        std::time_t now_time = std::chrono::system_clock::to_time_t(now);
        std::tm now_tm;
        localtime_r(&now_time, &now_tm);
        
        std::ostringstream oss;
        oss << (now_tm.tm_year + 1900) << "-"
            << std::setfill('0') << std::setw(2) << (now_tm.tm_mon + 1);
        return oss.str();
    }
    
    inline std::string getCurrentYear() {
        auto now = std::chrono::system_clock::now();
        std::time_t now_time = std::chrono::system_clock::to_time_t(now);
        std::tm now_tm;
        // Use localtime_r for thread-safe memory handling (POSIX standard)
        localtime_r(&now_time, &now_tm);
        
        std::ostringstream oss;
        oss << (now_tm.tm_year + 1900);
        return oss.str();
    }

    // --- CLI Help Menu ---
        // --- CLI Help Menu ---
    inline void printUsage() {
        std::cout << BOLD << CYAN << "chronork v0.1.0" << RESET << "\n\n";
        std::cout << BOLD << "Usage:" << RESET << " chronork [command] <options>\n";
        std::cout << "       chronork [flags] (implicitly dumps logs)\n\n";
        
        std::cout << BOLD << "Commands:" << RESET << "\n";
        std::cout << "  store                Interactively store multiple logs per category\n";
        std::cout << "  dump [flags]         Explicitly query and filter stored logs\n\n";
        
        std::cout << BOLD << "Filtering Flags:" << RESET << "\n";
        std::cout << "  -f <categories...>   Filter by category name (achievements, learnings, mistakes, ideas)\n";
        std::cout << "  -t <tags...>         Filter logs containing any of these explicit #tags\n";
        std::cout << "  -d <YYYY.MM.DD>      Target a specific exact date\n";
        std::cout << "  -m <MM or YYYY-MM>   Target a specific month (defaults to current year if only MM given)\n";
        std::cout << "  -y <YYYY>            Target an entire specific year\n\n";
        
        std::cout << BOLD << "Examples:" << RESET << "\n";
        std::cout << "  chronork store                    # Open interactive prompt. Leave empty + Enter to skip.\n";
        std::cout << "  chronork -f mistakes              # Implicit dump: displays today's mistakes\n";
        std::cout << "  chronork -y 2026                  # Scans and displays the entire year of 2026\n";
        std::cout << "  chronork -m 07 -f mistakes        # Displays mistakes made in July of the current year\n";
        std::cout << "  chronork -d 2026.07.09            # Targets logs from that exact date string\n";
        std::cout << "  chronork -f mistakes ideas -t ai  # Stacks filters: show mistakes/ideas tagged with #ai\n";
        std::cout << std::string(80, '-') << std::endl;
    }
}

#endif // UTILS_HPP