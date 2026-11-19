# samcad — Documentation Index

> Status: **Draft** · Last updated: 2026-11-19

This folder holds the product and engineering requirements for **samcad**, a lightweight 2D CAD application.

## Documents

| Document | Purpose |
| --- | --- |
| [project-overview.md](project-overview.md) | What samcad is, who it is for, goals and non-goals |
| [technical-stack.md](technical-stack.md) | Stack research, recommendation, alternatives and risks |
| [functional-requirements.md](functional-requirements.md) | Prioritized user-facing features with acceptance criteria |
| [non-functional-requirements.md](non-functional-requirements.md) | Performance, portability, reliability and licensing constraints |
| [architecture.md](architecture.md) | Proposed code structure, data model and pipelines |
| [decisions.md](decisions.md) | Architecture decision records (ADRs) |
| [roadmap.md](roadmap.md) | Phased delivery plan and exit criteria |

## How to use this folder

- Product/behavior questions → `project-overview.md` and `functional-requirements.md`.
- Implementation choices → `technical-stack.md` and `decisions.md`.
- "Where does this code go?" → `architecture.md`.
- "What do we build next?" → `roadmap.md`.

Requirements are expected to change as we build. Update the relevant document in the same commit that changes the implementation so the docs stay authoritative.
