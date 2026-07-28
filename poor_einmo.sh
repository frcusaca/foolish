#!/usr/bin/env bash
set -euo pipefail

# poor_einmo.sh — the poor foolisher's einmo review loop (FOOP-64).
#
# For every test in an einmo suite, opens vim with a short instructions window
# on top and four tall tiles below:
#
#     [ status + instructions ]
#     input | output | checked | verified
#
# The source, what the FVM just produced, what was last reviewed, and what a
# human last signed — side by side, one test at a time. Diff mode starts OFF;
# press \d to diff the four tiles against each other (or :diffthis by hand).
# This is the stopgap until `einmo console-review` (FOOP-92 Phase 12, absorbed
# into FOOP-64) exists.
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
#      → FLAG IT. The file is moved to <stage>/flagged/ immediately on settle
#        (no signature needed — just a mv). Your note text is recorded in the
#        results summary.
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
# Between tests (unless -f) a prompt offers: Enter=continue · e=edit (reopen,
# your text kept) · r=revert (reopen fresh) · u=back (keep this file's answer,
# re-review the PREVIOUS file — the review springs back here afterwards) ·
# q=quit. A revisited file left alone KEEPS its earlier answer; a new verb
# replaces it.
#
# The script auto-flags (mv to <stage>/flagged/) but never promotes or signs —
# it prints the exact commands for the promotions you asked for, and you (or an
# agent) run them. Promotion stays a deliberate, attributable act.
#
# The file scan and body extraction are einmo's own (`einmo list`, `einmo body`),
# not re-implemented here: einmo verifies every stamp before showing a byte
# (verify-on-inspect), and its body view excludes STAMPS/metadata — so what you
# review is exactly what `einmo compare` matches on. Timestamp/key churn never
# reaches your eyes.
#
# ── USAGE ─────────────────────────────────────────────────────────────────
#
#     ./poor_einmo.sh [-D] [-f] [-s] [-n] [-e EINMO] <suite-dir> [name-filter]
#
#     -d   differing only — skip tests whose output/checked/verified agree;
#          this is the DEFAULT: a fully-agreeing, fully-verified test needs no
#          human attention, so it is not visited
#     -D   visit ALL tests, including fully-agreeing verified ones (fresh-eyes
#          re-review, retracting something previously signed)
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
#     ./poor_einmo.sh foolish-ubca/einmo_suite               # what needs a look
#     ./poor_einmo.sh -D foolish-ubca/einmo_suite            # everything
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
# In vim: one-key decisions \C promote->checked · \V promote->verified ·
#   \c mark checked (stay) · \v mark verified (stay) ·
#   \K kick (demote) highest stage · \S skip · \Q stop · \A abort — each writes
#   the verb into the right pane and finishes the test (\c/\v stay for :qa).
# Also: \d toggle diff in the current window · \D toggle diff on all four tiles · \i / \I
#   shrink/expand the top instructions panel · ]c / [c next/prev change ·
#   zz centre the current line · :qa finish this test · :qa! finish discarding
#   unsaved pane edits · :cq abort the whole loop.

differing_only=1     # the default: only visit tests that need human attention
full_review=0
shuffle=0
dry_run=0
EINMO=""

while getopts "dDfsne:h" opt; do
    case "$opt" in
        d) differing_only=1 ;;   # the default, kept for muscle memory
        D) differing_only=0 ;;   # visit ALL tests, fully-verified ones included
        f) full_review=1 ;;
        s) shuffle=1 ;;
        n) dry_run=1 ;;
        e) EINMO="$OPTARG" ;;
        h) sed -n '4,72p' "$0"; exit 0 ;;
        *) echo "Try: $0 -h" >&2; exit 2 ;;
    esac
done
shift $((OPTIND - 1))

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 [-D] [-f] [-s] [-n] [-e EINMO] <suite-dir> [name-filter]" >&2
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

# input/ must exist (no inputs, no suite). The stage dirs may legitimately be
# absent — git does not track empty dirs, so a freshly-checked-out suite with an
# unpopulated verified/ has no verified/ at all. Create the missing ones rather
# than refuse; an empty stage is a valid "nothing promoted here yet".
[[ -d "$SUITE/input" ]] || { echo "Not an einmo suite (no input/): $SUITE" >&2; exit 1; }
for d in output checked verified flagged; do
    [[ -d "$SUITE/$d" ]] || mkdir -p "$SUITE/$d"
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

# Escape a filename for use inside a -c "split <name>" command. Spaces only —
# suite or scratch paths carrying vim-special characters (| ") are not
# supported here.
vimesc() { printf '%s' "${1// /\\ }"; }


echo "poor_einmo: ${#rows[@]} test(s) in $SUITE${FILTER:+ (filter: $FILTER)}"
if (( differing_only )); then
    echo "            differing only — fully-agreeing tests are skipped (-D visits all)"
else
    echo "            ALL tests — including fully-agreeing verified ones (-D)"
fi
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
# If the file was already moved to <stage>/flagged/, move it back.
drop_flag() {
    local needle="$1" i keep_s=() keep_r=() keep_reason=()
    for i in "${!flag_rel[@]}"; do
        if [[ "${flag_rel[$i]}" == "$needle" ]]; then
            local staged="$SUITE/${flag_stage[$i]}/$needle"
            local flagd="$SUITE/${flag_stage[$i]}/flagged/$needle"
            if [[ ! -f "$staged" && -f "$flagd" ]]; then
                mkdir -p "$(dirname "$staged")"
                mv "$flagd" "$staged"
                echo "   ↩ unflagged ${flag_stage[$i]}/${needle%.einmo}"
            fi
            continue
        fi
        keep_s+=("${flag_stage[$i]}"); keep_r+=("${flag_rel[$i]}"); keep_reason+=("${flag_reason[$i]}")
    done
    flag_stage=("${keep_s[@]+"${keep_s[@]}"}")
    flag_rel=("${keep_r[@]+"${keep_r[@]}"}")
    flag_reason=("${keep_reason[@]+"${keep_reason[@]}"}")
}

# What has this review already recorded for a file? Echoes a short description,
# or nothing. Read by the `u` (back) prompt to show the answer being revisited,
# and by the unchanged path so a revisit left alone KEEPS its earlier answer
# instead of demoting it to "no action".
answer_of() {  # answer_of <mirror-path> <test-path>
    local r="$1" t="$2" x j
    for x in "${promote_checked[@]+"${promote_checked[@]}"}"; do
        if [[ "$x" == "$r" ]]; then echo "promote output->checked"; return 0; fi
    done
    for x in "${promote_verified[@]+"${promote_verified[@]}"}"; do
        if [[ "$x" == "$r" ]]; then echo "promote checked->verified"; return 0; fi
    done
    for x in "${retract_checked[@]+"${retract_checked[@]}"}"; do
        if [[ "$x" == "$r" ]]; then echo "retract from checked"; return 0; fi
    done
    for x in "${retract_verified[@]+"${retract_verified[@]}"}"; do
        if [[ "$x" == "$r" ]]; then echo "retract from verified"; return 0; fi
    done
    for j in "${!flag_rel[@]}"; do
        if [[ "${flag_rel[$j]}" == "$r" ]]; then echo "flag ${flag_stage[$j]}: ${flag_reason[$j]}"; return 0; fi
    done
    for x in "${skip_list[@]+"${skip_list[@]}"}"; do
        if [[ "$x" == "$t" ]]; then echo "skip"; return 0; fi
    done
    for x in "${noop_list[@]+"${noop_list[@]}"}"; do
        if [[ "$x" == "$t" ]]; then echo "no action"; return 0; fi
    done
    return 0
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

# The review cursor. `u` at the between-tests prompt rewinds it to the previous
# file (a "revisit"): that file's earlier answer is kept unless the revisit
# writes a new one, and when the excursion settles the cursor springs back to
# where `u` was first pressed (u_resume).
i=0             # 0-based cursor into rows
u_resume=-1     # where to spring back to after a `u` excursion (-1: not in one)
revisit=0       # this visit re-opens a file that already has an answer (via `u`)
while (( i < ${#rows[@]} )); do
    row="${rows[$i]}"
    idx=$((i + 1))
    jump=-1         # set by `u`: rewind the cursor here instead of advancing
    re_render=1     # rebuild the panes from the stages for a fresh file
    # A file may be re-opened two ways: `edit` keeps your text and lets you
    # adjust it; `revert` throws it away and rebuilds from the stages.
    while :; do
    # `einmo list` prints: <mirror-path>\t<differ|same>\t<stage marks>
    rel="${row%%$'\t'*}"                       # e.g. misc/simple_addition.foo.einmo
    marks="${row##*$'\t'}"
    verdict="${row#*$'\t'}"; verdict="${verdict%%$'\t'*}"   # differ | same
    test_path="${rel%.einmo}"                  # e.g. misc/simple_addition.foo

    # Never open an editor swap/backup file: `einmo list` already skips
    # dot-prefixed entries, but a stale artifact in a stage would still route
    # here — and vim on a .swp makes a .swp.swp.
    case "$(basename "$test_path")" in
        .*) echo "── [$idx/${#rows[@]}] $test_path"
            echo "   · hidden file — skipped (editor droppings are not tests)"
            undo_last_decision   # a `u` revisit must not double-count it
            noop_list+=("$test_path")
            break ;;
    esac

    in_f="$SUITE/input/$test_path"
    base=$(basename "$test_path")

    # Render each stage's signed body (einmo verifies-on-inspect, then strips
    # STAMPS/metadata). A missing artifact yields an EMPTY pane — the stage
    # really is empty; all instructions live in the top panel. Whatever you
    # type into an empty pane is its whole content, verb or note. Skipped on
    # an `edit` re-open so the reviewer's text survives; rebuilt on a `revert`.
    if (( re_render )); then
        declare -A pane=()
        for stage in output checked verified; do
            f="$SUITE/$stage/$rel"
            f_flagged="$SUITE/$stage/flagged/$rel"
            p="$TMP/$stage--$base"
            if [[ -f "$f" ]]; then
                if ! "$EINMO" body "$f" > "$p" 2>"$TMP/err"; then
                    { echo "(( $stage: REFUSED — einmo could not verify this artifact ))"
                      echo; cat "$TMP/err"; } > "$p"
                fi
            elif [[ -f "$f_flagged" ]]; then
                if ! "$EINMO" body "$f_flagged" > "$p" 2>"$TMP/err"; then
                    { echo "(( $stage/flagged: REFUSED — einmo could not verify this artifact ))"
                      echo; cat "$TMP/err"; } > "$p"
                fi
            else
                : > "$p"
            fi
            pane[$stage]="$p"
            cp "$p" "$p.orig"
        done

        flagged_stages=()
        for stage in output checked verified; do
            [[ -f "$SUITE/$stage/flagged/$rel" ]] && flagged_stages+=("$stage")
        done
        if (( ${#flagged_stages[@]} )); then
            echo "   ⚑ previously flagged in: ${flagged_stages[*]}"
            if (( full_review )); then
                echo "   · full-review mode — keeping flag, skipping"
                noop_list+=("$test_path")
                break
            fi
            while :; do
                read -r -p "   [k]eep flag and move on, or [e]dit flag? " ans </dev/tty || ans=""
                case "${ans,,}" in
                    k*|"")
                        echo "   · keeping flag — skipping"
                        noop_list+=("$test_path")
                        break 2 ;;
                    e*)
                        for stage in "${flagged_stages[@]}"; do
                            src="$SUITE/$stage/flagged/$rel"
                            dst="$SUITE/$stage/$rel"
                            if [[ -f "$src" ]]; then
                                mkdir -p "$(dirname "$dst")"
                                mv "$src" "$dst"
                                echo "   ↩ unflagged $stage/${rel%.einmo}"
                            fi
                        done
                        break ;;
                esac
            done
        fi
    fi

    echo "── [$idx/${#rows[@]}] $test_path"
    echo "   $marks"

    # Per-test vim session config, sourced with `-c` (AFTER the user's vimrc
    # loads) so it wins for this review:
    #   - the status bar carries the vim key reminders (it is full width under
    #     the top panel, so they stay readable there even when the four tiles
    #     squash their own copies);
    #   - \d (backslash is vim's default leader): toggle diff mode in the
    #     window the cursor is in; \D: toggle it across all four file tiles.
    #     The instructions window never joins the diff either way;
    #   - \i / \I: shrink the top info panel to its status line / expand it to
    #     show every line, without moving the cursor.
    status_text='poor_einmo · \c/\C checked · \v/\V verified · \d diff here · \D diff all · ]c/[c jump · :qa done · :cq abort · \I for bigger info window'

    # One-key decisions write the verb into the right pane and leave (:xa).
    # \K (kick) demotes the HIGHEST stage present; 0 = nothing to retract.
    retract_target=0
    if   [[ -f "$SUITE/verified/$rel" ]]; then retract_target=5
    elif [[ -f "$SUITE/checked/$rel"  ]]; then retract_target=4
    fi
    cat > "$TMP/session.vim" <<SESSION
set laststatus=2
let &g:statusline = '$status_text'
" Windows: 1 instructions · 2 input · 3 output · 4 checked · 5 verified.
" PoorEinmoVerb replaces the target pane with one verb and exits via :xa
" (writes every modified pane), so the script reads the decision on return.
function! PoorEinmoVerb(target, verb)
  if a:target < 2
    echo 'poor_einmo: no stage artifact to act on'
    return
  endif
  execute a:target . 'wincmd w'
  silent %delete _
  call setline(1, a:verb)
  xa
endfunction
" Lowercase \c/\v: mark the pane with the verb but stay in vim (no :xa),
" so you can mark both checked and verified (or do other edits) before :qa.
function! PoorEinmoMark(target, verb)
  if a:target < 2
    echo 'poor_einmo: no stage artifact to act on'
    return
  endif
  execute a:target . 'wincmd w'
  silent %delete _
  call setline(1, a:verb)
  write
  echo 'poor_einmo: marked ' . (a:target == 4 ? 'checked' : 'verified') . ' as ' . a:verb . ' — :qa to finish'
endfunction
function! PoorEinmoMassEdit(verb)
  call PoorEinmoMark(4, a:verb)
  call PoorEinmoMark(5, a:verb)
  xa
endfunction

nnoremap <silent> \C :call PoorEinmoVerb(4, 'promote')<CR>
nnoremap <silent> \V :call PoorEinmoVerb(5, 'promote')<CR>
nnoremap <silent> \c :call PoorEinmoMark(4, 'promote')<CR>
nnoremap <silent> \v :call PoorEinmoMark(5, 'promote')<CR>
nnoremap <silent> \Y :call PoorEinmoMassEdit('promote')<CR>
nnoremap <silent> \K :call PoorEinmoVerb($retract_target, 'retract')<CR>
nnoremap <silent> \S :call PoorEinmoVerb(4, 'skip')<CR>
nnoremap <silent> \Q :call PoorEinmoVerb(4, 'stop')<CR>
nnoremap <silent> \A :call PoorEinmoVerb(4, 'abort')<CR>
function! PoorEinmoToggleDiffHere()
  if winnr() == 1 | return | endif
  if &diff | diffoff | else | diffthis | endif
endfunction
function! PoorEinmoToggleDiffAll()
  if getwinvar(2, '&diff')
    diffoff!
  else
    let l:cur = winnr()
    for l:w in range(2, winnr('\$'))
      execute l:w . 'wincmd w'
      diffthis
    endfor
    execute l:cur . 'wincmd w'
  endif
endfunction
function! PoorEinmoTopHeight(full)
  let l:cur = winnr()
  1wincmd w
  execute 'resize' (a:full ? line('\$') : 1)
  execute l:cur . 'wincmd w'
endfunction
nnoremap <silent> \d :call PoorEinmoToggleDiffHere()<CR>
nnoremap <silent> \D :call PoorEinmoToggleDiffAll()<CR>
nnoremap <silent> \i :call PoorEinmoTopHeight(0)<CR>
nnoremap <silent> \I :call PoorEinmoTopHeight(1)<CR>
SESSION

    # The top panel, two SEPARATE tables:
    #   1. "where we started" — the state of this test as the review opens:
    #      per-stage signature checks (einmo's verify-on-inspect status from
    #      `einmo list`) and a byte comparison between corresponding parts —
    #      the pristine rendered bodies (.orig), exactly what `einmo compare`
    #      matches on. Visible at the default panel height.
    #   2. the instruction table (verbs and the after-:qa prompt) — \I unfolds
    #      it; the vim key reminders live in the status bar.
    # Table cells are ASCII so printf's byte padding aligns the columns.
    o_stat="${marks#*output:}";   o_stat="${o_stat%% *}";  o_stat="${o_stat/—/-}"
    c_stat="${marks#*checked:}";  c_stat="${c_stat%% *}";  c_stat="${c_stat/—/-}"
    v_stat="${marks#*verified:}"; v_stat="${v_stat%% *}";  v_stat="${v_stat/—/-}"
    cmp_chk="-"; cmp_ver="-"
    if [[ -f "$SUITE/checked/$rel" ]]; then
        if [[ ! -f "$SUITE/output/$rel" ]]; then cmp_chk="no output"
        elif cmp -s "${pane[output]}.orig" "${pane[checked]}.orig"; then cmp_chk="same"
        else cmp_chk="DIFFERS"
        fi
    fi
    if [[ -f "$SUITE/verified/$rel" ]]; then
        if [[ ! -f "$SUITE/checked/$rel" ]]; then cmp_ver="no checked"
        elif cmp -s "${pane[checked]}.orig" "${pane[verified]}.orig"; then cmp_ver="same"
        else cmp_ver="DIFFERS"
        fi
    fi
    prev_answer="$(answer_of "$rel" "$test_path")"
    instr="$TMP/instructions"
    {
        echo "[$idx/${#rows[@]}] $test_path ($verdict)${prev_answer:+ · answer so far: $prev_answer}"
        echo '┌─ where we started ┬─ output ───┬─ checked ──┬─ verified ─┐'
        printf '│ %-17s │ %-10s │ %-10s │ %-10s │\n' 'artifact & stamps' "$o_stat" "$c_stat" "$v_stat"
        printf '│ %-17s │ %-10s │ %-10s │ %-10s │\n' 'vs previous stage' '.' "$cmp_chk" "$cmp_ver"
        echo '└───────────────────┴────────────┴────────────┴────────────┘'
        # Keys and commands are highlighted with `` quotes; cells stay ASCII
        # (plus the quotes) so printf's byte padding keeps the columns true.
        # Full 80-column width; the dense vim column gets the widest share.
        # Rows 1-4: the one-key decisions line up with the verbs they perform.
        row='│ %-27s │ %-19s │ %-24s │\n'
        echo   '┌─ vim ───────────────────────┬─ verbs (pane word) ─┬─ after :qa ──────────────┐'
        printf "$row" '`\C`/`\c` checked `\V`/`\v` verified' '`promote` `approve`' '`Enter` next / `q` quit'
        printf "$row" '`\Y` mass approve+cont.' '                   ' '                       '
        printf "$row" '`\K` kick highest stage'    '`retract` `demote`'  '`e` edit / `r` revert'
        printf "$row" '`\S` skip this test'        '`skip` `pass` `idk`' '`u` back to prev file'
        printf "$row" '`\Q` stop `\A` abort'       '`stop` / `abort`'    'untouched keeps answer'
        printf "$row" '`:qa` finish `:qa!` discard' 'other text = a note' 'empty tile = no artifact'
        printf "$row" '`\d`/`\D` diff win / all 4' ''                    ''
        printf "$row" '`\i`/`\I` panel min/max'    ''                    ''
        printf "$row" '`]c`/`[c` prev/next diff'   ''                    ''
        printf "$row" '`:cq` abort whole review'   ''                    ''
        echo   '└─────────────────────────────┴─────────────────────┴──────────────────────────┘'
    } > "$instr"
    # Below the tight tables: the wordy cheat-sheet. Now that \i/\I resize the
    # panel at will, this part can take the space it needs (\I shows it all).
    cat >> "$instr" <<'FINEPRINT'
──────────────────────────────── the fine print ────────────────────────────────
You review by EDITING the checked/verified panes; decisions are read at `:qa`.
Replace the ENTIRE pane with ONE word, alone: in the checked pane it means
"promote output->checked"; in the verified pane, "promote checked->verified".
  · `promote` `promoted` `approve` `approved` `lgtm` `sgtm`  all mean promote
  · `retract` `demote` `reexamine` `unpromote`  demote that stage for
    re-examination; retracting checked also removes any downstream verified
  · `skip` `pass` `idk`  you looked and chose not to rule — recorded as skipped
  · `stop`   end the review; earlier files keep their decisions
  · `abort`  leave NOW; nothing promoted, nothing reported
Anything else you write becomes a note for an agent: it is printed as an
`einmo flag` command with your text as the --reason, so the note travels INTO
the corpus (flagged/), not a temp file. An empty tile means that stage has no
artifact yet — write your verb into the empty pane to promote into it.
One-key decisions (each writes the verb into the right pane, saves every
modified pane via `:xa`, and finishes the test): `\C` promote output->checked ·
`\V` promote checked->verified · `\K` kick (demote) the highest stage present ·
`\S` skip · `\Q` stop the review · `\A` abort it.
Mark-only variants (write the verb but stay in vim — useful for marking both
panes before `:qa`): `\c` mark checked · `\v` mark verified.
Vim: `]c`/`[c` jump between differences · `zz` centre the line · `\d` toggles
diff in the window under the cursor · `\D` toggles all four tiles · `\i`/`\I`
shrink/expand this panel · `:qa` finishes this test (edits are read) · `:qa!`
finishes DISCARDING unsaved pane edits · `:cq` aborts the whole review loop.
After `:qa` the terminal prompt offers: `Enter`=next · `e`=edit (reopen, your
text kept) · `r`=revert (reopen fresh from the stages) · `u`=back (keep this
file's answer, re-review the previous file; review resumes here) · `q`=quit.
poor_einmo auto-flags notes (mv to <stage>/flagged/) on settle. Promotions and
retracts are printed as commands and only run behind the typed PROMOTE gate.
FINEPRINT

    review_opts=(
        "${VIM_OPTS[@]}"
        -c "source $TMP/session.vim"
    )

    if (( dry_run )); then
        # Debugging aid: show the call instead of locking the terminal.
        echo "   + vim ${review_opts[*]} '$instr' + tiles: '$in_f' '${pane[output]}' '${pane[checked]}' '${pane[verified]}'"
        noop_list+=("$test_path")
        break
    fi

    # Five windows: the instructions on top (short, read-only), then the source
    # under test and the three stages tiled vertically below. No diff mode
    # until you ask (\d, or :diffthis in the windows you choose).
    if ! vim "${review_opts[@]}" \
            -c "setlocal readonly nomodifiable" \
            -c "botright split $(vimesc "$in_f")" \
            -c "vertical belowright split $(vimesc "${pane[output]}")" \
            -c "vertical belowright split $(vimesc "${pane[checked]}")" \
            -c "vertical belowright split $(vimesc "${pane[verified]}")" \
            -c "1wincmd k" -c "resize 5" -c "wincmd j" \
            "$instr" \
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

    # A fresh answer supersedes whatever an earlier pass or a `u` revisit had
    # recorded for this file — replace, never stack.
    if (( chk_changed || ver_changed )); then
        undo_last_decision
    fi

    if (( chk_changed == 0 && ver_changed == 0 )); then
        prev_answer="$(answer_of "$rel" "$test_path")"
        if [[ -n "$prev_answer" ]]; then
            # A revisited file left alone keeps what the review already decided.
            echo "   · unchanged — keeping the answer already recorded: $prev_answer"
        else
            noop_list+=("$test_path")
            echo "   · unchanged — no action"
        fi
        if ! (( full_review )); then
            while :; do
                read -r -p "   next? [Enter=continue, e=edit, r=revert, u=back, q=quit] " ans </dev/tty || true
                case "${ans,,}" in
                    e*) echo "   ✎ edit — reopening with your text kept"
                        drop_flag "$rel"
                        re_render=0
                        continue 2 ;;
                    r*) echo "   ↺ revert — discarding this file's answer, reopening fresh"
                        drop_flag "$rel"
                        re_render=1
                        continue 2 ;;
                    u*|b*)
                        if (( i == 0 )); then
                            echo "   · already at the first file — nothing to go back to"
                            continue
                        fi
                        prev_rel="${rows[$((i - 1))]%%$'\t'*}"
                        prev_answer="$(answer_of "$prev_rel" "${prev_rel%.einmo}")"
                        echo "   ← back — this file keeps its answer; re-opening ${prev_rel%.einmo}${prev_answer:+ (answer so far: $prev_answer)}"
                        if (( u_resume < 0 )); then u_resume=$((i + 1)); fi
                        jump=$((i - 1))
                        break ;;
                    q*) echo "poor_einmo: stopped."; break 3 ;;
                    *)  break ;;
                esac
            done
        fi
        break
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
        break
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
    if (( acted == 0 && (chk_changed || ver_changed) )); then
        local_stage=""; note_text=""
        if (( ver_changed )); then
            local_stage=verified
            note_text="$(cat "${pane[verified]}")"
        else
            local_stage=checked
            note_text="$(cat "${pane[checked]}")"
        fi
        note_text="$(printf '%s' "$note_text" | tr '\n' ' ' | sed 's/  */ /g; s/^ //; s/ $//')"
        flag_stage+=("$local_stage")
        flag_rel+=("$rel")
        flag_reason+=("$note_text")
        send_to_agent_list+=("$test_path")
        acted=1
        echo "   ✎ note — will flag $local_stage/$test_path on settle"
    fi

    if ! (( full_review )); then
        # The decision is not final until you leave the file:
        #   edit   — reopen with YOUR text intact, to adjust it
        #   revert — discard this file's answer, reopen from the stages fresh
        #   back   — keep this file's answer, rewind to the PREVIOUS file; when
        #            that settles, the review springs back to resume here
        while :; do
            read -r -p "   next? [Enter=continue, e=edit, r=revert, u=back, q=quit] " ans </dev/tty || true
            case "${ans,,}" in
                e*) echo "   ✎ edit — reopening with your text kept"
                    undo_last_decision   # retract the recorded outcome; text stays
                    re_render=0
                    continue 2 ;;
                r*) echo "   ↺ revert — discarding this file's answer, reopening fresh"
                    undo_last_decision
                    re_render=1
                    continue 2 ;;
                u*|b*)
                    if (( i == 0 )); then
                        echo "   · already at the first file — nothing to go back to"
                        continue
                    fi
                    prev_rel="${rows[$((i - 1))]%%$'\t'*}"
                    prev_answer="$(answer_of "$prev_rel" "${prev_rel%.einmo}")"
                    echo "   ← back — this file keeps its answer; re-opening ${prev_rel%.einmo}${prev_answer:+ (answer so far: $prev_answer)}"
                    if (( u_resume < 0 )); then u_resume=$((i + 1)); fi
                    jump=$((i - 1))
                    break ;;
                q*) echo "poor_einmo: stopped."; break 3 ;;
                *)  break ;;
            esac
        done
    fi
    break   # this file is settled; on to the next
    done

    for fi in "${!flag_rel[@]}"; do
        [[ "${flag_rel[$fi]}" == "$rel" ]] || continue
        src="$SUITE/${flag_stage[$fi]}/$rel"
        if [[ -f "$src" ]]; then
            dest_dir="$SUITE/${flag_stage[$fi]}/flagged"
            mkdir -p "$(dirname "$dest_dir/$rel")"
            mv "$src" "$dest_dir/$rel"
            echo "   ▸ flagged ${flag_stage[$fi]}/$test_path → ${flag_stage[$fi]}/flagged/"
        fi
    done

    # Advance the cursor: a `u` rewind, a spring-back after one, or plain next.
    if (( jump >= 0 )); then
        i=$jump
        revisit=1
    elif (( u_resume >= 0 )); then
        i=$u_resume
        u_resume=-1
        revisit=0
    else
        i=$((i + 1))
        revisit=0
    fi
done

# Print a command block in a copy/pasteable way (one file per continued line).
show_cmd() {  # show_cmd "<prose>" <einmo-args...> -- <files...>
    local prose="$1"; shift
    local head=() f; while [[ "$1" != "--" ]]; do head+=("$1"); shift; done; shift
    echo
    echo "$prose"
    printf '  %s' "$EINMO"; printf ' %q' "${head[@]}"; printf ' \\\n'
    for f in "$@"; do printf '      %q \\\n' "$f"; done | sed '$ s/ \\$//'
}

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
    echo "flagged (${#flag_rel[@]}):"
    for i in "${!flag_rel[@]}"; do
        echo "  ${flag_stage[$i]}/${flag_rel[$i]%.einmo} — ${flag_reason[$i]}"
    done
fi

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
        checked_ok=1
        if (( ${#promote_checked[@]} )); then
            echo "→ running: promote output to checked"
            if ! "$EINMO" promote output to checked "$SUITE" -- "${promote_checked[@]}" </dev/tty; then
                checked_ok=0
                echo "⚠ promote output->checked failed — you can retry with:"
                show_cmd "" promote output to checked "$SUITE" -- "${promote_checked[@]}"
            fi
        fi
        if (( ${#promote_verified[@]} )); then
            verified_ok=0
            while :; do
                echo "→ running: promote checked to verified (you will be asked for your passphrase)"
                if "$EINMO" promote checked to verified "$SUITE" --interactive -- "${promote_verified[@]}" </dev/tty; then
                    verified_ok=1
                    break
                fi
                echo
                read -r -p "   passphrase mismatch — retry? [Y/n] " ans2 </dev/tty || ans2=""
                case "${ans2,,}" in
                    n*|q*) break ;;
                esac
            done
            if (( ! verified_ok )); then
                echo "⚠ promote checked->verified was not completed — you can retry with:"
                show_cmd "" promote checked to verified "$SUITE" --interactive -- "${promote_verified[@]}"
            fi
        fi
        if (( checked_ok && ( ${#promote_verified[@]} == 0 || verified_ok ) )); then
            echo "✓ promotions done."
        fi
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
