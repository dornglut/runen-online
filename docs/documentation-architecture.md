# Documentation Architecture

This document owns repository documentation boundaries and dependency direction. It does not define RunenOnline semantics, project priorities, package topology, provider choices, or verification policy.

## Artifact ownership

- `spec/` — normative RunenOnline specification only;
- `ROADMAP.md` — project sequencing and acceptance gates only;
- `ARCHITECTURE.md` — repository package and dependency structure only;
- `TESTING.md` — mechanical repository validation only;
- `docs/architecture/` — non-normative implementation and realization design when needed;
- `docs/verification/` — non-normative assurance and conformance strategy when needed;
- `docs/decisions/` — historical repository-local design decisions and rationale when needed;
- `docs/research/` — external research and provider/integration evidence when needed;
- `CONTRIBUTING.md` — contributor process when present;
- `AGENTS.md` — automation-specific contributor constraints;
- `README.md` and directory README files — navigation and orientation.

## Dependency direction

Normative specification artifacts may reference other normative specification artifacts. They MUST NOT depend on roadmap, repository implementation, verification documents, design decisions, research notes, provider behavior, RunenNet implementation, Runenwerk implementation, or contributor workflow for their meaning.

Non-normative documents may reference normative specification owners. Roadmap and contributor documents may reference any artifact needed to identify work, but they do not acquire authority over the referenced concern.

## No duplicated authority

Each rule has one canonical owner. Documents may summarize their own scope and link to another owner, but SHOULD NOT duplicate another owner's detailed rules.

If a concept change requires editing multiple documents that each claim to define the same rule, the documentation decomposition is defective and ownership must be corrected.

## Provider and implementation evidence

Existing backend products, database schemas, service APIs, RunenNet behavior, Runenwerk behavior, game-server implementations, tests, or historical documents MAY be used as research and realization evidence. They MUST NOT become normative RunenOnline semantics merely because they already exist or are convenient to integrate.

## Growth rule

Split an artifact when responsibilities can evolve independently under different correctness or review obligations. Do not pre-create taxonomy, service-shaped documentation, or package-shaped documentation solely in anticipation of future implementation.
