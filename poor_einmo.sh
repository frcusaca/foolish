#!/usr/bin/env bash
set -euo pipefail

# poor_einmo.sh — the poor foolisher's einmo review loop (FOOP-64).
#
# For every test in an einmo suite, opens a 4-way vimdiff:
#
#     input | output | checked | verified
#
# The source, what the FVM just produced, what was last reviewed, and what a
# human last signed — side by side, one test at a time. This is the stopgap
# until `einmo console-review` (FOOP-92 Phase 12, absorbed into FOOP-64) exists.
#
# ── HOW THE SCRIPT READS YOUR INTENT ──────────────────────────────────────
#
# You review by EDITING the checked/verified panes. On :qa the script diffs each
# pane against what it wrote, and accumulates one of three outcomes per test:
#
#   1. You typed a PROMOTE word into the checked or verified pane
#        promote · promoted · approve · approved · lgtm · sgtm
#      → PROMOTE. In the checked pane it means "promote the previous stage into
#        checked" (output->checked); in the verified pane, checked->verified.
#        Accumulated into the promote lists.
#
#   2. You changed a pane any other way (a note, a question, an @agent comment)
#      → INSTRUCTIONS FOR THE AGENT. The test is accumulated into
#        `send_to_agent_list`, printed at the end with your text preserved in
#        $REVIEW_DIR so an agent can act on it.
#
#   3. You changed nothing, or typed a SKIP word (skip · pass · idk)
#      → NO ACTION. Accumulated into `noop_list`, reported on one line. A skip
#        word is a deliberate non-decision: you looked and chose not to rule.
#
#   4. You typed `stop` into ANY pane
#      → END THE REVIEW. The current file is left completely untouched — not
#        promoted, not recorded, not counted — and the loop ends immediately,
#        printing the promotions and notes the EARLIER files already earned.
#        Checked before everything else, so it always wins.
#
# The script itself NEVER promotes, flags, or signs — it prints the exact
# commands for the promotions you asked for, and you (or an agent) run them.
# Promotion stays a deliberate, attributable act.
#
# The file scan and body extraction are einmo's own (`einmo list`, `einmo body`),
# not re-implemented here: einmo verifies every stamp before showing a byte
# (verify-on-inspect), and its body view excludes STAMPS/metadata — so what you
# review is exactly what `einmo compare` matches on. Timestamp/key churn never
# reaches your eyes.
#
# ── USAGE ─────────────────────────────────────────────────────────────────
#
#     ./poor_einmo.sh [-d] [-f] [-s] [-n] [-e EINMO] <suite-dir> [name-filter]
#
#     -d   differing only — skip tests whose output/checked/verified agree
#     -f   full review — do not prompt between tests
#     -s   shuffle the order (fresh eyes; mirrors foolish_review.sh)
#     -n   dry run — echo the vimdiff call instead of running it (debugging;
#          vimdiff locks the terminal, so this is how to trace the loop)
#     -e   path to the einmo binary (default: ./target/debug/einmo, then $PATH)
#
#     <suite-dir>     e.g. foolish-ubca/einmo_suite
#     [name-filter]   substring of the test's mirror path (e.g. foop/23)
#
# Examples:
#     ./poor_einmo.sh foolish-ubca/einmo_suite
#     ./poor_einmo.sh -d foolish-ubca/einmo_suite            # only what changed
#     ./poor_einmo.sh -s foolish-ubca/einmo_suite foop/23    # shuffle, one FOOP
#     ./poor_einmo.sh -n foolish-ubca/einmo_suite            # trace, no editor
#
# Every verb is ONE word alone in a pane (whitespace stripped, case ignored);
# anything else is a message for an agent:
#     promote · promoted · approve · approved · lgtm · sgtm   → promote
#     skip · pass · idk                                       → no action
#     stop                                                    → end the review
#
# In vimdiff: ]c / [c next/prev change · :qa finish this test · :cq abort.

differing_only=0
full_review=0
shuffle=0
dry_run=0
EINMO=""

while getopts "dfsne:h" opt; do
    case "$opt" in
        d) differing_only=1 ;;
        f) full_review=1 ;;
        s) shuffle=1 ;;
        n) dry_run=1 ;;
        e) EINMO="$OPTARG" ;;
        h) sed -n '4,62p' "$0"; exit 0 ;;
        *) echo "Try: $0 -h" >&2; exit 2 ;;
    esac
done
shift $((OPTIND - 1))

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 [-d] [-f] [-s] [-n] [-e EINMO] <suite-dir> [name-filter]" >&2
    echo "Example: $0 foolish-ubca/einmo_suite" >&2
    exit 1
fi

SUITE="${1%/}"
FILTER="${2:-}"

# --- locate einmo ---------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ -z "$EINMO" ]]; then
    if [[ -x "$SCRIPT_DIR/target/debug/einmo" ]]; then
        EINMO="$SCRIPT_DIR/target/debug/einmo"
    elif command -v einmo >/dev/null 2>&1; then
        EINMO="einmo"
    else
        echo "einmo binary not found; building it in $SCRIPT_DIR ..." >&2
        ( cd "$SCRIPT_DIR" && cargo build -p einmo --bins )
        EINMO="$SCRIPT_DIR/target/debug/einmo"
    fi
fi

for d in input output checked verified; do
    [[ -d "$SUITE/$d" ]] || { echo "Not an einmo suite (no $d/): $SUITE" >&2; exit 1; }
done

# --- ask einmo for the tests ----------------------------------------------
# `einmo list` walks input/ plus every stage tree, so a test present only in a
# stage (input deleted) or only in output/ (never promoted) still appears.
list_args=("list" "$SUITE")
[[ -n "$FILTER" ]] && list_args+=("--filter" "$FILTER")
(( differing_only )) && list_args+=("--differing")

mapfile -t rows < <("$EINMO" "${list_args[@]}" 2>/dev/null || true)

if (( shuffle )) && (( ${#rows[@]} > 0 )); then
    mapfile -t rows < <(printf '%s\n' "${rows[@]}" | shuf)
fi

if (( ${#rows[@]} == 0 )); then
    echo "No tests to review in $SUITE${FILTER:+ matching '$FILTER'}."
    exit 0
fi

# --- where reviewer notes survive the loop --------------------------------
REVIEW_DIR="${POOR_EINMO_DIR:-$(mktemp -d -t poor_einmo.XXXXXX)}"
mkdir -p "$REVIEW_DIR"
TMP=$(mktemp -d)

# Vim writes swap/backup/undo files next to the file being edited by default.
# Reviewing an input under `input/` therefore drops `.name.foo.swp` INTO the
# suite — einmo skips dot-files when walking, but the droppings still litter
# the tree, land in `git status`, and (if a stale one is ever opened) breed
# `.name.foo.swp.swp`. Give vim its own scratch directories for this run and
# clean them up on exit. Trailing `//` makes vim encode the full path into the
# swap file's name, so two same-named tests in different FOOP dirs cannot
# collide.
VIMTMP="$TMP/vim"
mkdir -p "$VIMTMP/swap" "$VIMTMP/backup" "$VIMTMP/undo"
VIM_OPTS=(
    -c "set directory=$VIMTMP/swap//"
    -c "set backupdir=$VIMTMP/backup//"
    -c "set undodir=$VIMTMP/undo//"
)

trap 'rm -rf "$TMP"' EXIT

echo "poor_einmo: ${#rows[@]} test(s) in $SUITE${FILTER:+ (filter: $FILTER)}"
(( differing_only )) && echo "            differing only — stages that agree are skipped"
(( dry_run ))        && echo "            DRY RUN — vimdiff calls are echoed, not executed"
echo "            notes kept in $REVIEW_DIR"
echo

promote_checked=()     # tests where you typed `promote` in the checked pane
promote_verified=()    # ... in the verified pane
send_to_agent_list=()  # tests where you left instructions
noop_list=()           # tests you did not touch

# The vocabulary. A pane must contain ONE of these words and nothing else —
# whitespace is stripped and case ignored, but any extra text means you were
# talking to an agent, not issuing a verb.
#
# Synonyms exist because a reviewer should not have to remember which word this
# particular tool chose. They are exact single words, not prefixes: "lgtm apart
# from line 3" is a note, as it should be.
PROMOTE_WORDS=(promote promoted approve approved lgtm sgtm)
SKIP_WORDS=(skip pass idk)
STOP_WORDS=(stop)

# `pane_says <file> <word>...` — true when the pane's whole content is exactly
# one of `word...`, ignoring surrounding whitespace and case.
pane_says() {
    local file="$1"; shift
    local content word
    content="$(tr -d '[:space:]' < "$file" | tr '[:upper:]' '[:lower:]')"
    for word in "$@"; do
        [[ "$content" == "$word" ]] && return 0
    done
    return 1
}

idx=0
for row in "${rows[@]}"; do
    idx=$((idx + 1))
    # `einmo list` prints: <mirror-path>\t<differ|same>\t<stage marks>
    rel="${row%%$'\t'*}"                       # e.g. misc/simple_addition.foo.einmo
    marks="${row##*$'\t'}"
    test_path="${rel%.einmo}"                  # e.g. misc/simple_addition.foo

    # Never open an editor swap/backup file: `einmo list` already skips
    # dot-prefixed entries, but a stale artifact in a stage would still route
    # here — and vim on a .swp makes a .swp.swp.
    case "$(basename "$test_path")" in
        .*) echo "── [$idx/${#rows[@]}] $test_path"
            echo "   · hidden file — skipped (editor droppings are not tests)"
            noop_list+=("$test_path")
            continue ;;
    esac

    in_f="$SUITE/input/$test_path"
    base=$(basename "$test_path")

    # Render each stage's signed body (einmo verifies-on-inspect, then strips
    # STAMPS/metadata). A missing artifact yields a marker pane, so the 4-way
    # diff still lines up.
    declare -A pane=()
    for stage in output checked verified; do
        f="$SUITE/$stage/$rel"
        p="$TMP/$stage--$base"
        if [[ -f "$f" ]]; then
            if ! "$EINMO" body "$f" > "$p" 2>"$TMP/err"; then
                { echo "(( $stage: REFUSED — einmo could not verify this artifact ))"
                  echo; cat "$TMP/err"; } > "$p"
            fi
        else
            echo "(( no $stage artifact — type promote to accept the previous stage ))" > "$p"
        fi
        pane[$stage]="$p"
        cp "$p" "$p.orig"          # pristine copy: the reviewer's diff baseline
    done

    echo "── [$idx/${#rows[@]}] $test_path"
    echo "   $marks"

    if (( dry_run )); then
        # Debugging aid: show the call instead of locking the terminal.
        echo "   + vimdiff ${VIM_OPTS[*]} '$in_f' '${pane[output]}' '${pane[checked]}' '${pane[verified]}'"
        noop_list+=("$test_path")
        continue
    fi

    # 4-way: the source under test, then the three stages.
    if ! vimdiff "${VIM_OPTS[@]}" \
            "$in_f" "${pane[output]}" "${pane[checked]}" "${pane[verified]}" \
            </dev/tty >/dev/tty; then
        echo
        echo "poor_einmo: aborted at $test_path."
        break
    fi

    # --- read the reviewer's intent out of the panes -----------------------
    #
    # `stop` is checked FIRST and in EVERY pane: it is an escape hatch, so it
    # must not depend on which pane you happened to be in, and it must beat
    # every other reading. The current file is left completely alone — not
    # promoted, not recorded as a note, not counted as reviewed — and the loop
    # ends, reporting what the earlier files already decided.
    stopped=0
    for stage in output checked verified; do
        if pane_says "${pane[$stage]}" "${STOP_WORDS[@]}"; then
            stopped=1
            break
        fi
    done
    if (( stopped )); then
        echo "   ■ stop — leaving $test_path untouched and ending the review"
        break
    fi

    chk_changed=0; ver_changed=0
    cmp -s "${pane[checked]}"  "${pane[checked]}.orig"  || chk_changed=1
    cmp -s "${pane[verified]}" "${pane[verified]}.orig" || ver_changed=1

    if (( chk_changed == 0 && ver_changed == 0 )); then
        noop_list+=("$test_path")
        echo "   · unchanged — no action"
        continue
    fi

    # An explicit skip ("skip"/"pass"/"idk") is a deliberate non-decision: you
    # looked and chose not to rule. It lands in noop_list exactly like leaving
    # the pane alone — the difference is that you meant it, and it must not be
    # mistaken for a note to an agent.
    skipped_word=0
    for stage in checked verified; do
        if pane_says "${pane[$stage]}" "${SKIP_WORDS[@]}"; then
            skipped_word=1
            break
        fi
    done
    if (( skipped_word )); then
        noop_list+=("$test_path")
        echo "   · skipped — no action"
        continue
    fi

    acted=0
    if (( chk_changed )) && pane_says "${pane[checked]}" "${PROMOTE_WORDS[@]}"; then
        promote_checked+=("$rel")
        echo "   → promote output->checked"
        acted=1
    fi
    if (( ver_changed )) && pane_says "${pane[verified]}" "${PROMOTE_WORDS[@]}"; then
        promote_verified+=("$rel")
        echo "   → promote checked->verified"
        acted=1
    fi

    # Any other edit is a message to the agent: keep it verbatim.
    if (( acted == 0 )); then
        note="$REVIEW_DIR/${test_path//\//__}.note"
        {
            echo "# poor_einmo review note"
            echo "# test:  $test_path"
            echo "# suite: $SUITE"
            echo "# stages: $marks"
            echo
            if (( chk_changed )); then
                echo "## reviewer edits in the CHECKED pane (diff vs what einmo showed):"
                diff -u "${pane[checked]}.orig" "${pane[checked]}" \
                     --label "checked (as shown)" --label "checked (as you left it)" || true
                echo
            fi
            if (( ver_changed )); then
                echo "## reviewer edits in the VERIFIED pane (diff vs what einmo showed):"
                diff -u "${pane[verified]}.orig" "${pane[verified]}" \
                     --label "verified (as shown)" --label "verified (as you left it)" || true
            fi
        } > "$note"
        send_to_agent_list+=("$test_path")
        echo "   ✎ instructions recorded → $note"
    fi

    if ! (( full_review )); then
        read -r -p "   next? [Enter=continue, q=quit] " ans </dev/tty || true
        [[ "$ans" == "q" ]] && { echo "poor_einmo: stopped."; break; }
    fi
done

# --- the accumulated results ---------------------------------------------
echo
echo "══ poor_einmo results ══════════════════════════════════════════════"

if (( ${#noop_list[@]} )); then
    echo
    echo "noop_list (${#noop_list[@]}): ${noop_list[*]}"
fi

if (( ${#send_to_agent_list[@]} )); then
    echo
    echo "send_to_agent_list (${#send_to_agent_list[@]}) — you left instructions on these:"
    for t in "${send_to_agent_list[@]}"; do
        echo "  $t"
        echo "      note: $REVIEW_DIR/${t//\//__}.note"
    done
    echo
    echo "  Hand the notes to an agent:  ls $REVIEW_DIR/*.note"
fi

if (( ${#promote_checked[@]} )); then
    echo
    echo "promote output->checked (${#promote_checked[@]}) — run:"
    printf '  %s promote output->checked %s \\\n' "$EINMO" "$SUITE"
    printf '      %s\n' "${promote_checked[@]}"
fi

if (( ${#promote_verified[@]} )); then
    echo
    echo "promote checked->verified (${#promote_verified[@]}) — run (human passphrase):"
    printf '  %s promote checked->verified %s --interactive \\\n' "$EINMO" "$SUITE"
    printf '      %s\n' "${promote_verified[@]}"
fi

if (( ${#promote_checked[@]} == 0 && ${#promote_verified[@]} == 0 && ${#send_to_agent_list[@]} == 0 )); then
    echo
    echo "Nothing to do — every test reviewed was unchanged."
fi

echo
echo "poor_einmo is read-only: nothing was promoted, flagged, or signed."
