#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <crate-path>"
    echo "Example: $0 foolish-ubca"
    exit 1
fi

CRATE_DIR="$1"
APPROVED_DIR=$(find "$CRATE_DIR" -path '*/snapshot_tests/approved' -type d | head -1)

if [[ -z "$APPROVED_DIR" ]]; then
    echo "No snapshot_tests/approved directory found under $CRATE_DIR"
    exit 1
fi

cd "$APPROVED_DIR"

shopt -s nullglob
#files=(*.snap.new)
readarray -d '' files < <(shuf -z -e *.snap.new)
shopt -u nullglob

if [[ ${#files[@]} -eq 0 ]]; then
    echo "No .snap.new files to review."
    exit 0
else
  reread=$((${#files[@]}/10))
  reread=$(( reread > 0 ? reread : 1))
  for rereadfn in $( ls *.snap | shuf|head -n $reread); do
	 if [ $reread -gt 0 ]; then
      if [ ! -e ${rereadfn}.new]; then
        echo marking $rereadfn for rereading
        cp $rereadfn ${rereadfn}.new
        files+=(${rereadfn}.new)
        ((reread--))
      fi
    else
       break
    fi
  done
fi

CNT=0
TTL=${#files[@]}

afiles=()
for x in "${files[@]}"; do
    CNT=$((CNT + 1))
    echo "Reviewing ${CNT}/${TTL}: $x"
    DC="$(diff -I '^\(Public key\|Foolish signature\|HFS signature\|Comments signature\)' ${x%%.new} $x| wc -l || true)"
    echo "Reviewing ${CNT}/${TTL}: $DC differences for $x"
	 if [ ! -e ${x%%.new} -o $DC -gt 0 ]; then 
				sleep 2s
				vimdiff "${x%%.new}" "$x"
				if grep -qi '@agent, skip' "$x"; then
					echo "Skipping $x"
				elif grep -qi '@agent' "$x"; then
					 echo "  → agent notified about $x"
					 (echo -n "$(date) cat $x"; cat "$x") >> "${x}.check"
                rm "$x"
				else
					 echo "  → approved $x"
				    af="${x}.approved"
					 mv "$x"  $af
                afiles+=("$af")
				fi
	fi
done

echo ""
echo "Done. Reviewed ${TTL} files."
echo "Approved here: $afiles"
echo "Approved all : $(ls *.snap.new.approved 2>/dev/null | wc -l)"
echo "Flagged:  $(ls *.snap.new 2>/dev/null | wc -l)"
egrep -Hi '[@]agent' *.new
echo "Sanity check to see if we've missed any @Agents comments in the approved files"
egrep -Hi gent *.approved 
