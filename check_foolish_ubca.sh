#!/bin/bash -x

MINUS_F=""
if [ "$1" == '-f' ]; then
		  rm foolish-ubca/snapshot_tests/approved/*.new
		  #cargo clean
		  # Run this loop in your terminal to target every local package
		  for pkg in $(cargo metadata --format-version 1 | jq -r '.workspace_members[] | split(" ")[0]'); do
				cargo clean -p $pkg
		  done
        MINUS_F="-f"
fi
cargo build
cargo insta test
./foolish_review.sh ${MINUS_F} foolish-ubca
echo "Press enter to approve."
read -x asdf
./accept_approved.sh foolish-ubca
