#!/usr/bin/env bash
set -euo pipefail

SIGNER="/home/hcbusy/tmp/foolish-worktrees/foop-62-ubca-mimo/foolish/target/debug/verify_signatures"

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <crate-path>"
    echo "Example: $0 foolish-ubca"
    exit 1
fi

CRATE_DIR="$1"
if [ -z "$CRATE_DIR" ]; then
	echo "Must specify the crate directory as parameter to $0"
	exit -1
fi

APPROVED_DIR=$(find "$CRATE_DIR" -path '*/snapshot_tests/approved' -type d | head -1)

if [[ -z "$APPROVED_DIR" ]]; then
    echo "No snapshot_tests/approved directory found under $CRATE_DIR"
    exit 1
fi

# Collect all .snap.approved files
files=()
while IFS= read -r -d '' f; do
    files+=("$f")
done < <(find "$APPROVED_DIR" -name '*.snap.approved' -print0)

if [[ ${#files[@]} -eq 0 ]]; then
    echo "No .snap.approved files found."
    exit 0
fi

echo "Found ${#files[@]} .snap.approved files:"
printf '  %s\n' "${files[@]}"
echo ""

# Sign all at once (password typed once)
echo "Enter passphrase (will be used for all files):"
"$SIGNER" --stdin-passphrase --write-verified "${files[@]}"

echo ""
echo "Moving .snap.approved → .snap ..."
for f in "${files[@]}"; do
    target="${f%.approved}"
    mv "$f" "$target"
    echo "  $(basename "$target")"
done

echo ""
echo "Done. ${#files[@]} snapshots signed and moved."
