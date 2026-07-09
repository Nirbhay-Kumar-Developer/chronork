#ifndef MODELS_HPP
#define MODELS_HPP

#include <string>
#include <vector>
#include <cstdint>
#include <stdexcept>
#include <nlohmann/json.hpp>

namespace Models {

    // --- Data Structures ---

    struct LogEntry {
        std::string id;
        int64_t timestamp;
        std::string content;
        std::vector<std::string> tags;

        // Built-in integrity check to prevent writing corrupt states
        bool isValid() const {
            return !id.empty() && timestamp > 0 && !content.empty();
        }
    };

    struct Metadata {
        std::string date; // Format: YYYY-MM-DD
        int64_t updated_at;
    };

    struct LogCategories {
        std::vector<LogEntry> achievements;
        std::vector<LogEntry> mistakes;
        std::vector<LogEntry> learnings;
        std::vector<LogEntry> ideas;
    };

    struct DailyLog {
        Metadata metadata;
        LogCategories logs;
        
        // Helper to validate the entire daily log before serialization
        bool isValid() const {
            if (metadata.date.empty() || metadata.updated_at <= 0) return false;
            
            for (const auto& entry : logs.achievements) if (!entry.isValid()) return false;
            for (const auto& entry : logs.mistakes) if (!entry.isValid()) return false;
            for (const auto& entry : logs.learnings) if (!entry.isValid()) return false;
            for (const auto& entry : logs.ideas) if (!entry.isValid()) return false;
            
            return true;
        }
    };

    // --- JSON Serialization Macros ---
    // These macros automatically generate the to_json and from_json functions.
    // They must be defined in the same namespace as the structs they target.

    NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE(LogEntry, id, timestamp, content, tags)
    NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE(Metadata, date, updated_at)
    NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE(LogCategories, achievements, mistakes, learnings, ideas)
    NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE(DailyLog, metadata, logs)

} // namespace Models

#endif // MODELS_HPP