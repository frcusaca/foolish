# Phase 4 — Web Brane Browser (LOD)

> Goal: A browser-based viewer for Foolish brane trees, with level-of-detail
> windowed viewports. The screen is finite; branes can have thousands of
> statements. Multiple viewports navigate via the search operators from Phase 1+2.

---

## Phase 4 Deliverable

A web app that:

1. Accepts Foolish source via `POST /eval` and runs it through Phase 1 + Phase 2.
2. Returns the resulting brane tree as JSON.
3. Renders one or more viewports, each showing a *window* of consecutive statements
   into a brane.
4. Click a nested brane → open a new viewport on it.
5. Search box drives navigation: jump by name, regex, head/tail, or index.

---

## API Sketch

| Endpoint | Purpose |
|----------|---------|
| `POST /eval` | Compile + evaluate Foolish source; return brane tree as JSON |
| `GET /brane/:id/window?from=N&size=M` | Windowed view of statements N..N+M of a brane |
| `GET /brane/:id/search?q=pat&dir=backward` | Search within a brane; return matching statement index |

### Window JSON shape

```json
{
  "braneId": "abc123",
  "totalStatements": 1200,
  "window": { "from": 40, "size": 20 },
  "statements": [
    {"index": 40, "name": "x", "state": "Constant", "value": 42},
    {"index": 41, "name": "y", "state": "Constanic"},
    {"index": 42, "name": null, "state": "Constant",
     "value": {"type": "brane", "id": "def456", "size": 5}}
  ]
}
```

---

## Stack

- **HTTP server**: http4s + circe (pure Scala, lightweight, integrates trivially
  with the existing FIR Circe codecs).
- **Frontend**: thin HTML+JS or htmx — no need for a heavy SPA. The brane viewer
  is fundamentally a paged list with click-to-expand semantics.
- **State**: server-side in-memory map of `braneId -> Fir`; refreshes on each
  `POST /eval`.

---

## Phase 4 Exit Criteria

- Viewer renders a 1000-statement brane without browser jank (lazy load windows).
- Multiple simultaneous viewports work (each independently scrollable / searchable).
- Constanic statements visually distinct (`🧠??`).
- Click-to-open-nested-brane works without page reload.

---

## Last Updated

**Date**: 2026-05-01
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.7 (1M Context)
**Changes**: Initial Phase 4 outline — LOD brane browser with windowed viewports.
