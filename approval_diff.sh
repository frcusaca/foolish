#!/bin/bash

# Script to open received and approved .foo files in vimdiff for full content review
# Usage: ./diff_foo_files.sh <test_name>

TEST_NAME="$1"
RECEIVED_FILE="${TEST_NAME}.received.foo"
APPROVED_FILE="${TEST_NAME}.approved.foo"

# Open both files in vimdiff with full content display
vim -d -c "set foldlevel=1000000" "$RECEIVED_FILE" "$APPROVED_FILE"
