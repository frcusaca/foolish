# P2 TODO: Grouping and Operator Precedence

## Description

How `(a + b) * c` groups differently from `a + b * c`.

## Priority

**P2** — Shared parser library task. Not blocking core UBC1 development.

## Rationale

All grouping and precedence operators are treated as effects of brane concatenation (see UBC1 docs). This determination means the precedence rules themselves are not primal to the UBC1 design — they can be addressed later in the shared parser library.

## Related

- See `docs/ubc1/` for brane concatenation design context

---

## Last Updated

Date: 2026-03-07
Updated By: Claude Code / cyankiwi/Qwen3.5-27B-AWQ-BF16-INT8
Changes: Created as P2 TODO item for shared parser library. Determined that grouping/precedence operators are effects of brane concatenation, not primal concerns.
