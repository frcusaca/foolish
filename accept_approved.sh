#!/usr/bin/env bash
set -euo pipefail

# Resolve the signer relative to this script's own repository. verify_signatures
# is a bin in foolish-core; build it if missing.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SIGNER="$SCRIPT_DIR/target/debug/verify_signatures"
if [[ ! -x "$SIGNER" ]]; then
    echo "verify_signatures not found; building it in $SCRIPT_DIR ..."
    ( cd "$SCRIPT_DIR" && cargo build -p foolish-core --bin verify_signatures )
fi

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

# Collect all .snap.new.approved files
files=()
while IFS= read -r -d '' f; do
    files+=("$f")
done < <(find "$APPROVED_DIR" -name '*.snap.new.approved' -print0)

if [[ ${#files[@]} -eq 0 ]]; then
    echo "No .snap.new.approved files found."
    exit 0
fi

echo "Found ${#files[@]} .snap.new.approved files:"
printf '  %s\n' "${files[@]}"
echo ""

# Sign all at once (password typed once)
echo "Enter passphrase (will be used for all files):"
"$SIGNER" --stdin-passphrase --write-verified "${files[@]}"

echo ""
echo "Moving .snap.new.approved → .snap ..."
for f in "${files[@]}"; do
    target="${f%.new.approved}"
    mv "$f" "$target"
    echo "  $(basename "$target")"
done

echo ""
echo "Done. ${#files[@]} snapshots signed and moved."
