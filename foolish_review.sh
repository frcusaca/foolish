#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 [-f] <crate-path>"
    echo "Example: $0 foolish-ubca"
    exit 1
fi

full_review=0
if [[ "$1" == "-f" ]]; then
   echo "performing full review"
   full_review=1
	shift
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
readarray -d '' snapfiles < <(shuf -z -e *.snap)
shopt -u nullglob

echo "There are ${#files[@]} new snaps to review and ${#snapfiles[@]} approved snaps."
if [[ ${#files[@]} -eq 0 ]]; then
    exit 0
else
  if (( full_review )); then
    echo "Full review requetsed."
    reread=$((${#files[@]}/10))
      reread=$(( reread > 0 ? reread : 1))
      echo "Plan to reread $reread from $( ls *.snap | shuf)"
      for rereadfn in $( ls *.snap | shuf|head -n $reread); do
    	 echo "Debuginspecting $rereadfn"
    	 if [ $reread -gt 0 ]; then
    		echo "Debug inspecting will reread $rereadfn"
    		if [ ! -e ${rereadfn}.new ]; then
    		  # This branch does not trigger using inst and signage
    		  echo Requesting Review of $rereadfn for rereading.
    		  mv $rereadfn ${rereadfn}.new
    		  files+=(${rereadfn}.new)
    		  ((reread--))
    		else
    		  echo Requesting De Novo review of ${rereadfn} by removing approved snap.
    		  rm $rereadfn
    		  # Already part of files files+=(${rereadfn}.new)
    		fi
    	 else
    		 break
    	 fi
      done
    fi
fi

CNT=0
TTL=${#files[@]}

afiles=()
for x in "${files[@]}"; do
    CNT=$((CNT + 1))
    echo "Reviewing ${CNT}/${TTL}: $x"
    DC="$(diff -I '^\(Public key\|Foolish signature\|HFS signature\|Comments signature\|generated: 2\)' ${x%%.new} $x| wc -l || true)"
    echo "Reviewing ${CNT}/${TTL}: $DC differences for $x"
	 if [ ! -e ${x%%.new} -o $DC -gt 0 ]; then 
				sleep 2s
				vimdiff "${x%%.new}" "$x"
				if egrep -qi '@agents?, skip' "$x"; then
					echo "Unchanged output $x"
					rm "$x"
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
echo "Approved here: ${afiles[@]}"
echo "Approved all : $(ls *.snap.new.approved 2>/dev/null | wc -l)"
echo "Flagged:  $(ls *.snap.new 2>/dev/null | wc -l)"
egrep -Hi '[@]agent' *.new
echo "Sanity check to see if we've missed any @Agents comments in the approved files"
egrep -Hi gent *.approved 
