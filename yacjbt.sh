#!/bin/bash
./prep_java25_offline.sh
RPD="$PWD/claude_mvn/repo"
mvn -o -Dmaven.repo.local="$RPD" $@
