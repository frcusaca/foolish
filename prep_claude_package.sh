#!/bin/bash
RPD="$PWD/claude_mvn/repo"
mvn -Dmaven.repo.local="$RPD" -U -DskipTests clean
mvn -Dmaven.repo.local="$RPD" -U -DskipTests package
mvn -Dmaven.repo.local="$RPD" -U test
mvn -Dmaven.repo.local="$RPD" -U dependency:go-offline
git add claude_mvn
git commit -m "Vendor Maven repo for offline builds."
./yacjbt.sh test
