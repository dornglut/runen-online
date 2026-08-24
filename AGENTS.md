# Agent Instructions

Automated contributors must follow the repository documentation authority defined in [docs/documentation-architecture.md](docs/documentation-architecture.md).

Before editing, inspect the canonical owner of the concern and its direct normative dependencies. The cross-repository RunenNet/RunenOnline boundary is owned by accepted Dornglut Engineering ADR 0006; repository-local semantics remain owned here.

For iterative continuation, follow Dornglut's [Authority and work](https://github.com/dornglut/engineering/blob/main/governance/authority-and-work.md) rules. Re-establish current repository state and live issue authority before selecting work. An open specification item is not permission to invent semantics or implementation.

Do not create compatibility aliases, duplicate authorities, speculative crate/service splits, RunenNet or Runenwerk semantic dependencies, provider-owned control-plane semantics, or implementation authority from database/backend/service behavior unless accepted repository authority explicitly requires them.

RunenOnline's provider-neutral semantic core MUST remain independently definable without RunenNet, Runenwerk, a concrete ECS/engine runtime, a concrete database/backend provider, a server-fleet provider, or a concrete service transport.

Before proposing acceptance, run the canonical validation defined by [TESTING.md](TESTING.md) and review the exact changed head.
