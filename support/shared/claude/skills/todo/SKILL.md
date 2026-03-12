---
name: todo
description: Manages persistent, file-based todo lists in docs/todo/*.todo.md files. Use this skill whenever the user wants to add, complete, list, reprioritize, or work on tasks — including "add a todo", "what's next", "mark that done", "show my tasks", "move that up", "reprioritize", "what should I work on", or any reference to a project task list. The docs/todo/ directory and all *.todo.md files are exclusively owned and maintained by this skill.
---

# TODO v2.0: Task Management Skill

Manages persistent todo lists in `docs/todo/*.todo.md`. The `docs/todo/` directory and every `*.todo.md` file within it are **exclusively maintained by this skill** — never create, edit, or delete these files by any other means.

**Skill version**: v2.0

---

## Part 1: File Format

### Version compatibility

Each todo file declares its format version at the top. The skill must check version compatibility before operating on a file.

```markdown
# TODO v2.0: <File Title>
```

- **Skill v2.0** requires files with `# TODO v2.0:` header
- If a file has a different version (e.g., `# TODO v1.0`), the skill must notify the user and offer to migrate or skip
- All new files created by this skill use v2.0 format

### Location and naming

All todo files live in `docs/todo/` at the project root.

**Default session files** are named after the Claude session ID:
`docs/todo/claude-<session-id>.todo.md`

The session ID is available in Claude Code as the environment variable `$CLAUDE_SESSION_ID`. Use it literally — do not abbreviate or transform it.

**Project-specific todo files** may have any name the user chooses, as long as they live in `docs/todo/` and end in `.todo.md` (e.g., `docs/todo/sprint-3.todo.md`, `docs/todo/auth-refactor.todo.md`).

### Active file resolution

At the start of each session, the **active file** is determined as follows:

1. If the user specifies a file explicitly (by name or path), use that file for the session
2. Otherwise, use `docs/todo/claude-<session-id>.todo.md` as the default

The active file persists for the whole session unless the user explicitly switches with a command like "switch to `sprint-3`" or "use the auth todo file". On switch, note the change in the Log of both files.

When a command doesn't specify a file, always use the active file for the session.

### Structure

Each file has three top-level sections: `# TODO vX.X: Title`, `## Open`, and `## Completed`, followed by `## Log`.

```markdown
# TODO v2.0: Session Tasks

## Open

### [ ] (0)

First task to do.

### [ ] (1)

Second task to do. This one has a multi-line description.
More detail here.

### [ ] (2)

Third task to do.

## Completed

### [x] (a)

Earlier completed task.

## Log

### (a)
*2025-03-08 10:14* completed

Task was finished successfully.

### session started
*2025-03-08 09:00* planning

Working on initial setup.
```

### Section organization

| Section | Purpose | Marker |
|---------|---------|--------|
| `# TODO vX.X: Title` | File header with version and title | `#` |
| `## Open` | Tasks not yet done | `##` |
| `## Completed` | Tasks marked done/abandoned/canceled | `##` |
| `## Log` | Append-only event history | `##` |

### Task subsections

Each task is a subsection under Open or Completed:

- **Heading**: `### [ ] (id)` for open, `### [x] (id)` for completed (or `[?]` or `[-]`)
- **Body**: Description goes in the body of the section (after the heading line)
- **Separation**: Always have a blank line after the last line of each task subsection's body

Example:
```markdown
### [ ] (0)

First task description. This can span multiple lines.

### [ ] (1)

Second task description.
```

### Priority ordering

- **Open section**: Tasks are in priority order — top = next up
- **Completed section**: Tasks are in completion order — most recent at top
- Tasks are **moved** from Open to Completed when closed (not copied)

### Markers

| Marker | Section     | Meaning |
|--------|-------------|---------|
| `[ ]`  | Open        | Not yet done |
| `[x]`  | Completed   | Completed successfully |
| `[?]`  | Completed   | Abandoned — insufficient documentation or task didn't make sense |
| `[-]`  | Completed   | Canceled — no longer needed |

Never use any other marker. Markers appear in task subsection headings.

### Task subsection rules

- Each task is a `###` subsection under Open or Completed
- **Heading format**: `### [ ] (id)` for open, `### [x] (id)` for completed
- **Body format**: Description goes in the body (lines after the heading)
- **Separation**: Always have a blank line after the last line of each task's body
- Tasks in Open are in **priority order** — top = next up
- Tasks in Completed are in **completion order** — most recent at top

### Log subsection rules

- `## Log` contains one `###` entry per event — creation, closure, reprioritization, file switch, or any other mutation
- Each entry heading: `### (id) <description>` for task-specific events, or `### <short label>` for file-level events
- Immediately below the heading: `*YYYY-MM-DD HH:MM* <event type>` on its own line
- Below that: optional free-form content — comments, reasons, context
- Log entries are in **reverse chronological order** — most recent at the top (just after `## Log`)
- **Never reorder or edit existing Log entries** — the Log is append-only (prepend-only in the file)
- Every mutation command must write a Log entry — no silent changes

---

## Part 2: Todo IDs

Every task gets a **permanent unique ID** assigned at creation and carried unchanged for its entire lifetime.

### Format

IDs are base-36 integers using digits `0–9` then letters `a–z`, always written in parentheses: `(0)`, `(9)`, `(a)`, `(z)`, `(10)`, `(1a)`, `(zz)`.

Counting sequence: `0, 1, 2, ..., 9, a, b, ..., z, 10, 11, ..., 19, 1a, 1b, ..., 1z, 20, ...`

The parenthesized form makes IDs grep-friendly and visually distinct in a text editor.

### Counter

The **next available ID** is tracked internally by counting existing IDs (in both Open and Completed sections) and computing the next value.

- A new file starts with counter `0`
- On `/todo-add`: assign the current counter value as the new task's ID, then increment the counter
- Counter **never decrements** — closing, canceling, or abandoning a task does not affect it
- IDs are never reused

### Base-36 increment algorithm

To increment a base-36 string, work right to left:
1. If the rightmost digit is not `z`, increment it (`0→1`, `9→a`, `a→b`, `y→z`) and stop
2. If it is `z`, set it to `0` and carry left
3. If all digits are `z`, prepend `1` (e.g. `z→10`, `zz→100`)

Examples: `9→a`, `z→10`, `1z→20`, `zz→100`

### ID placement

- Task subsections: `### [ ] (id) <description>` — ID immediately after the checkbox
- Log subsections for task events: `### (id) <description>` — ID immediately after `###`
- Log subsections for file-level events (session start/end, file switch): no ID prefix

---

## Part 3: Commands

All commands share these conventions:
- Tasks can be matched by parenthesized ID `(1a)`, description substring, or 1-based position in the Open list
- If a match is ambiguous, list candidates and ask the user to clarify
- Every mutation reads the file fresh, writes the full file, and prepends a Log entry
- `-- <comment>` separates a required closing comment from the task identifier; if omitted when required, Claude asks for it
- If no file is specified, use the active file for the session

---

### `/todo-list [file]`

Display the current state of the active (or specified) todo file.

Show the Open list with IDs and positions. Then show a compact summary of the Log (heading + timestamp line only, no body). If the file doesn't exist, say so and offer to create it.

---

### `/todo-use <filename>`

Switch the active todo file for this session.

1. Resolve the filename: if it doesn't contain a path, prepend `docs/todo/`; if it lacks `.todo.md`, append it
2. Check version compatibility: read the file header and verify it starts with `# TODO v2.0:`
3. If the file doesn't exist, ask the user whether to create it
4. Prepend a Log subsection in the **previous** active file:
   ```
   ### switched away
   *YYYY-MM-DD HH:MM* file switch

   Switched to: docs/todo/<new-file>
   ```
5. Prepend a Log subsection in the **new** active file (create it if needed):
   ```
   ### switched here
   *YYYY-MM-DD HH:MM* file switch

   Switched from: docs/todo/<previous-file>
   ```
6. Confirm to the user which file is now active

---

### `/todo-add <description> [file]`

Add a new open task. Description may be multi-line.

1. Read file (create `docs/todo/` dir and file with v2.0 header if missing):
   ```markdown
   # TODO v2.0: <Session or Project Name>

   ## Open

   ## Completed

   ## Log
   ```
2. Find the next ID by counting existing IDs (in both Open and Completed), compute next value
3. Append to bottom of Open section (description in body, blank line after):
   ```markdown
   ### [ ] (n)

   <description>
   <continuation lines if multi-line>

   ```
4. Prepend Log subsection:
   ```markdown
   ### (n)
   *YYYY-MM-DD HH:MM* created

   <description>
   <continuation lines if multi-line>

   ```
5. Write full file; confirm with the assigned ID

---

### `/todo-done <task> -- <comment> [file]`

Mark a task as completed (`[x]`), move from Open to Completed, log it.

1. Find the matching open task
2. Remove the full subsection (all lines + trailing blank line) from Open
3. Change marker to `[x]` and prepend to top of Completed section:
   ```markdown
   ### [x] (id)

   <description>
   <continuation lines if any>

   ```
4. Prepend Log subsection:
   ```markdown
   ### (id)
   *YYYY-MM-DD HH:MM* completed

   <comment>

   ```
5. Write full file; confirm

`-- <comment>` required. If omitted, ask: "What was accomplished?"

---

### `/todo-abandon <task> -- <comment> [file]`

Mark a task as abandoned (`[?]`) — blocked by missing docs or unclear requirements.

Same as `/todo-done` but uses `[?]` marker and event label `abandoned`.

`-- <comment>` required. If omitted, ask: "What was unclear or missing?"

---

### `/todo-cancel <task> -- <comment> [file]`

Mark a task as canceled (`[-]`) — no longer needed.

Same as `/todo-done` but uses `[-]` marker and event label `canceled`.

`-- <comment>` required. If omitted, ask: "Why is this being canceled?"

---

### `/todo-next [file]`

Show the first task subsection in the Open section in full (ID, description, continuation lines). Ask: "Ready to start this? I can begin working on it now."

If Open is empty, say so and offer to add a task.

---

### Reprioritization commands

All reprioritization commands reorder subsections in the Open section only — the Log is never reordered. Each writes a Log subsection describing the move.

Log subsection format for moves:
```markdown
### (id) <description>
*YYYY-MM-DD HH:MM* reprioritized — <human-readable description of the move>

<optional comment if provided>

```

#### `/todo-up <task> [-- <comment>] [file]`
Move the task one position up (swap with the task above). No-op with notification if already at top.

#### `/todo-down <task> [-- <comment>] [file]`
Move the task one position down (swap with the task below). No-op with notification if already at bottom.

#### `/todo-top <task> [-- <comment>] [file]`
Move the task to position 1 (top of the Open section).

#### `/todo-bottom <task> [-- <comment>] [file]`
Move the task to the last position in the Open section.

#### `/todo-before <task> <target-id> [-- <comment>] [file]`
Move `<task>` to the position immediately before `<target-id>`.

#### `/todo-after <task> <target-id> [-- <comment>] [file]`
Move `<task>` to the position immediately after `<target-id>`.

---

## Part 4: Session Workflow

When the user asks Claude to plan or execute a multi-step task, the todo file is Claude's **external working memory** — keeping it synchronized makes progress visible, recoverable across sessions, and auditable in the Log.

### Session start

At the beginning of each session:

1. Determine the active file: `docs/todo/claude-<session-id>.todo.md` unless the user specifies otherwise
2. Create the file if it doesn't exist (with v2.0 header)
3. Check version compatibility: verify the file starts with `# TODO v2.0:`
4. Read it fresh
5. If the user has stated a task or goal, add todo items for each planned step before starting
6. Prepend a session-start Log subsection:
   ```markdown
   ### session started
   *YYYY-MM-DD HH:MM* planning

   Working on: <brief description>
   Plan: items (id1), (id2), (id3)
   ```

### During execution

- **Before starting a step**: prepend an `in progress` Log entry for that item
- **After completing a step**: close it with `/todo-done`, `/todo-abandon`, or `/todo-cancel` with a meaningful summary
- **If a step is blocked or reveals new work**: add new todo items immediately and log the discovery
- **If priorities shift**: use the reprioritization commands and log why

### Step summary content

Closing comments should be concise but meaningful — not just "done":
- What was actually done or changed
- Decisions made or trade-offs taken
- Anything the next session should know

Good: `-- rewrote connection pool using context manager; dropped max_connections from 20 to 10 after load testing`
Poor: `-- completed`

### Session end

When wrapping up or pausing, prepend a closing Log subsection:
```markdown
### session ended
*YYYY-MM-DD HH:MM* summary
Completed: (id1), (id3)
Remaining: (id2), (id4)
<brief note on where things stand and what is next>
```

### Relationship to Claude's built-in task system

- The **todo file is the canonical record** — not Claude's built-in `TodoWrite`/`TodoRead` system
- The built-in system may be used transiently within a single tool call sequence, but all state must be reflected in the todo file before the response is returned to the user

---

## Part 5: Sync and Edge Cases

### Sync discipline

- **Always read the file fresh** before any operation — never use a cached version
- **Always write the full file** after any mutation — not just the changed lines
- **Never silently fail** — surface errors to the user
- The file on disk is always authoritative — if the user edited it in their editor, their version wins

### Edge cases

- `docs/todo/` doesn't exist → create it along with the file on first use
- File has no `# TODO vX.X:` header → add one before inserting (use v2.0)
- File has no `## Open` section → add one before inserting
- File has no `## Completed` section → add one before inserting completed tasks
- File has no `## Log` section → add one before writing a log entry
- User specifies filename without `.todo.md` suffix → append it automatically
- User specifies filename without path → prepend `docs/todo/`
- `/todo-up` at top or `/todo-down` at bottom → inform user, write no Log entry
- `/todo-before` or `/todo-after` where target equals task → inform user, no-op
- Multi-line task: description goes in body of the subsection
- When removing a task subsection, remove its trailing blank line too — do not leave double blank lines
