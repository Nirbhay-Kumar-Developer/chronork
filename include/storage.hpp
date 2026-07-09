#ifndef STORAGE_HPP
#define STORAGE_HPP

#include <string>
#include <filesystem>
#include <fstream>
#include <stdexcept>
#include <cstdlib>
#include <iostream>
#include "models.hpp"

namespace fs = std::filesystem;

class StorageManager {
private:
    // Resolves the root directory natively in Termux
    static std::string getBasePath() {
        const char* prefix = std::getenv("PREFIX");
        return prefix ? std::string(prefix) + "/var/lib/chronork/logs/" 
                      : "/var/lib/chronork/logs/";
    }

    // Constructs the exact file path based on the YYYY-MM-DD date string
    static fs::path resolveFilePath(const std::string& dateStr) {
        if (dateStr.length() < 10) {
            throw std::invalid_argument("Date string must be in YYYY-MM-DD format.");
        }

        std::string year = dateStr.substr(0, 4);
        std::string month = dateStr.substr(5, 2);

        fs::path fullPath = fs::path(getBasePath()) / year / month / (dateStr + ".json");
        return fullPath;
    }

    // Applies strict 600 permissions (Owner Read/Write only)
    static void secureFilePermissions(const fs::path& target) {
        fs::permissions(target, 
            fs::perms::owner_read | fs::perms::owner_write, 
            fs::perm_options::replace);
    }

public:
    /**
     * Loads a daily log from disk. Returns a new empty log if it doesn't exist.
     */
    static Models::DailyLog load(const std::string& dateStr) {
        fs::path targetPath = resolveFilePath(dateStr);

        if (!fs::exists(targetPath)) {
            Models::DailyLog newLog;
            newLog.metadata.date = dateStr;
            newLog.metadata.updated_at = 0;
            return newLog;
        }

        std::ifstream file(targetPath);
        if (!file.is_open()) {
            throw std::runtime_error("Failed to open file for reading: " + targetPath.string());
        }

        nlohmann::json j;
        try {
            file >> j;
        } catch (const nlohmann::json::parse_error& e) {
            throw std::runtime_error("JSON Parse Error in " + targetPath.string() + ": " + e.what());
        }

        return j.get<Models::DailyLog>();
    }

    /**
     * Safely serializes and atomically writes a daily log to disk.
     */
    static void save(const Models::DailyLog& log) {
        // 1. Data Integrity Check before touching the filesystem
        if (!log.isValid()) {
            throw std::invalid_argument("DailyLog validation failed. Aborting write to prevent data corruption.");
        }

        fs::path targetPath = resolveFilePath(log.metadata.date);
        fs::path tmpPath = targetPath.string() + ".tmp";

        // 2. Ensure parent directories exist
        fs::create_directories(targetPath.parent_path());

        // 3. Write to a temporary file
        std::ofstream tmpFile(tmpPath, std::ios::out | std::ios::trunc);
        if (!tmpFile.is_open()) {
            throw std::runtime_error("Failed to create temporary write file: " + tmpPath.string());
        }

        nlohmann::json j = log;
        tmpFile << j.dump(4); // 4 spaces for readability 
        
        // 4. Force OS to flush buffer to physical disk
        tmpFile.flush();
        tmpFile.close();

        // 5. Secure the temp file before making it live
        secureFilePermissions(tmpPath);

        // 6. Atomic swap: Overwrite the old file seamlessly
        // If power fails exactly here, the old file remains perfectly intact.
        fs::rename(tmpPath, targetPath);
    }
};

#endif // STORAGE_HPP