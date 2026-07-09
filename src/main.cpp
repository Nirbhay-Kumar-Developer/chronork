#include <iostream>
#include <string>
#include <vector>
#include "commands.hpp"
#include "utils.hpp"

int main(int argc, char* argv[]) {
    if (argc < 2) {
        Utils::printUsage();
        return 1;
    }

    std::string command;
    std::vector<std::string> args;

    // UX Feature: If the first argument is a flag (e.g., "-y" or "-f"), 
    // implicitly route it to the 'dump' command.
    if (argv[1][0] == '-') {
        command = "dump";
        for (int i = 1; i < argc; ++i) {
            args.push_back(argv[i]);
        }
    } else {
        command = argv[1];
        for (int i = 2; i < argc; ++i) {
            args.push_back(argv[i]);
        }
    }

    // Command Routing Logic
    if (command == "dump") {
        Commands::handleDump(args);
    } 
    else if (command == "store") {
        Commands::handleStore(args);
    } 
    else {
        std::cerr << Utils::RED << "Error: Unknown command '" << command << "'" << Utils::RESET << std::endl;
        Utils::printUsage();
        return 1;
    }

    return 0;
}