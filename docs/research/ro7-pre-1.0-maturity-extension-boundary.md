# RO7 pre-1.0 maturity and extension-boundary audit

Status: **non-normative investigation**

This record supports RO7A issue #44 under parent #43. It does not define RunenOnline semantics, compatibility guarantees, package ownership, or work authorization.

Accepted investigation base: `abb989952a619286d72ee6e4de24b19d23c9564b`.

## Decision

RO7 does not currently justify a new online-game feature family, provider adapter, persistence/service/fleet realization, scaling subsystem, release pipeline, version bump, crates.io publication work, or compatibility-analysis tool.

The stage-start issue inventory contains no independently accepted extension demand beyond RO7 itself. The missing maturity artifact is instead a durable owner for RunenOnline's **current pre-1.0 support/compatibility claim and the evidence required before that claim may be strengthened**.

The minimum RO7 delivery should therefore add one concise top-level `MATURITY.md`, register its ownership in the documentation architecture, and link it from README orientation. It should route future changes to existing semantic, architecture, roadmap, verification, and live-work owners rather than create a second governance system.

## Current maturity inventory

### Semantic authority

`spec/README.md` identifies the specification as `0.1-provisional`. Under `spec/conventions.md`, provisional normative authority is accepted pre-stability authority that may change only through explicit normative revision. Open specification items remain explicitly undefined and cannot be filled implicitly by implementation/provider behavior.

This is already a meaningful semantic-authority discipline. It is **not** a promise that future explicit normative revisions will be backward compatible.

### Rust/package surface

`runen-online` and `runen-online-oidc` are currently version `0.1.0`. The core crate documents its Rust representations as provisional implementation choices rather than wire/storage/provider/service standards; the public implementation error taxonomy is likewise provisional.

The package version therefore must not be treated as equivalent to specification version or as an unstated compatibility guarantee.

No version bump is required merely to document this existing state.

### Assurance and production evidence

`docs/verification/supported-contracts.md` maps the supported normative surface to deterministic core evidence, the bounded OIDC production realization, and the RunenNet/game-server composition proof. It explicitly disclaims a formal conformance profile, interoperability claim, stable Rust API, and compatibility/version policy.

RO6 closure confirms the applicable hostile-input, replay/stale, expiry/currentness, authorization, resource-bound, serial race-order, OIDC failure, and composition boundaries have evidence on the current supported surface.

Assurance demonstrates behavior on reviewed revisions; it does not by itself promise compatibility across revisions.

### Package and extension boundaries

`ARCHITECTURE.md` already requires independent accepted evidence for additional package/provider/integration ownership and preserves the provider-neutral core from RunenNet, Runenwerk, engine/ECS, database/backend, fleet, and service-transport dependencies.

`ROADMAP.md` already requires broader RO7 capabilities and ecosystem work to arise only through separately accepted demand.

Those owners should remain authoritative. A maturity document should link to them, not restate their detailed package or sequencing rules.

### Release mechanics

The repository currently contains only the canonical validation workflow under `.github/workflows/`; there is no repository-owned release/publish workflow.

The available GitHub connector did not expose reliable tag/release enumeration during this audit, and the public release page could not be fetched reliably. This investigation therefore makes **no claim** that tags or GitHub releases do or do not exist.

That uncertainty does not block the RO7 gate: no accepted work currently requires a release, crates.io publication, or automated distribution path. Release mechanics become a maturity concern when a stronger compatibility/distribution claim is actually proposed.

## Assurance is not compatibility

The repository now has strong evidence that the current reviewed implementation realizes the current accepted contracts. Compatibility is a different claim about behavior or interfaces **between revisions**.

RO7 should make that distinction durable:

- normative assurance says a reviewed revision preserves accepted semantic owners;
- Rust API compatibility says downstream source code continues to compile/use interfaces across revisions;
- wire/storage compatibility would concern representations that are currently mostly undefined;
- provider compatibility concerns a concrete realization and cannot redefine core semantics;
- package/release compatibility depends on an actual distribution/version policy.

None of the latter four follows automatically from `cargo validate`, passing tests, `0.1.0`, or the `0.1-provisional` specification label.

## Proposed pre-1.0 maturity criteria

The current status should remain **provisional pre-1.0**. A stronger support/compatibility claim should require explicit accepted work showing, for the scope being strengthened:

1. **Defined support boundary** — the claimed semantic surface is indexed by normative owners; support-critical open items are either explicitly outside the claim or resolved by accepted normative revision.
2. **Current assurance** — `docs/verification/supported-contracts.md` and canonical validation cover the claimed behavior on the reviewed head.
3. **Public-surface audit** — implementation-local Rust representations/errors/provider details are not being accidentally promoted into portable requirements, and the exact compatibility surface to protect is identified.
4. **Package/dependency evidence** — independently reusable packages and production adapters have accepted ownership and failure/capability boundaries.
5. **Security/maintenance evidence** — supported production realizations have an explicit maintenance path for material dependency/security issues appropriate to their implemented capabilities.
6. **Consumer evidence where claimed** — integration claims have executable downstream proof rather than inferred compatibility.
7. **Version/distribution decision when needed** — before promising compatibility across published releases, the project explicitly decides version scope, breaking-change treatment, and release/distribution mechanics for that claim.

These are gates for **strengthening a claim**, not a requirement to resolve every open online-service problem before ordinary pre-1.0 development can continue.

RO7 does not define a 1.0 readiness checklist. A future 1.0 proposal requires its own accepted authority and evidence.

## Change and extension routing

The durable maturity policy should route changes without replacing existing owners:

| Change | Existing authority path |
| --- | --- |
| Portable provider-neutral behavior changes | investigate as needed, then explicit revision of the owning `spec/` artifact before implementation depends on the new rule |
| Rust API/type/error refactor with unchanged semantics | bounded implementation work under live issue authority; preserve documented implementation-local status and executable assurance |
| New independently reusable crate/provider adapter | accepted `ARCHITECTURE.md` package/dependency boundary plus evidence for the concrete capability; normative revision only if portable semantics change |
| Persistence/service/fleet/concurrent realization | investigate the real mechanism; route any new ordering, failure, authority, replay, or resource semantics to `spec/`; otherwise keep realization details non-normative |
| RunenNet/game-server/Runenwerk integration | preserve existing cross-repository and identity/lifecycle boundaries; require independent integration demand and explicit mapping evidence |
| Broader online-game capability | independent demand first; investigate semantic ownership; add normative owner only for rules actually accepted |
| Performance/scaling optimization | implementation work if externally observable semantics are unchanged; normative investigation if resource/order/failure semantics would change |
| Compatibility/stability claim | `MATURITY.md` owns the current claim and criteria for strengthening it; concrete release/version mechanics require separately accepted work when relevant |

Live issue/work authority remains governed by existing Dornglut governance and `AGENTS.md`. The maturity document must not become a task queue or approval substitute.

## Current demand inventory

At the RO7 stage start there were no other open RunenOnline issues. Accepted prior research establishes several **future possibilities** but does not itself create active demand for:

- additional authentication providers;
- persistence or service transport;
- server-fleet/allocation providers;
- social/lobby/economy/progression systems;
- a Runenwerk integration;
- a shared-concurrent/distributed Authority implementation;
- performance/scaling work;
- public grant credentials or remote recovery protocols;
- release automation, package publication, SemVer tooling, or compatibility analyzers.

No RO7 feature child should be created for any of those areas without a new independently justified issue/investigation.

## Documentation ownership decision

A lasting maturity/compatibility claim cannot live cleanly in existing owners:

- `spec/` owns semantics, not cross-revision Rust/package compatibility;
- `ROADMAP.md` owns sequencing/gates, not ongoing support policy;
- `ARCHITECTURE.md` owns package/dependency structure, not compatibility claims;
- `TESTING.md` owns mechanical validation;
- `docs/verification/` owns evidence, not promises across revisions;
- README is orientation only.

A single top-level `MATURITY.md` is therefore justified as an independent repository policy owner. `docs/documentation-architecture.md` should register exactly that responsibility, and README should link to it. No maturity-doc taxonomy is needed.

## Selected RO7 delivery

Create one RO7B slice: **pre-1.0 maturity and extension policy**.

### Allowed files

- `MATURITY.md` — new current support/stability/extension-admission policy;
- `docs/documentation-architecture.md` — register `MATURITY.md` ownership;
- `README.md` — link the current maturity statement to `MATURITY.md`.

No other file is justified by RO7A.

### Required `MATURITY.md` content

Keep it concise. It should:

1. state the current status as provisional pre-1.0;
2. distinguish provisional normative semantics, current executable assurance, Rust API/package compatibility, and release/distribution compatibility;
3. state that current `0.1.0` package versions and passing validation do not create an unstated backward-compatibility promise;
4. give the evidence-backed criteria above for strengthening a compatibility/maturity claim;
5. route semantic/API/package/provider/integration/performance changes to existing owners without duplicating their detailed rules;
6. require independent demand before broader extensions;
7. explicitly state that no broader RO7 extension is currently selected;
8. state that 1.0, release automation/publication, and a formal compatibility/version scheme require separately accepted future work.

### Forbidden in RO7B

Do not change:

- normative `spec/`;
- product or adapter source/tests;
- `ARCHITECTURE.md`, `ROADMAP.md`, or `TESTING.md`;
- manifests, lockfile, package versions, release/tag state, or workflows;
- verification evidence content;
- RunenNet or Runenwerk.

## Stop conditions

Stop and route to separate investigation/authority if delivery would need to decide:

- a new normative semantic compatibility rule;
- a concrete SemVer/breaking-change scheme for published releases;
- crates.io/release distribution mechanics;
- deprecation windows or supported-version lifetimes;
- stable wire/storage/credential formats;
- provider compatibility guarantees;
- a new extension feature/provider/scaling project without independent demand;
- a 1.0 readiness or release commitment.

## RO7 closure expectation

If RO7B lands cleanly with exact-head and merged-main `cargo validate`, no extension feature is currently required to satisfy RO7. The parent can be reviewed for closure with the explicit conclusion that the stage established maturity and extension-admission boundaries while selecting **zero** broader extensions because none had independent demand.
