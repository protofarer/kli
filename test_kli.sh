#!/bin/bash

set -e # Exit immediately if a command exits with a non-zero status.

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Function to print colored output
print_color() {
	printf "${2}${1}${NC}\n"
}

# Function to run a command and check its exit status
run_command() {
	"$@"
	local status=$?
	if [ $status -ne 0 ]; then
		print_color "Error: Command '$*' failed with exit status $status" "$RED"
		exit 1
	fi
	return $status
}

# Setup
print_color "Setting up test environment..." "$GREEN"
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"
print_color "Working in temporary directory: $TEST_DIR" "$GREEN"

# Test config command
print_color "\nTesting 'config' command..." "$GREEN"
run_command kli config

# Test ergo command
print_color "\nTesting 'ergo' command..." "$GREEN"
echo -e "term\n\n\nn\n" | run_command kli ergo # Provide default answers to prompts

# Verify .kli file was created
if [ -f .kli ]; then
	print_color ".kli file created successfully" "$GREEN"
else
	print_color "Error: .kli file not created" "$RED"
	exit 1
fi

# Test ide command
print_color "\nTesting 'ide' command..." "$GREEN"
run_command kli ide

# Test tinker command
print_color "\nTesting 'tinker' command..." "$GREEN"
run_command kli tinker node
if [ -d ~/scratch/tinker/node ]; then
	print_color "Node tinker project created successfully" "$GREEN"
else
	print_color "Error: Node tinker project not created" "$RED"
	exit 1
fi

# Test repo commands
print_color "\nTesting 'repo new' command..." "$GREEN"
run_command kli repo new -n test-repo
if [ -d test-repo ]; then
	print_color "test-repo created successfully" "$GREEN"
else
	print_color "Error: test-repo not created" "$RED"
	exit 1
fi

print_color "\nTesting 'repo del' command..." "$GREEN"
run_command kli repo del test-repo

# Test help command
print_color "\nTesting 'help' command..." "$GREEN"
run_command kli help

# Test version command
print_color "\nTesting 'version' command..." "$GREEN"
run_command kli --version

# Teardown
print_color "\nTearing down test environment..." "$GREEN"
cd ~
rm -rf "$TEST_DIR"
rm -rf ~/scratch/tinker/node

print_color "\nAll tests completed successfully!" "$GREEN"
