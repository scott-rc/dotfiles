# gd Web UI

@README.md

---

Any change to architecture, state shape, component structure, protocol, keyboard mappings, design decisions, or project layout MUST include updates to README.md (and this file if agent rules are affected) in the same commit.

---

E2E tests verify user behavior, not implementation details. Test that pressing `j` moves the cursor, not that a signal incremented. Unit tests cover pure functions in `utils/` thoroughly (grouping, navigation, keyboard mapping, display flattening) -- target 100% line coverage on `utils/`. Prefer fewer comprehensive E2E tests over many shallow component tests.

---

View-only UI -- no staging/unstaging. If a change would require staging support, it belongs in the TUI pager (`src/pager/`), not here.
