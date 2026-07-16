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
#
#   1b. You typed a RETRACT word (retract · demote · reexamine · unpromote)
#      → RETRACT (demote) that stage's artifact for re-examination. Retracting
#        checked also removes any downstream verified (the cascade). Like
#        promote, it is a printed command you run -- the script never touches
#        the corpus.
#
#   2. You changed a pane any other way (a note, a question, an @agent comment)
#      → FLAG IT. The results print an `einmo flag <suite> <stage> <file>
#        --reason "<your note>"` command: your note travels INTO the corpus
#        (flagged/) as the artifact's advisory reason, so nothing depends on a
#        temp file. Run it (behind the same gate) to act.
#
#   3. You typed a SKIP word (skip · pass · idk)
#      → NO ACTION, deliberately: you looked and chose not to rule. Accumulated
#        into `skip_list`, listed separately from files you never touched.
#
#   4. You changed nothing
#      → NO ACTION. Accumulated into `noop_list`, reported on one line.
#
#   5. You typed `stop` into ANY pane
#      → END THE REVIEW GRACEFULLY. The current file is left completely
#        untouched — not promoted, not recorded, not counted — and the loop ends,
#        printing the promotions and notes the EARLIER files already earned.
#
#   6. You typed `abort` into ANY pane
#      → LEAVE NOW. Nothing is promoted, nothing is reported, no results are
#        printed at all — not even for files you already reviewed. Exit 130.
#        Use `stop` to finish up gracefully; `abort` is for "get me out of
#        here". Read before every other word, so nothing can outrank it.
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
# SECURITY: the panes, your notes, and vim's swap/backup/undo files all hold the
# signed content under review. All scratch lives under mode-700 directories
# (u+rwx, og-rwx — readable/writable only by you), enforced on every run. On a
# shared host, prefer a private location: set POOR_EINMO_DIR to a directory of
# your own (an encrypted or tmpfs-backed path rather than shared /tmp); it is
# still forced to 700.
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
#     retract · demote · reexamine · unpromote                → retract (demote)
#     skip · pass · idk                                       → no action
#     stop                                                    → end, print results
#     abort                                                   → end NOW, print nothing
#
# In vimdiff: ]c / [c next/prev change · zz centre the current line ·
#   :qa finish this test · :qa! finish discarding unsaved pane edits ·
#   :cq abort the whole loop.

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

# --- private scratch space (SECURITY) -------------------------------------
#
# EVERYTHING under our scratch dirs — the panes, the .orig baselines, the notes,
# the vim swap/backup/undo files — is the SIGNED CONTENT under review. On a
# shared host, any of it being group- or world-readable leaks it.
#
# The clean lever is `umask`: it is the process-wide DEFAULT for how new files
# and directories are created. `umask 077` makes every file we create 600 and
# every directory 700 — private at birth, no per-file chmod chase. We set it
# once here, before any scratch exists, so it governs all of it.
#
# `harden_dir` is the belt to umask's braces: it forces an EXISTING directory
# (and its contents) to owner-only and REFUSES to run if that did not take —
# for a user-supplied $POOR_EINMO_DIR that may predate our umask or arrive
# world-writable. mktemp already makes 0700, but we do not trust defaults for
# material this sensitive.
#
# To place scratch somewhere private of your own (encrypted, or tmpfs-backed
# rather than shared /tmp), set POOR_EINMO_DIR — it is still forced private.
umask 077

harden_dir() {  # force owner-only on the dir AND its contents; refuse if it fails
    chmod -R go-rwx "$1" 2>/dev/null || true
    chmod 700 "$1" 2>/dev/null || true
    local mode
    mode="$(stat -c '%a' "$1" 2>/dev/null || echo '?')"
    if [[ "$mode" != "700" ]]; then
        echo "poor_einmo: refusing to use $1 — could not secure it to mode 700 (got $mode)." >&2
        echo "            Its contents would be the signed material under review." >&2
        exit 1
    fi
    # No group/other bit may survive anywhere beneath it.
    if [[ -n "$(find "$1" \( -perm /0077 \) -print -quit 2>/dev/null)" ]]; then
        echo "poor_einmo: refusing to use $1 — something under it is group/other-accessible." >&2
        exit 1
    fi
}

# Scratch: pane renders, .orig baselines, vim swap/backup/undo. All ephemeral;
# nothing durable lives here (notes now become `flag` commands into the corpus).
#
# We always create OUR OWN subdirectory and remove exactly that on exit — never
# a directory we did not make. If you set POOR_EINMO_DIR, our subdir is created
# INSIDE it (so you get your private/encrypted/tmpfs location) and only the
# subdir is removed; your directory is left as you supplied it.
#
# The trap is set the instant the subdir exists, BEFORE any early exit, so even
# a no-tests run leaves nothing behind.
if [[ -n "${POOR_EINMO_DIR:-}" ]]; then
    mkdir -p "$POOR_EINMO_DIR"
    harden_dir "$POOR_EINMO_DIR"
    TMP="$(mktemp -d "$POOR_EINMO_DIR/poor_einmo.XXXXXX")"
else
    TMP="$(mktemp -d -t poor_einmo.XXXXXX)"
fi
harden_dir "$TMP"

# Exiting poor_einmo removes OUR scratch subdir — nothing survives the run.
# The `[[ -d ]]` guard makes a bare `rm -rf "$TMP"` safe even if $TMP were ever
# empty or unset (which `rm -rf ""` / `rm -rf /` would make catastrophic).
cleanup() {
    if [[ -n "${TMP:-}" && -d "$TMP" ]]; then
        rm -rf "$TMP"
    fi
}
trap cleanup EXIT INT TERM

if (( shuffle )) && (( ${#rows[@]} > 0 )); then
    mapfile -t rows < <(printf '%s\n' "${rows[@]}" | shuf)
fi

if (( ${#rows[@]} == 0 )); then
    echo "No tests to review in $SUITE${FILTER:+ matching '$FILTER'}."
    exit 0
fi



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
harden_dir "$VIMTMP"; harden_dir "$VIMTMP/swap"; harden_dir "$VIMTMP/backup"; harden_dir "$VIMTMP/undo"
# `--cmd` runs these BEFORE any file is loaded. With `-c` they run too late:
# vim has already opened the command-line files and chosen their swap location
# next to them, inside the suite. (Verified: `-c` left `.foo.swp` in input/;
# `--cmd` puts it in $VIMTMP.) The trailing `//` encodes the full path into the
# swap name, so same-named tests in different FOOP dirs cannot collide.
VIM_OPTS=(
    --cmd "set directory=$VIMTMP/swap//"
    --cmd "set backupdir=$VIMTMP/backup//"
    --cmd "set undodir=$VIMTMP/undo//"
)


echo "poor_einmo: ${#rows[@]} test(s) in $SUITE${FILTER:+ (filter: $FILTER)}"
(( differing_only )) && echo "            differing only — stages that agree are skipped"
(( dry_run ))        && echo "            DRY RUN — vimdiff calls are echoed, not executed"
echo "            scratch (removed on exit): $TMP"
echo

promote_checked=()     # tests where you typed a promote word in the checked pane
promote_verified=()    # ... in the verified pane
retract_checked=()     # tests to demote from checked/ (cascades to verified)
retract_verified=()    # tests to demote from verified/
flag_stage=()          # parallel arrays: flag this artifact ...
flag_rel=()            # ... at this mirror path ...
flag_reason=()         # ... with this note as its advisory reason
send_to_agent_list=()  # tests where you left instructions
skip_list=()           # tests you deliberately passed on (skip/pass/idk)
noop_list=()           # tests you did not touch at all

# The placeholder shown when a stage holds no artifact. It doubles as the
# cheat-sheet: the whole vocabulary sits in front of you exactly when you need
# it. Every line starts with MARKER_SIGIL so the script can tell "left the
# instructions alone" from "wrote a verb".
MARKER_SIGIL="!! poor_einmo:"
marker_text() {
    local stage="$1"
    cat <<MARKER
$MARKER_SIGIL no $stage artifact yet.
$MARKER_SIGIL
$MARKER_SIGIL Replace EVERYTHING here with ONE word, then :qa
$MARKER_SIGIL
$MARKER_SIGIL   promote promoted approve approved lgtm sgtm
$MARKER_SIGIL        accept the previous stage into this one
$MARKER_SIGIL   skip pass idk      no action; move on
$MARKER_SIGIL   stop               end the review, keep what you decided
$MARKER_SIGIL   abort              leave now, discard everything
$MARKER_SIGIL
$MARKER_SIGIL Anything else becomes a note for an agent.
$MARKER_SIGIL Leave this text as-is and the file counts as unreviewed.
$MARKER_SIGIL Vim command reminders: You can turn diff mode off and on using:
$MARKER_SIGIL 
$MARKER_SIGIL   :diffoff
$MARKER_SIGIL   :diffthis
MARKER
}

# True when these instructions survived in the pane.
marker_left_intact() {
    grep -qF "$MARKER_SIGIL" "$1"
}

# The reviewer's own text in a pane: everything that is NOT a marker line.
strip_marker() {
    grep -vF "$MARKER_SIGIL" "$1"
}

# Drop `$1` from array `$2` (by name). Used to retract a decision on re-edit.
drop_from() {
    local needle="$1" name="$2"
    local -n arr="$name"
    local kept=() item
    for item in "${arr[@]+"${arr[@]}"}"; do
        [[ "$item" == "$needle" ]] || kept+=("$item")
    done
    arr=("${kept[@]+"${kept[@]}"}")
}

# Retract everything the last pass recorded for the file being re-edited, so a
# corrected answer replaces the mistake instead of stacking on top of it.
undo_last_decision() {
    drop_from "$rel"       promote_checked
    drop_from "$rel"       promote_verified
    drop_from "$rel"       retract_checked
    drop_from "$rel"       retract_verified
    drop_flag "$rel"
    drop_from "$test_path" send_to_agent_list
    drop_from "$test_path" skip_list
    drop_from "$test_path" noop_list
}

# Remove a queued flag for $1 (parallel arrays kept in step).
drop_flag() {
    local needle="$1" i keep_s=() keep_r=() keep_reason=()
    for i in "${!flag_rel[@]}"; do
        [[ "${flag_rel[$i]}" == "$needle" ]] && continue
        keep_s+=("${flag_stage[$i]}"); keep_r+=("${flag_rel[$i]}"); keep_reason+=("${flag_reason[$i]}")
    done
    flag_stage=("${keep_s[@]+"${keep_s[@]}"}")
    flag_rel=("${keep_r[@]+"${keep_r[@]}"}")
    flag_reason=("${keep_reason[@]+"${keep_reason[@]}"}")
}

# The vocabulary. A pane must contain ONE of these words and nothing else —
# whitespace is stripped and case ignored, but any extra text means you were
# talking to an agent, not issuing a verb.
#
# Synonyms exist because a reviewer should not have to remember which word this
# particular tool chose. They are exact single words, not prefixes: "lgtm apart
# from line 3" is a note, as it should be.
PROMOTE_WORDS=(promote promoted approve approved lgtm sgtm)
RETRACT_WORDS=(retract demote reexamine unpromote)
SKIP_WORDS=(skip pass idk)
STOP_WORDS=(stop)
ABORT_WORDS=(abort)

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
    re_render=1     # rebuild the panes from the stages for a fresh file
    # A file may be re-opened two ways: `edit` keeps your text and lets you
    # adjust it; `revert` throws it away and rebuilds from the stages.
    while :; do
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
    # diff still lines up. Skipped on an `edit` re-open so the reviewer's text
    # survives; rebuilt on a `revert`.
    if (( re_render )); then
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
                marker_text "$stage" > "$p"
            fi
            pane[$stage]="$p"
            cp "$p" "$p.orig"      # pristine copy: the reviewer's diff baseline
        done
    fi

    echo "── [$idx/${#rows[@]}] $test_path"
    echo "   $marks"

    # A persistent status line naming what you are looking at and why it is in
    # the queue: the test, its differ/same verdict, and the per-stage marks
    # (output/checked/verified: ok | a status | — for absent). Set with `-c`
    # (after the user's vimrc loads) so it wins for this review; % is doubled so
    # our text is not taken as a statusline format item.
    status_text="poor_einmo │ $test_path │ ${marks//%/%%}"
    review_opts=(
        "${VIM_OPTS[@]}"
        -c "set laststatus=2"
        -c "set statusline=${status_text// /\\ }"
    )

    if (( dry_run )); then
        # Debugging aid: show the call instead of locking the terminal.
        echo "   + vimdiff ${review_opts[*]} '$in_f' '${pane[output]}' '${pane[checked]}' '${pane[verified]}'"
        noop_list+=("$test_path")
        break
    fi

    # 4-way: the source under test, then the three stages.
    if ! vimdiff "${review_opts[@]}" \
            "$in_f" "${pane[output]}" "${pane[checked]}" "${pane[verified]}" \
            </dev/tty >/dev/tty; then
        echo
        echo "poor_einmo: aborted at $test_path."
        break 2
    fi

    # --- read the reviewer's intent out of the panes -----------------------
    #
    # `stop` is checked FIRST and in EVERY pane: it is an escape hatch, so it
    # must not depend on which pane you happened to be in, and it must beat
    # every other reading. The current file is left completely alone — not
    # promoted, not recorded as a note, not counted as reviewed — and the loop
    # ends, reporting what the earlier files already decided.
    # `abort` is read first and in every pane: it is the strongest word in the
    # vocabulary, so nothing may outrank it. It leaves immediately — no
    # promotion list, no notes, no results — because "abort" means you want out
    # NOW, not a tidy summary of a review you are repudiating.
    for stage in output checked verified; do
        if pane_says "${pane[$stage]}" "${ABORT_WORDS[@]}"; then
            echo "   ✖ abort — leaving $test_path untouched; nothing promoted, nothing reported"
            exit 130
        fi
    done

    stopped=0
    for stage in output checked verified; do
        if pane_says "${pane[$stage]}" "${STOP_WORDS[@]}"; then
            stopped=1
            break
        fi
    done
    if (( stopped )); then
        echo "   ■ stop — leaving $test_path untouched and ending the review"
        break 2
    fi

    chk_changed=0; ver_changed=0
    cmp -s "${pane[checked]}"  "${pane[checked]}.orig"  || chk_changed=1
    cmp -s "${pane[verified]}" "${pane[verified]}.orig" || ver_changed=1

    # A changed pane that STILL holds the instructions means the reviewer wrote
    # *around* them — almost always "typed the verb under the cheat-sheet".
    # Read literally that is a note; read charitably it is the promotion they
    # believe they just made. Neither guess is ours, so ask.
    marker_choice=""
    for stage in checked verified; do
        case "$stage" in
            checked)  (( chk_changed )) || continue ;;
            verified) (( ver_changed )) || continue ;;
        esac
        marker_left_intact "${pane[$stage]}" || continue

        echo "   ⚠ the $stage pane still holds the instructions, plus your text."
        echo "     A verb must stand ALONE — as written this becomes a note."
        read -r -p "     [r]e-view now (to promote), [f]lag as a note, [s]kip? " ans </dev/tty || ans=s
        case "${ans,,}" in
            r*) echo "     ↺ re-opening fresh — replace ALL the text with one word"
                marker_choice=redo ;;
            f*) echo "     ✎ recorded as a note"
                marker_choice=flag ;;
            *)  echo "     · skipped"
                marker_choice=skip ;;
        esac
        break
    done
    case "$marker_choice" in
        redo) re_render=1; continue ;;                     # re-open fresh
        skip) skip_list+=("$test_path"); break ;;          # settled: skipped
    esac

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
        skip_list+=("$test_path")
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
    if (( chk_changed )) && pane_says "${pane[checked]}" "${RETRACT_WORDS[@]}"; then
        retract_checked+=("$rel")
        echo "   ↩ retract from checked (cascades to verified)"
        acted=1
    fi
    if (( ver_changed )) && pane_says "${pane[verified]}" "${RETRACT_WORDS[@]}"; then
        retract_verified+=("$rel")
        echo "   ↩ retract from verified"
        acted=1
    fi

    # Any other edit is a note: flag the artifact with the note as its reason,
    # so it travels INTO the corpus (flagged/) rather than a throwaway temp
    # file. The note is what you typed; a multi-line note is folded to one line
    # for the advisory (the full text is shown in the printed command).
    if (( acted == 0 )); then
        # Which stage did you write on, and what did you write? Prefer verified.
        local_stage=""; note_text=""
        if (( ver_changed )); then
            local_stage=verified
            note_text="$(strip_marker "${pane[verified]}")"
        else
            local_stage=checked
            note_text="$(strip_marker "${pane[checked]}")"
        fi
        # Fold to a single line for the advisory reason.
        note_text="$(printf '%s' "$note_text" | tr '\n' ' ' | sed 's/  */ /g; s/^ //; s/ $//')"
        flag_stage+=("$local_stage")
        flag_rel+=("$rel")
        flag_reason+=("$note_text")
        send_to_agent_list+=("$test_path")
        echo "   ✎ note recorded → will flag $local_stage/$test_path"
    fi

    if ! (( full_review )); then
        # The decision is not final until you leave the file:
        #   edit   — reopen with YOUR text intact, to adjust it
        #   revert — discard this file's answer, reopen from the stages fresh
        read -r -p "   next? [Enter=continue, e=edit, r=revert, q=quit] " ans </dev/tty || true
        case "${ans,,}" in
            e*) echo "   ✎ edit — reopening with your text kept"
                undo_last_decision   # retract the recorded outcome; text stays
                re_render=0
                continue ;;
            r*) echo "   ↺ revert — discarding this file's answer, reopening fresh"
                undo_last_decision
                re_render=1
                continue ;;
            q*) echo "poor_einmo: stopped."; break 2 ;;
        esac
    fi
    break   # this file is settled; on to the next
    done
done

# --- the accumulated results ---------------------------------------------
echo
echo "══ poor_einmo results ══════════════════════════════════════════════"

if (( ${#noop_list[@]} )); then
    echo
    echo "noop_list (${#noop_list[@]}): ${noop_list[*]}"
fi

if (( ${#skip_list[@]} )); then
    echo
    echo "skip_list (${#skip_list[@]}) — you looked and chose not to rule:"
    printf '  %s\n' "${skip_list[@]}"
fi

if (( ${#flag_rel[@]} )); then
    echo
    echo "‼ YOU MUST RUN THESE to record your notes — each flags the artifact"
    echo "  (moves it to flagged/) with your note as its advisory reason:"
    for i in "${!flag_rel[@]}"; do
        show_cmd "  # ${flag_rel[$i]%.einmo}" \
            flag "$SUITE" "${flag_stage[$i]}" --reason "${flag_reason[$i]}" \
            -- "${flag_rel[$i]}"
    done
fi

# Print a command block in a copy/pasteable way (one file per continued line).
show_cmd() {  # show_cmd "<prose>" <einmo-args...> -- <files...>
    local prose="$1"; shift
    local head=() f; while [[ "$1" != "--" ]]; do head+=("$1"); shift; done; shift
    echo
    echo "$prose"
    printf '  %s' "$EINMO"; printf ' %q' "${head[@]}"; printf ' \\\n'
    for f in "$@"; do printf '      %q \\\n' "$f"; done | sed '$ s/ \\$//'
}

# The PROMOTE gate. poor_einmo can RUN promotions for you, but only after you
# type the whole word PROMOTE in capitals — a deliberate, unmistakable act,
# never a stray keystroke. Retractions and notes are NEVER auto-run: they
# remove signed baselines or need a human, so they are always printed for you
# to run yourself, with a stern reminder.
have_promotions=$(( ${#promote_checked[@]} + ${#promote_verified[@]} ))

if (( ${#promote_checked[@]} )); then
    show_cmd "promote output to checked (${#promote_checked[@]}):" \
        promote output to checked "$SUITE" -- "${promote_checked[@]}"
fi
if (( ${#promote_verified[@]} )); then
    show_cmd "promote checked to verified (${#promote_verified[@]}) — needs your passphrase:" \
        promote checked to verified "$SUITE" --interactive -- "${promote_verified[@]}"
fi

if (( have_promotions )); then
    echo
    echo "════════════════════════════════════════════════════════════════════"
    read -r -p "Type PROMOTE (all caps) to RUN the promotions above, or Enter to skip: " ans </dev/tty || ans=""
    if [[ "$ans" == "PROMOTE" ]]; then
        if (( ${#promote_checked[@]} )); then
            echo "→ running: promote output to checked"
            "$EINMO" promote output to checked "$SUITE" -- "${promote_checked[@]}" </dev/tty
        fi
        if (( ${#promote_verified[@]} )); then
            echo "→ running: promote checked to verified (you will be asked for your passphrase)"
            "$EINMO" promote checked to verified "$SUITE" --interactive -- "${promote_verified[@]}" </dev/tty
        fi
        echo "✓ promotions done."
    else
        echo "⚠ NOT promoted. The commands above must be run for anything to take effect."
    fi
fi

# Retractions: never auto-run (they remove signed baselines).
if (( ${#retract_checked[@]} )); then
    show_cmd "‼ YOU MUST RUN THIS to retract from checked — it removes the checked artifact AND any verified (${#retract_checked[@]}):" \
        retract "$SUITE" checked -- "${retract_checked[@]}"
fi
if (( ${#retract_verified[@]} )); then
    show_cmd "‼ YOU MUST RUN THIS to retract from verified (${#retract_verified[@]}):" \
        retract "$SUITE" verified -- "${retract_verified[@]}"
fi

if (( ${#promote_checked[@]} == 0 && ${#promote_verified[@]} == 0 && ${#retract_checked[@]} == 0 && ${#retract_verified[@]} == 0 && ${#send_to_agent_list[@]} == 0 )); then
    echo
    if (( ${#skip_list[@]} )); then
        echo "Nothing to do — every test reviewed was skipped or unchanged."
    else
        echo "Nothing to do — every test reviewed was unchanged."
    fi
fi

if ! { (( have_promotions )) && [[ "${ans:-}" == "PROMOTE" ]]; }; then
    echo
    echo "poor_einmo did not change the corpus: run the commands above to act on your review."
fi
