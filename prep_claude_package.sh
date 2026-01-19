#!/bin/bash
RPD="$PWD/claude_mvn/repo"
mvn -Dmaven.repo.local="$RPD" -U -DskipTests package
mvn -Dmaven.repo.local="$RPD" -U -DskipTests test
mvn -Dmaven.repo.local="$RPD" -U -DskipTests dependency:go-offline
git add claude_mvn
git commit -m "Vendor Maven repo for offline builds."
mvn -o -Dmaven.repo.local="$RPD" test
