#!/bin/bash
# Simple test script for UBC REPL

echo "Testing UBC REPL with sample expressions..."
echo ""

# Test expressions
echo "{5;}" | mvn -q exec:java -Dexec.mainClass="org.foolish.ubc.UbcRepl"
echo ""

echo "{10 + 20;}" | mvn -q exec:java -Dexec.mainClass="org.foolish.ubc.UbcRepl"
echo ""

echo "{(5 + 3) * 2;}" | mvn -q exec:java -Dexec.mainClass="org.foolish.ubc.UbcRepl"
echo ""

echo "{1; 2; 3 + 4;}" | mvn -q exec:java -Dexec.mainClass="org.foolish.ubc.UbcRepl"
echo ""

echo "Test complete!"
