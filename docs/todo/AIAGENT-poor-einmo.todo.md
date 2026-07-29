# poor_einmo TODO

## poor_einmo.sh

- [ ] Trap unexpected exits (SIGINT, SIGTERM, ERR, EXIT) and write out:
  - The temporary editing directory path ($TMP / $VIMTMP)
  - Which test was being reviewed when the exit happened
  - How to recover: "To resume, re-run poor_einmo.sh — scratch is cleaned up automatically. If you need to recover unsaved pane edits, check: $TMP"
  - Currently the EXIT trap only does `rm -rf "$TMP"` — it should print the recovery info BEFORE cleanup, or skip cleanup on unexpected signals so the user can inspect the scratch dir.

