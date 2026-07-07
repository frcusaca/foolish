#!/bin/bash -x

if [ "$1" == '-f' ]; then
		  rm foolish-ubca/snapshot_tests/approved/*.new
		  cargo clean
fi
cargo build
cargo insta test foolish-ubca
./foolish_review.sh foolish-ubca
echo "Press enter to approve."
read -x asdf
./accept_approved.sh foolish-ubca
