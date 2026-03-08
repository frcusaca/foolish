# P0 TODO: UBC0_1 Design Migration Using CMfir

## Description

Reverse-engineer the UBC0_1 design specification using CMfir (ConText Manipulation FIR) derived from UBC1's design. The goal is to maintain the operational semantics of UBC1 while applying them to the UBC0_1 computational model.

## Priority

**P0** — Foundational work for UBC0_1 implementation.

## Rationale

UBC0_1 is a reimplementation of UBC0 semantics using UBC1's clarified microstate definitions. Before implementation can begin, we need to:

1. Analyze UBC1's CMfir design (context manipulation, two-phase re-evaluation)
2. Map UBC0 operational semantics to UBC1's state machine
3. Specify how CMfir behaves in the UBC0_1 context
4. Document the migration path in engineering docs (`how/`)

## Tasks

- [ ] Read and analyze UBC1's CMfir implementation in `ubc1/how/ubc_engineering.md`
- [ ] Review UBC0 semantics in `vintage_legacy/` documentation
- [ ] Specify CMfir behavior for UBC0_1 microstates
- [ ] Write engineering reference in `ubc0_1/how/`
- [ ] Define approval test migration strategy

## Related

- UBC1 Engineering: `docs/ubc1/how/ubc_engineering.md` § Context Manipulation: CMFir
- UBC0_1 Overview: `docs/ubc0_1/README.md`
- Legacy UBC0 docs: `docs/vintage_legacy/`

---

## Last Updated

Date: 2026-03-07
Updated By: Claude Code / cyankiwi/Qwen3.5-27B-AWQ-BF16-INT8
Changes: Created P0 TODO for UBC0_1 design migration using CMfir from UBC1.
