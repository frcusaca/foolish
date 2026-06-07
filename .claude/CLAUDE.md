# CLAUDE.md — Claude Code-Specific Configuration

> **All project-wide agent guidance lives in `AGENTS.md` at the repo root.**
> This file contains only Claude Code-specific configuration that other agents do not need.
> When in doubt, `AGENTS.md` takes precedence.

---

## Git (Claude Code-Specific)

### Branch Naming

Branches must follow: `claude/<descriptive-name>-<session-id>`

Example: `claude/run-tests-8vk4v`

**CRITICAL**: Branch must start with `claude/` and end with the matching session ID, otherwise push will fail with a 403 HTTP error.

### Push Guidelines

- Always use: `git push -u origin <branch-name>`
- If push fails with network errors, retry up to 4 times with exponential backoff (2s, 4s, 8s, 16s).

---

## Maintenance Instructions

**Weekly Check**: After one week past the last update date below, review this file for accuracy:

1. Verify Claude Code-specific content is still accurate.
2. Confirm this file focuses ONLY on Claude Code-specific features — everything else belongs in `AGENTS.md`.
3. Update the Last Updated section below even if no changes are made.

---

## Last Updated

**Date**: 2026-06-07
**Updated By**: Claude Code 2.1.119 (Claude Code); Sonnet 4.6
**Changes**: Merged CLAUDE.md with AGENTS.md. Removed all duplicate content (now lives in
AGENTS.md only): FOOP conventions, documentation structure, markdown update protocol,
clarifications, Rust code style, task management, development rules, UBC terminology,
commit message format, and build commands. Also removed all Java/Maven/Scala/ANTLR content
(Java implementation has been retired). CLAUDE.md now contains only the Claude Code branch
naming and push guidelines. Added prominent pointer to AGENTS.md as the primary reference.
