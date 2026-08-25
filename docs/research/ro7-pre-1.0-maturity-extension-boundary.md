# RO7 pre-1.0 maturity and extension-boundary audit

Status: **non-normative investigation**

This record supports RO7A issue #44 under parent #43. It does not define RunenOnline semantics, compatibility guarantees, package ownership, sequencing, or work authorization.

Accepted investigation base: `abb989952a619286d72ee6e4de24b19d23c9564b`.

## Decision

RO7 does not currently justify a new online-game feature family, provider adapter, persistence/service/fleet realization, scaling subsystem, release pipeline, version bump, crates.io publication work, or compatibility-analysis tool.

The stage-start issue inventory contains no independently accepted extension demand beyond RO7 itself. The actual missing artifact is a durable owner for RunenOnline's **current pre-1.0 support/maturity/compatibility claim and the evidence required before that claim may be strengthened**.

The minimum RO7 delivery should therefore add one concise top-level `MATURITY.md`, register that responsibility in the documentation architecture, and link it from README orientation.

`MATURITY.md` must **not** become an extension-admission authority. `ROADMAP.md` continues to own sequencing and separately accepted demand; `ARCHITECTURE.md` continues to own package/dependency boundaries; `spec/` continues to own portable semantics; live issue/work authority remains governed by `AGENTS.md` and Dornglut governance. The maturity document may only state the current maturity claim and route changes to those existing owners.

## Current maturity inventory

### Semantic authority

`spec/README.md` identifies the specification as `0.1-provisional`. Under `spec/conventions.md`, provisional normative authority is accepted pre-stability authority that may change only through explicit normative revision. Open specification items remain undefined until an accepted normative revision resolves them.

That is a semantic-authority discipline, not a promise that future explicit normative revisions will be backward compatible.

### Rust and package surface

`runen-online` and `runen-online-oidc` are currently version `0.1.0`. The core crate explicitly describes its public Rust representations as provisional implementation choices rather than wire, storage, provider, or service standards. `AuthorityError` is likewise documented as a provisional implementation diagnostic surface. The OIDC crate similarly keeps its provider and verification details realization-local.

Package version, specification version, semantic authority, and Rust API compatibility are therefore distinct concerns. No version bump is required merely to document the current state.

### Assurance and production evidence

`docs/verification/supported-contracts.md` maps the supported normative surface to deterministic core evidence, the bounded OIDC realization, and the RunenNet/game-server composition proof. It explicitly disclaims a formal conformance profile, interoperability claim, stable Rust API, and compatibility/version policy.

Assurance proves behavior on a reviewed revision. It does not promise compatibility across revisions.

### Existing extension boundaries

`ARCHITECTURE.md` already requires accepted evidence before adding independently reusable packages, provider adapters, integrations, or service/runtime topology. `ROADMAP.md` already requires RO7 capabilities and ecosystem work to arise only through separately accepted demand.

A maturity document must link to those owners rather than restate or replace their detailed rules.

### Release mechanics

The repository currently has the canonical validation workflow and no repository-owned release/publish workflow. The available repository tooling did not provide reliable tag/release enumeration during this audit, so this record makes no claim about whether historical tags or GitHub releases exist.

That uncertainty is not a current gate: no accepted work presently requires publication, release automation, or a cross-release compatibility promise.

## Assurance is not compatibility

The repository now has strong evidence that the current reviewed implementation realizes the current accepted contracts. Compatibility is a separate claim about continuity **between revisions**.

The durable maturity policy should distinguish:

- normative semantic authority — what the current specification requires;
- executable assurance — evidence that one reviewed revision realizes that authority;
- Rust API/package compatibility — whether downstream source use remains compatible across revisions;
- wire/storage compatibility — representation continuity, mostly undefined today;
- provider compatibility — guarantees of a concrete realization, not core semantics;
- release/distribution compatibility — guarantees tied to an explicit version and distribution policy.

None of the latter compatibility claims follows automatically from `cargo validate`, passing tests, package version `0.1.0`, or the `0.1-provisional` specification label.

## Criteria for strengthening maturity or compatibility claims

The current status should remain **provisional pre-1.0**. A stronger claim should require separately accepted work showing, for the exact scope being strengthened:

1. **Defined support boundary** — the claimed semantic surface is owned by indexed normative artifacts; support-critical open items are resolved or explicitly outside the claim.
2. **Current assurance** — the claimed behavior has executable evidence and passes canonical validation on the reviewed head.
3. **Public-surface audit** — the exact Rust/package/provider compatibility surface being protected is identified and implementation accidents are not promoted into portable requirements.
4. **Package/dependency evidence** — independently reusable packages and production adapters have accepted ownership and explicit capability/failure boundaries.
5. **Security/maintenance evidence** — supported production realizations have an appropriate path for material dependency/security maintenance.
6. **Consumer evidence where claimed** — integration claims have executable downstream proof rather than inferred compatibility.
7. **Version/distribution decision when needed** — before promising compatibility across published releases, the project explicitly decides the version scope, breaking-change treatment, and release/distribution mechanics for that claim.

These are gates for strengthening a claim, not a requirement to resolve every future online-service concern before ordinary pre-1.0 development continues.

RO7 does not define a 1.0 readiness checklist. A future 1.0 proposal requires separate accepted authority and evidence.

## Change routing through existing owners

`MATURITY.md` should summarize routing only; it must not authorize work.

| Change | Existing owner / authority path |
| --- | --- |
| Portable provider-neutral behavior | owning `spec/` artifact through explicit normative revision |
| Rust API/type/error refactor with unchanged semantics | bounded live issue/work authority plus current assurance |
| New independently reusable crate or provider adapter | `ARCHITECTURE.md` package/dependency ownership plus separately accepted capability evidence |
| Persistence/service/fleet/concurrent realization | investigate the real mechanism; new portable ordering/failure/authority/replay/resource rules go to `spec/` |
| RunenNet/game-server/Runenwerk integration | independent integration demand plus existing cross-repository and identity/lifecycle boundaries |
| Broader online-game capability | separately accepted demand under `ROADMAP.md`, then semantic ownership investigation as needed |
| Performance/scaling optimization | implementation work if semantics are unchanged; normative investigation if externally observable resource/order/failure semantics change |
| Stronger compatibility/stability claim | `MATURITY.md` states current claim and evidence gates; concrete version/release mechanics require separate accepted work |

## Current demand inventory

At RO7 stage start there were no other open RunenOnline issues representing independently accepted extension demand. Prior research identifies future possibilities but does not create active work for:

- additional authentication providers;
- persistence or service transport;
- server-fleet/allocation providers;
- social/lobby/economy/progression systems;
- Runenwerk integration;
- a shared-concurrent/distributed Authority realization;
- performance/scaling work;
- public grant credentials or remote recovery protocols;
- release automation, package publication, version-policy tooling, or compatibility analyzers.

No RO7 feature child is justified for those areas without new independent demand.

## Documentation ownership decision

A lasting support/maturity/compatibility claim does not fit cleanly in existing owners:

- `spec/` owns semantics;
- `ROADMAP.md` owns sequencing and stage gates;
- `ARCHITECTURE.md` owns package/dependency structure;
- `TESTING.md` owns mechanical validation;
- `docs/verification/` owns assurance evidence;
- README owns orientation.

A single top-level `MATURITY.md` is therefore justified as the owner of the **current support/maturity/compatibility claim and criteria for strengthening that claim**. It does not own semantic revision, package creation, extension demand, work approval, or release mechanics.

`docs/documentation-architecture.md` should register exactly that responsibility. No maturity-document taxonomy is needed.

## Selected RO7 delivery

Create one RO7B slice: **pre-1.0 maturity/support policy**.

### Allowed files

- `MATURITY.md` — current support/maturity/compatibility claim and strengthening criteria;
- `docs/documentation-architecture.md` — register that ownership;
- `README.md` — link the current maturity statement to `MATURITY.md`.

No other file is justified by RO7A.

### Required `MATURITY.md` content

Keep it concise. It should:

1. state the current status as provisional pre-1.0;
2. distinguish normative semantics, executable assurance, Rust API/package compatibility, and release/distribution compatibility;
3. state that `0.1.0` package versions and passing validation do not create an unstated backward-compatibility promise;
4. state the evidence-backed criteria above for strengthening a maturity/compatibility claim;
5. route semantic/API/package/provider/integration/performance changes to existing owners without duplicating their detailed rules;
6. point to `ROADMAP.md` for the separately accepted demand rule rather than owning extension admission;
7. state that no broader RO7 extension is currently selected;
8. state that 1.0, release/publication mechanics, supported-version lifetimes, and a formal compatibility/version scheme require separate future work.

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
- a concrete version/breaking-change scheme for published releases;
- crates.io/release distribution mechanics;
- deprecation windows or supported-version lifetimes;
- stable wire/storage/credential formats;
- provider compatibility guarantees;
- a new feature/provider/scaling project without independent demand;
- a 1.0 readiness or release commitment.

## RO7 closure expectation

If RO7B lands with exact-head and merged-main `cargo validate` success and no new independent demand or critical defect appears, no extension feature is required to satisfy RO7. The parent can then be reviewed for closure with the explicit conclusion that RO7 established maturity/support boundaries while selecting **zero** broader extensions because none had independent demand.
