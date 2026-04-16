# Create Plan

Break a design document (PRD or refactor RFC) into a phased implementation plan using vertical slices (tracer bullets). Output is a Markdown file placed next to the source design document.

## Process

### 1. Confirm the design document is in context

The design document (PRD or RFC) should already be in the conversation. If it isn't, ask the user to paste it or point you to the file.

Accepted inputs:
- **PRD** — typically at `./tmp/prd/<name>/prd.md`
- **Refactor RFC** — typically at `./tmp/refactor/<name>/rfc.md`
- Any other design document the user points to

### 2. Explore the codebase

If you have not already explored the codebase, do so to understand the current architecture, existing patterns, and integration layers.

### 3. Identify durable architectural decisions

Before slicing, identify high-level decisions that are unlikely to change throughout implementation:

- Route structures / URL patterns
- Database schema shape
- Key data models
- Authentication / authorization approach
- Third-party service boundaries
- **For refactors specifically**: module boundaries, port interface contracts, adapter shapes, dependency categories (in-process, local-substitutable, ports-and-adapters, mock-boundary)

These go in the plan header so every phase can reference them.

### 4. Draft vertical slices

Break the design doc into **tracer bullet** phases. Each phase is a thin vertical slice that cuts through ALL integration layers end-to-end, NOT a horizontal slice of one layer.

<vertical-slice-rules>
- Each slice delivers a narrow but COMPLETE path through every layer (schema, API, UI, tests)
- A completed slice is demoable or verifiable on its own
- Prefer many thin slices over few thick ones
- Do NOT include specific file names, function names, or implementation details that are likely to change as later phases are built
- DO include durable decisions: route paths, schema shapes, data model names, port interfaces
</vertical-slice-rules>

### 5. Quiz the user

Present the proposed breakdown as a numbered list. For each phase show:

- **Title**: short descriptive name
- **Covered**: which user stories (from a PRD) or which parts of the refactor (from an RFC) this addresses

Ask the user:

- Does the granularity feel right? (too coarse / too fine)
- Should any phases be merged or split further?

Iterate until the user approves the breakdown.

### 6. Write the plan file

Output location mirrors the source design document's directory:

- PRD at `./tmp/prd/<name>/prd.md` → plan at `./tmp/prd/<name>/plan.md`
- RFC at `./tmp/refactor/<name>/rfc.md` → plan at `./tmp/refactor/<name>/plan.md`
- Other input → ask the user where to save

Use the template below.

<plan-template>
# Plan: <Name>

> Source: <path to prd.md or rfc.md>

## Architectural decisions

Durable decisions that apply across all phases:

- **Routes**: ...
- **Schema**: ...
- **Key models**: ...
- **Module boundaries / ports** (for refactors): ...
- (add/remove sections as appropriate)

---

## Phase 1: <Title>

**Covered**: <user stories from PRD, or refactor scope from RFC>

### What to build

A concise description of this vertical slice. Describe the end-to-end behavior, not layer-by-layer implementation.

### Acceptance criteria

- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Criterion 3

---

## Phase 2: <Title>

**Covered**: ...

### What to build

...

### Acceptance criteria

- [ ] ...

<!-- Repeat for each phase -->
</plan-template>
