# chronork
chronork is a command-line utility designed to record, categorize, and query daily entries across distinct operational categories: achievements, learnings, mistakes, and ideas. Logs are stored locally as structured JSON files organized chronologically.
## Core Operations
The application operates using two primary commands: data entry (store) and log retrieval (dump). Passing filtering flags as the first argument implicitly routes the execution to log retrieval.
### Recording Entries (store)
Entries can be recorded either through a step-by-step interactive prompt or via direct inline flags.
#### Interactive Mode
Running the command without flags initializes a structured prompt for each category. Pressing Enter on an empty line skips the active category.
```bash
chronork store

```
#### Inline Storage
Use specific flags followed by the text string to record data directly without entering the interactive menu:
 * -a <text>: Append an entry to Achievements.
 * -l <text>: Append an entry to Learnings.
 * -m <text>: Append an entry to Mistakes.
 * -i <text>: Append an entry to Ideas.
```bash
chronork store -a "Completed network migration" -m "Misconfigured routing table #dns"

```
## Data Parsing and Metadata
 * **Hashtag Extraction**: The parser extracts inline hashtags (e.g., #network, #ai) from your entry text automatically, stripping trailing punctuation and registering them as searchable metadata tags. Text without tags is assigned to a general tag by default.
 * **Data Isolation**: Every saved log entry receives a unique identifier based on its category and execution timestamp. Storage access defaults to standard paths (/var/lib/chronork/logs/) or adjusts dynamically if a system PREFIX environment variable is defined.
## Querying and Filtering (dump)
Log retrieval allows multi-layered filtering by time ranges, specific categories, or context tags. If no command is specified but filtering flags are present, the tool executes a query for the current date.
### Filtering Parameters
| Flag | Parameter | Description |
|---|---|---|
| -f | <categories...> | Restricts output to specified types: achievements (a), learnings (l), mistakes (m), or ideas (i). |
| -t | <tags...> | Filters entries matching one or more explicit metadata tags. |
| -d | <YYYY.MM.DD> | Targets a specific calendar date. |
| -m | <MM> or <YYYY-MM> | Scans an entire month (defaults to current year if only MM is passed). |
| -y | <YYYY> | Scans an entire calendar year. |
## Usage Examples
### Immediate Queries
Display all recorded mistakes from the current date:
```bash
chronork -f mistakes

```
### Tag and Category Stacking
Display both mistakes and ideas from the current date that match the tag #ai:
```bash
chronork -f mistakes ideas -t ai

```
### Historic Time Ranges
Scan and output all logged entries for the entire year of 2026:
```bash
chronork -y 2026

```
Display only mistakes logged during July of the current calendar year:
```bash
chronork -m 07 -f mistakes

```
Target a specific historical date precisely:
```bash
chronork -d 2026.07.09

```
