# Supported Contract Assurance

This document is **non-normative verification evidence** for the currently supported RunenOnline surface. The `spec/` tree remains the sole authority for portable semantics.

It is not a conformance profile, interoperability claim, wire/storage contract, or public Rust API stability commitment.

## Evidence map

| Normative owner | Current executable evidence |
| --- | --- |
| [Core identity and external principal trust](../../spec/core/identity.md) | Authority-domain separation, identifier non-reuse/exhaustion, trusted external-principal acceptance, association uniqueness/conflict, stale-ID isolation, and finite representation/association boundaries are exercised by [`semantic_core.rs`](../../crates/runen-online/tests/semantic_core.rs), [`acceptance_edges.rs`](../../crates/runen-online/tests/acceptance_edges.rs), and [`limit_boundaries.rs`](../../crates/runen-online/tests/limit_boundaries.rs). |
| [Core control-plane time and deadlines](../../spec/core/time.md) | Exact deadline boundaries, overflow-safe lifetime checks, comparison-domain rejection, irreversible expiry, and foreign-domain no-mutation behavior are exercised by core unit tests plus [`semantic_core.rs`](../../crates/runen-online/tests/semantic_core.rs) and [`acceptance_edges.rs`](../../crates/runen-online/tests/acceptance_edges.rs). |
| [Assignment lifecycle](../../spec/assignment/lifecycle.md) | Direct/Pending establishment, one-way resolution, conflicting/stale resolution rejection, deadline expiry, End-vs-resolve orderings, finite lifetime/retention, and domain isolation are exercised by [`semantic_core.rs`](../../crates/runen-online/tests/semantic_core.rs), [`acceptance_edges.rs`](../../crates/runen-online/tests/acceptance_edges.rs), and [`limit_boundaries.rs`](../../crates/runen-online/tests/limit_boundaries.rs). |
| [Admission grants](../../spec/admission/grant.md) | Exact Player/Assignment binding, Usable-currentness, single successful redemption, replay handling, Assignment-end invalidation, irreversible expiry, lazy live-fanout reconciliation, finite lifetime/retention, and comparison-domain isolation are exercised by [`semantic_core.rs`](../../crates/runen-online/tests/semantic_core.rs) and [`acceptance_edges.rs`](../../crates/runen-online/tests/acceptance_edges.rs). |
| [Matchmaking lifecycle](../../spec/matchmaking/lifecycle.md) | Fixed bounded cohorts/inputs, Pending quotas, immutable requests/results, exact all-or-nothing Match commit, overlap/duplicate rejection, expiry materialization, terminal race orderings, exactly-once request consumption, roster/candidate limits, and comparison-domain isolation are exercised by [`semantic_core.rs`](../../crates/runen-online/tests/semantic_core.rs), [`acceptance_edges.rs`](../../crates/runen-online/tests/acceptance_edges.rs), and [`limit_boundaries.rs`](../../crates/runen-online/tests/limit_boundaries.rs). |

Tests are evidence for these owners; test structure, Rust method names, error variants, collection layout, and fixture choices do not become specification authority.

## Production realization evidence

The optional [`runen-online-oidc`](../../crates/runen-online-oidc/) crate is production-realization evidence for the accepted verified-external-principal handoff. Its deterministic [`verification.rs`](../../crates/runen-online-oidc/tests/verification.rs) coverage includes bounded raw token/JWKS input, bounded key count, the supported cryptographic/token profile, signature and key selection, exact issuer/audience/expiry/nonce checks, and RunenOnline trust-set rejection.

OIDC protocol, JWT/JWK representation, cryptographic dependencies, verification-time representation, and adapter-local failures are realization choices. They do not redefine portable RunenOnline identity or control-plane time semantics.

## Composition evidence

The downstream [`runen-net-composition`](../../examples/runen-net-composition/src/main.rs) proof demonstrates that a game-server host can consume RunenOnline and RunenNet without collapsing their authority domains. It exercises an explicit authorization-to-admission handoff, independent failure commits, post-admission lifecycle separation, and RunenNet recovery without reusing an AdmissionGrant as reconnect authority.

The proof-local PlayerId/ParticipantId and AssignmentId/SessionId mappings are application evidence only, not universal identity relationships or a standardized game-server API.

## Explicit nonclaims

The current supported assurance does not claim standardized behavior for:

- public serialization or wire/storage representation of RunenOnline IDs, authority domains, logical destinations, or trusted time;
- a public AdmissionGrant credential/presentation format or cryptographic profile;
- persistent restart restoration, terminal-state compaction, database schema, or durable transaction mechanism;
- remote service/RPC representation, idempotency key, ambiguous-commit recovery, or distributed transaction semantics;
- individual live-grant revocation, reusable reconnect authority, or a public redemption-recovery protocol;
- one global clock, epoch, unit, skew allowance, synchronization protocol, or timer API;
- a shared-concurrent or distributed `Authority` execution model;
- framework-standardized metrics, tracing, audit-event, or long-term history semantics;
- provider discovery, HTTP/JWKS refresh, persistence, service transport, or server-fleet realizations beyond separately accepted work;
- Runenwerk integration;
- stable pre-1.0 Rust APIs, compatibility/version policy, or a formal RunenOnline conformance profile.

Specification items that remain open stay open. Absence of a realization for an open item is not filled by current implementation or test behavior.

## Validation boundary

Repository acceptance remains the canonical [`cargo validate`](../../TESTING.md) gate. It checks the complete workspace, including these executable assurance paths, on the exact reviewed revision.

Passing tests demonstrate that the reviewed implementation preserves the currently supported contracts under the exercised scenarios. They do not grant tests or this document normative authority.
