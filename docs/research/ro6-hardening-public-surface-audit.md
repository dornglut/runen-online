# RO6 hardening and public-surface audit

Status: **non-normative investigation**

This record supports RO6A issue #39 under parent #38. It does not define RunenOnline semantics. The accepted `spec/` tree remains the sole authority for portable behavior.

Accepted investigation base: `f0bc653053e3a16423c4e458121c6c2b9f4e6933`.

## Decision

RO6 does **not** currently justify new runtime architecture, persistence, service APIs, observability events, concurrent containers, provider features, or public wire types.

The supported core and OIDC realization already have strong deterministic assurance. The material remaining gap is smaller:

1. RunenOnline has no durable non-normative verification map from supported normative contracts to executable evidence even though `TESTING.md` and the documentation architecture reserve `docs/verification/` for that purpose.
2. Existing tests prove mismatched time-domain rejection when establishing a Pending Assignment, but do not directly prove that a wrong comparison-domain observation cannot mutate **already-established** Assignment, AdmissionGrant, or MatchRequest state.

The minimum RO6 delivery should therefore be one compact assurance document plus focused trust-boundary edge assertions in the existing test suite. No product-source change is justified by this audit.

## Coverage matrix

| Supported area | Current evidence | RO6 classification |
| --- | --- | --- |
| Authority-domain identity and external principals | Domain-scoped non-equivalence, non-reuse/exhaustion, trusted-authority bounds, principal uniqueness/conflict, association quotas, stale-ID isolation | Covered; map evidence only |
| Trusted time and Assignment | Exact deadline boundary, overflow-safe lifetime, time-domain rejection, irreversible expiry, stale resolution isolation, End-vs-resolve orderings | Covered; add explicit no-mutation evidence for wrong-domain observations |
| AdmissionGrant | Exact binding, Usable-currentness, one successful redemption, replay result, Assignment-end invalidation, irreversible expiry, live fan-out reconciliation and retained-capacity bounds | Covered; add explicit no-mutation evidence for wrong-domain observations |
| Matchmaking | Cohort/input bounds, per-player Pending quota, immutable request/result, exact atomic commit, overlap/duplicate rejection, expiry materialization, End-vs-commit and overlapping-candidate orderings, candidate/roster/retained caps | Covered; add explicit no-mutation evidence for wrong-domain observations |
| OIDC production adapter | Raw token/JWKS bounds before parse, bounded JWK count, narrow RS256/JWK profile, signature/issuer/audience/expiry/nonce checks, unknown-key failure, core trust-set rejection, no implicit PlayerId/association mutation | Covered; verification map only |
| RunenNet/game-server composition | Explicit host-owned redemption→mapping→admission, failed authorization, post-redemption RunenNet failure without rollback, Assignment-end separation, retained membership rebind without grant replay | Covered; verification map only |
| Public framework surface | Core crate docs identify Rust representations/errors/handles as provisional implementation-local surfaces; OIDC docs separate realization time/provider data from RunenOnline semantics; examples are downstream proof packages | Covered; navigation/assurance clarification only |

## Hardening findings

### 1. Hostile input and amplification — no product defect found

The current in-process `Authority` enforces finite implementation policy for every retained collection it owns. Remotely influenced external-principal representations, matchmaking cohorts/inputs, request fan-out and live grant/request indexes are bounded before proportional retained growth.

The OIDC adapter enforces raw ID-token and JWKS byte limits before parsing, a finite JWK-count limit before activation, and a bounded RSA public-key profile. One raw token/JWKS document is itself the outer representation bound for claim/key material parsed inside it.

No batching API exists for identity, Assignment, AdmissionGrant, or MatchRequest creation, so one call cannot create an unbounded semantic batch by hidden fan-out.

### 2. Replay, stale evidence and expiry — semantics covered

Current assurance already proves:

- Assignment expiry is irreversible under a later lower observation;
- stale Assignment resolution cannot mutate a replacement Assignment;
- AdmissionGrant redemption succeeds at most once;
- grant expiry and Assignment-end invalidation are irreversible;
- lazy expired grants/MatchRequests are reconciled before applying live quotas;
- expired Match candidates terminalize only the expired requests and consume none;
- a consumed/Ended MatchRequest cannot be consumed again;
- overlapping Match candidates have exactly one sequentially ordered winner;
- RO5 reconnect uses retained RunenNet membership rather than reauthorizing from a redeemed grant.

The in-process realization currently retains terminal Assignments, grants, MatchRequests and Matches up to explicit finite partition caps. It therefore does not compact away stale-rejection evidence. Designing compaction/tombstones/durable replay evidence is not required until a realization actually introduces compaction or restart persistence.

### 3. Trusted-time boundary — one assurance gap, no implementation bug

Deadline-sensitive public paths validate the `TimeDomainHandle` before expiry/currentness mutation. A wrong-domain observation therefore fails closed before changing established state.

That ordering is visible in implementation but only partially explicit in integration tests. RO6 should add one focused test proving a wrong-domain observation cannot:

- expire/resolve a Pending Assignment;
- redeem/expire an AdmissionGrant;
- expire/consume a MatchRequest through Match commit.

After each rejected operation, a valid same-domain observation must show the object remains in its prior live state.

This adds assurance for accepted semantics; it does not add a new clock rule. The specification deliberately leaves wall-clock/monotonic representation and synchronization open.

### 4. Authorization/trust boundaries — no API change justified

`Authority::accept_verified_external_principal` is intentionally a trusted host/adapter boundary. Safe Rust cannot prove that an arbitrary host actually verified a credential, and adding a credential protocol to the core would violate the identity specification.

`VerifiedExternalPrincipal` itself cannot be freely constructed by callers. Authority-domain IDs remain scoped by process-local domain capability identity. `TrustedTime` is intentionally host-constructed only after obtaining the authority's comparison-domain handle.

The OIDC adapter keeps cryptographic verification distinct from RunenOnline issuer trust: a valid token from an issuer not configured in the owning `Authority` is rejected at the core handoff.

RO5 similarly keeps RunenOnline redemption distinct from RunenNet participant admission and transport binding.

### 5. Concurrency and atomicity — sequential assurance is truthful today

`Authority` exposes mutation through exclusive `&mut self`. It does not expose a shared concurrent mutation API, background worker, actor runtime, database transaction implementation, or distributed commit service.

The supported in-process realization therefore serializes semantic commits by construction. Existing deterministic two-order tests are the truthful assurance level for competing terminal operations and overlapping Match candidates.

Adding thread stress, `loom`, locks, actors, or a transaction abstraction would test or introduce an execution model the public realization does not currently own. A future concurrent/durable realization must add assurance appropriate to its actual mechanism.

### 6. OIDC production failure boundary — already strong

The adapter deterministically covers malformed/bounded input, unsupported JWK/token profile, signature failure, issuer/audience failure, exact expiry, nonce policy, unknown/missing key ID, duplicate/excessive keysets, and RunenOnline trust-set rejection. It performs no HTTP/discovery/refresh work, so network ambiguity and stale-key refresh policy remain correctly host-owned.

Point-in-time dependency security review on 2026-08-25 found no current defect in the pinned stack:

- `jsonwebtoken` is pinned to `11.0.0`; GHSA-h395-gr6q-cpjc / CVE-2026-25537 affected versions below `10.3.0` and was fixed in `10.3.0`;
- the lockfile carries `aws-lc-sys 0.44.0`; RUSTSEC-2026-0047 / CVE-2026-3338 is patched in `>=0.38.0` and concerns AWS-LC PKCS7 verification rather than this adapter's JWT RSA path.

Sources:

- https://github.com/advisories/GHSA-h395-gr6q-cpjc
- https://rustsec.org/advisories/RUSTSEC-2026-0047.html

A live advisory database check should not be inserted into deterministic semantic conformance merely to satisfy RO6. Organization dependency/security automation may evolve independently.

### 7. Observability — no new surface justified

Current public operations already return typed outcomes/errors and immutable views sufficient for deterministic host logging and debugging of the supported in-process behavior. No production service/process boundary exists that requires framework-owned metrics, tracing, audit events, or event IDs.

Adding an observability API now would risk creating a competing event/history model, especially for lazy expiry and partial cross-framework progress. RO6 should add none.

### 8. Public API and documentation — clarify, do not churn

The current crate-level and type-level docs already state that:

- Rust ID/domain/time representations are implementation-local and not wire/storage standards;
- `Authority` is one bounded in-process realization, not persistence/service topology;
- `AuthorityError` is a provisional implementation diagnostic surface, not a normative failure taxonomy;
- OIDC types are adapter-local realization types;
- proof packages do not define universal host mappings.

Removing numeric getters, domain handles, trusted-time construction, or current error variants would therefore be churn rather than evidence-backed hardening.

The missing durable artifact is the assurance map itself. `docs/verification/` is already the repository-owned location for it, but currently does not exist.

## Explicit non-gaps / deferred work

Do **not** create RO6 work merely for:

- a public AdmissionGrant credential/token format;
- public ID/domain/time serialization;
- one global clock, epoch, skew or synchronization mechanism;
- persistent storage/restart restoration or terminal-state compaction;
- remote service/RPC idempotency and ambiguous-commit recovery;
- a concurrent shared `Authority` container;
- generic Repository/transaction/actor abstractions;
- OIDC discovery, HTTP, automatic JWKS refresh, login/logout/session orchestration;
- metrics/tracing/audit-event APIs;
- Runenwerk integration;
- public API 1.0 stability or compatibility policy.

Those areas are either explicitly open normative items, unsupported realization mechanisms, or RO7/future work. Their absence does not weaken the currently supported contract claim.

## Selected RO6 delivery

Create one RO6B delivery slice: **supported-contract assurance surface**.

### Allowed files

- `docs/verification/supported-contracts.md` — new non-normative assurance map;
- `crates/runen-online/tests/acceptance_edges.rs` — append only focused wrong-time-domain no-mutation assurance;
- `README.md` — add one navigation link to the assurance map;
- `TESTING.md` — replace the generic future `docs/verification/` wording with a link to the actual assurance map if needed for navigation.

No other file is justified by RO6A.

### Forbidden in RO6B

Do not change:

- normative `spec/`;
- `crates/runen-online/src/`;
- `crates/runen-online-oidc/src/` or its manifest/dependencies;
- workspace/package manifests or `Cargo.lock`;
- `ARCHITECTURE.md`;
- proof package source;
- RunenNet or Runenwerk.

### Assurance document shape

The new verification document should remain concise and contain:

1. explicit statement that it is non-normative evidence, not a new conformance profile;
2. evidence map from each current normative owner to existing core/OIDC/proof tests;
3. production evidence for the bounded OIDC realization and downstream RO5 composition proof;
4. explicit nonclaims for unresolved serialization, persistence, service, clock, revocation, recovery, observability and API-stability areas;
5. canonical validation boundary (`cargo validate`).

Do not copy normative rules into the document.

### New deterministic edge scenario

Using the existing `acceptance_edges.rs` helpers:

1. establish a Pending Assignment, Redeemable AdmissionGrant and Pending MatchRequest under one valid time domain;
2. construct a `TrustedTime` from an unrelated `TimeDomainHandle` at or beyond their deadlines;
3. assert Assignment observation/resolution, grant redemption and Match commit reject with `TimeDomainMismatch`;
4. observe all three with a valid same-domain time still before their deadlines;
5. assert Assignment remains Pending, grant remains Redeemable and MatchRequest remains Pending.

This test demonstrates fail-closed comparison-domain isolation and no hidden terminalization/consumption from unrelated time evidence.

## Stop conditions

Stop RO6 delivery and route to separate normative investigation if implementation would need to decide any of the following:

- new public failure semantics rather than implementation diagnostics;
- credential/grant presentation or recovery semantics;
- persistent authority-domain/time representation;
- remote idempotency/transaction semantics;
- revocation or reconnect authority;
- standardized observability/audit history;
- a new public conformance profile.

## RO6 closure expectation

If RO6B lands with exact-head and merged-main `cargo validate` success and no new critical finding appears in its final review, no additional RO6 child is currently justified. The parent stage can then be reviewed for closure rather than creating hardening work by roadmap-category symmetry.
