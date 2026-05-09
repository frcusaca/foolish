#!/usr/bin/env python3
"""
foop_check.py — Utility for managing FOOP numbering.

FOOPs use a little-endian numbering convention. The filename digits ARE
the identifier — written front-to-back. The chronological order is
recovered by reversing the digits and reading as decimal (this is the
sort key, stored in the frontmatter `foop:` field).

    Filename       Identifier      Sort key (chronological order)
    FOOP-1.md      FOOP-1          1
    FOOP-2.md      FOOP-2          2
    ...
    FOOP-9.md      FOOP-9          9
    FOOP-01.md     FOOP-01         10
    FOOP-11.md     FOOP-11         11
    FOOP-21.md     FOOP-21         12
    FOOP-31.md     FOOP-31         13
    FOOP-41.md     FOOP-41         14
    FOOP-51.md     FOOP-51         15
    FOOP-61.md     FOOP-61         16
    FOOP-71.md     FOOP-71         17
    FOOP-81.md     FOOP-81         18
    FOOP-91.md     FOOP-91         19
    FOOP-02.md     FOOP-02         20
    FOOP-12.md     FOOP-12         21
    ...

Per AGENTS.md:
- In free text, use the space form: "FOOP 51" (no dash).
- In filenames, code references, and formal citations, use the dash
  form: "FOOP-51".
- Always use the FILENAME DIGITS as the identifier — never use the sort
  key as the identifier in prose.

This script provides:

  check       — verify all FOOPs are consecutive (no gaps in sort key).
  get_last    — print the identifier of the most-recently-created FOOP.
  gen_next    — print the identifier for the next FOOP to be created.
  list        — list all FOOPs in chronological order, oldest to newest.

Usage:
    python3 foop_check.py check
    python3 foop_check.py get_last
    python3 foop_check.py gen_next
    python3 foop_check.py list
"""

import argparse
import re
import sys
from pathlib import Path


FOOP_DIR = Path(__file__).resolve().parent.parent
FILENAME_RE = re.compile(r"^FOOP-(\d+)\.md$")


def filename_to_sort_key(filename: str) -> int | None:
    """
    Convert a FOOP filename to its chronological sort key.

    The filename has the form FOOP-<digits>.md where <digits> are the
    identifier written little-endian (front-to-back). The sort key is
    obtained by reversing the digits and reading as decimal.

    Returns None if the filename does not match the FOOP pattern.

    Examples:
        FOOP-1.md   -> 1
        FOOP-9.md   -> 9
        FOOP-01.md  -> 10
        FOOP-11.md  -> 11
        FOOP-31.md  -> 13
        FOOP-51.md  -> 15
    """
    m = FILENAME_RE.match(filename)
    if not m:
        return None
    digits = m.group(1)
    return int(digits[::-1])


def sort_key_to_filename_digits(sort_key: int) -> str:
    """
    Convert a sort key to the filename's little-endian digits.

    The digits are the decimal representation of sort_key, reversed.

    Examples:
        1  -> "1"
        9  -> "9"
        10 -> "01"
        11 -> "11"
        13 -> "31"
        15 -> "51"
        20 -> "02"
        21 -> "12"
    """
    return str(sort_key)[::-1]


def find_foops(foop_dir: Path) -> list[tuple[int, str]]:
    """
    Find all FOOP files in foop_dir.

    Returns a list of (sort_key, filename) pairs sorted by sort_key
    ascending (chronological order). Excludes plan files (.plan.md),
    template, and INDEX.
    """
    found: list[tuple[int, str]] = []
    for entry in foop_dir.iterdir():
        if not entry.is_file():
            continue
        if entry.name.endswith(".plan.md"):
            continue
        if entry.name == "FOOP-template.md":
            continue
        if entry.name == "INDEX.md":
            continue
        k = filename_to_sort_key(entry.name)
        if k is None:
            continue
        found.append((k, entry.name))
    found.sort(key=lambda pair: pair[0])
    return found


def filename_to_identifier(filename: str) -> str:
    """
    Extract the FOOP identifier from a filename.

    The identifier is "FOOP-<digits>" where <digits> are the
    little-endian digits in the filename.

    Examples:
        FOOP-1.md   -> "FOOP-1"
        FOOP-01.md  -> "FOOP-01"
        FOOP-31.md  -> "FOOP-31"
        FOOP-51.md  -> "FOOP-51"
    """
    return filename[: -len(".md")]


def cmd_check(foop_dir: Path) -> int:
    """
    Verify FOOPs are consecutive in chronological order (no gaps).

    Returns 0 on success, 1 if gaps or duplicates are found.
    """
    foops = find_foops(foop_dir)
    if not foops:
        print(f"No FOOPs found in {foop_dir}", file=sys.stderr)
        return 1

    problems: list[str] = []

    seen: dict[int, list[str]] = {}
    for k, fn in foops:
        seen.setdefault(k, []).append(fn)

    for k, names in sorted(seen.items()):
        if len(names) > 1:
            ids = [filename_to_identifier(n) for n in names]
            problems.append(
                f"Duplicate sort key {k}: {', '.join(ids)} ({', '.join(names)})"
            )

    sort_keys = sorted(seen.keys())
    expected = 1
    for k in sort_keys:
        while expected < k:
            expected_filename = f"FOOP-{sort_key_to_filename_digits(expected)}.md"
            expected_id = filename_to_identifier(expected_filename)
            problems.append(
                f"Missing FOOP at sort key {expected}: expected {expected_id} ({expected_filename})"
            )
            expected += 1
        expected = k + 1

    if problems:
        print("FOOP numbering problems:", file=sys.stderr)
        for p in problems:
            print(f"  - {p}", file=sys.stderr)
        print(file=sys.stderr)
        last_id = filename_to_identifier(foops[-1][1])
        print(
            f"Found {len(foops)} FOOP files; most recent is {last_id} (sort key {sort_keys[-1]}).",
            file=sys.stderr,
        )
        return 1

    last_id = filename_to_identifier(foops[-1][1])
    first_id = filename_to_identifier(foops[0][1])
    print(
        f"OK: {len(foops)} FOOPs, consecutive sort keys 1 through {sort_keys[-1]}. "
        f"Range: {first_id} (oldest) through {last_id} (most recent)."
    )
    return 0


def cmd_get_last(foop_dir: Path) -> int:
    """Print the identifier of the most-recently-created FOOP."""
    foops = find_foops(foop_dir)
    if not foops:
        print("No FOOPs found", file=sys.stderr)
        return 1
    k, fn = foops[-1]
    ident = filename_to_identifier(fn)
    print(f"{ident}\t{fn}\t(sort key {k})")
    return 0


def cmd_gen_next(foop_dir: Path) -> int:
    """Print the identifier and filename for the next FOOP to be created."""
    foops = find_foops(foop_dir)
    next_k = (foops[-1][0] + 1) if foops else 1
    digits = sort_key_to_filename_digits(next_k)
    filename = f"FOOP-{digits}.md"
    ident = filename_to_identifier(filename)
    print(f"{ident}\t{filename}\t(sort key {next_k})")
    return 0


def cmd_list(foop_dir: Path) -> int:
    """List all FOOPs in chronological order."""
    foops = find_foops(foop_dir)
    if not foops:
        print("No FOOPs found", file=sys.stderr)
        return 1
    for k, fn in foops:
        ident = filename_to_identifier(fn)
        print(f"{ident}\t{fn}\t(sort key {k})")
    return 0


COMMANDS = {
    "check": cmd_check,
    "get_last": cmd_get_last,
    "gen_next": cmd_gen_next,
    "list": cmd_list,
}


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Check and manage FOOP numbering.",
        epilog="Filenames use little-endian digits; the digits reversed "
               "and read as decimal give the identifier.",
    )
    parser.add_argument(
        "command",
        choices=sorted(COMMANDS.keys()),
        help="What to do.",
    )
    parser.add_argument(
        "--dir",
        type=Path,
        default=FOOP_DIR,
        help=f"FOOP directory (default: {FOOP_DIR}).",
    )
    args = parser.parse_args()

    if not args.dir.is_dir():
        print(f"Not a directory: {args.dir}", file=sys.stderr)
        return 2

    return COMMANDS[args.command](args.dir)


if __name__ == "__main__":
    sys.exit(main())
