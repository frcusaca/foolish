#!/bin/bash -x

MINUS_F=""
if [ "$1" == '-f' ]; then
		  rm foolish-ubca/snapshot_tests/approved/*.new
		  cargo clean
        MINUS_F="-f"
fi
cargo build
cargo insta test
./foolish_review.sh ${MINUS_F} foolish-ubca
echo "Press enter to approve."
read -x asdf
./accept_approved.sh foolish-ubca
