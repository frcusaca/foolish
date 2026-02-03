#!/bin/bash

# Script to extract approval test failure information from Maven build output
# Usage: ./extract_approval_failures.sh [<build_output_file>]
# If no file specified, reads from stdin

if [ $# -gt 1 ]; then
    echo "Usage: $0 [<build_output_file>]"
    echo "If no file specified, reads from stdin"
    exit 1
fi

# If no arguments provided, read from stdin
if [ $# -eq 0 ]; then
    INPUT_SOURCE="/dev/stdin"
else
    INPUT_SOURCE="$1"
    if [ ! -f "$INPUT_SOURCE" ]; then
        echo "Error: Build output file not found: $INPUT_SOURCE"
        exit 1
    fi
fi

cat $INPUT_SOURCE | grep -A2 "Failed Approval" |egrep -v "Failed Approval|--"|cut -d: -f2|sed 's/\(.*\)\..*\..*/\1/'|sort -u
