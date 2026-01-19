#!/bin/bash
RPD="$PWD/claude_mvn/repo"
mvn -o -Dmaven.repo.local="$RPD" $@
