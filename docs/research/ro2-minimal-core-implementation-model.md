# RO2 minimal semantic-core implementation model

Status: **non-normative investigation**

This record supports issue #19 under RO2 umbrella #18. It does not define RunenOnline semantics. The normative specification remains the sole owner of portable RunenOnline behavior; every Rust representation, storage choice, limit value, handle, and test fixture proposed here is implementation evidence only.

Accepted investigation base: `84ce302eecbc17ef4b184b6032837a396953a62e`.

## Question

What is the smallest standalone Rust implementation that can execute the accepted RO1 identity, time, assignment, admission, and optional matchmaking semantics deterministically without pre-creating provider, service, persistence, RunenNet, Runenwerk, or game-runtime architecture?

The implementation boundary must be strong enough to prove the accepted state machines and resource laws, but narrow enough that RO4 provider realizations and RO5 integration are not forced to inherit an in-memory test architecture as portable semantics.

## Authority and direct dependencies

Normative owners:

- `spec/conventions.md`;
- `spec/core/identity.md`;
- `spec/core/time.md`;
- `spec/assignment/lifecycle.md`;
- `spec/admission/grant.md`;
- `spec/matchmaking/lifecycle.md`.

Repository/package dependency law is owned by `ARCHITECTURE.md`. RO2 sequencing is owned by `ROADMAP.md`. Mechanical acceptance remains `cargo validate` from `TESTING.md`.

The RO1 research record and the existing RunenNet implementation are comparison evidence only. In particular, RunenNet's successful one-product-crate RN2 shape demonstrates that semantic document boundaries need not become crate boundaries, but it does not decide RunenOnline state, identity, or API design.

## Findings

### One product crate is sufficient

The accepted RO1 semantic domains are distinct authorities, but they do not currently demonstrate independent package ownership:

- assignment depends on core identity/time;
- admission depends on identity/time/assignment;
- matchmaking depends on identity/time and composes with assignment only through host policy;
- all areas need one consistent authority-domain and finite-policy realization;
- no domain currently has a distinct production dependency, deployment obligation, provider dependency, or reusable adapter boundary.

Creating `runen-online-core`, `runen-online-identity`, `runen-online-matchmaking`, `runen-online-admission`, provider, persistence, runtime, or service crates now would make semantic taxonomy become package topology without evidence.

**Proposed RO2 product shape:** exactly one product library crate:

```text
crates/runen-online
```

Internal modules may follow implementation cohesion (`identity`, `time`, `assignment`, `admission`, `matchmaking`, `limits`) without claiming permanent package or public API boundaries.

The RO2 delivery should update `ARCHITECTURE.md` only enough to record that RO2 establishes this one product crate and that later splits require independent dependency/build/reuse evidence.

### Do not introduce a generic persistence or transaction abstraction in RO2

A generic repository/database trait looks attractive because later providers will need durable state, but it is premature for the first semantic core:

- no production persistence contract is accepted;
- Match commit requires atomic mutation across multiple requests;
- AdmissionGrant redemption requires current Assignment authority plus single-redemption mutation;
- identity association requires uniqueness across principal bindings;
- premature transaction/repository traits would either expose database-shaped semantics or underspecify the atomic operations they are meant to preserve.

RO2 should therefore implement one bounded in-process semantic authority aggregate. RO4 production realizations may later map the same accepted semantics to transactions, actors, consensus, provider APIs, or other mechanisms without making the RO2 container a portable storage contract.

### One authority aggregate is the minimum coherent atomicity boundary

The first implementation should own one in-process `Authority`-like aggregate representing one RunenOnline authority-domain partition. The exact Rust name is implementation-local and may change before public API stabilization.

The aggregate owns bounded semantic state for:

- players and external-principal associations;
- assignments;
- admission grants;
- match requests;
- committed matches;
- per-object non-reusing local identity allocation state;
- the validated finite implementation policy for that partition.

Mutating semantic operations require exclusive mutable access to the aggregate. This provides one deterministic RO2 realization of semantic atomicity without claiming that production systems must use one process, lock, transaction, actor, or database.

For every multi-object operation, the implementation must validate all preconditions, resource limits, deadline truth, and fresh-ID availability before mutating committed state. Failure leaves prior committed state unchanged.

This is especially important for atomic Match commit: validate the complete candidate first, then create exactly one Match and transition every consumed MatchRequest together. No request may be partially consumed because an iteration fails after earlier requests were modified.

### Authority-domain context must be present in semantic identity equality

RO1 makes semantic identity equality meaningful only within an authority domain. Bare local integer newtypes would make equal values from unrelated in-process domains compare equal unless callers always remembered out-of-band context.

The first implementation should therefore use an opaque implementation-only `AuthorityDomainHandle` supplied by the trusted host when constructing the authority aggregate. It is a local implementation handle, not a standardized public/serialized RunenOnline realm identifier.

Each RunenOnline semantic ID should carry:

```text
(authority-domain handle, local incarnation value)
```

for:

- `PlayerId`;
- `AssignmentId`;
- `AdmissionGrantId`;
- `MatchRequestId`;
- `MatchId`.

A fixed Rust integer may represent the local incarnation value in RO2. Its width is an implementation choice, not wire/storage authority. The aggregate owns per-ID-type monotonic allocation state and rejects exhaustion before wrap/reuse. No process-global allocator is needed.

This design makes cross-domain equality fail structurally while leaving public serialization and cross-process authority-domain identity explicitly open.

### External principals remain host-verified input

RO2 must not implement OAuth, OIDC, JWT, passwords, provider SDKs, or credential verification.

The core should accept an explicit trusted-call boundary equivalent to `VerifiedExternalPrincipal` containing one bounded external-authority representation plus one bounded external-subject representation. Byte strings are sufficient as an implementation representation because the normative schema is opaque.

The host is responsible for verification before invoking this boundary. The core then enforces:

- configured external authority is recognized/trusted by this authority aggregate;
- full authority+subject pair identity;
- bounded representation sizes;
- one principal resolves to at most one PlayerId;
- repeated equal association is idempotent;
- conflicting association fails;
- per-player association fan-out is finite.

Trust-set mutation, unlinking, reassociation, merge, recovery, and deletion remain absent because RO1 leaves them open.

### Trusted time should use one explicit comparison domain and zero tolerance

RO2 needs deterministic deadline behavior without standardizing wall-clock or monotonic time.

The minimum implementation should use:

- one host-supplied opaque `TimeDomainHandle` configured for the authority aggregate;
- one fixed unsigned integer observation value;
- deadlines stored in that same comparison domain;
- comparison rule `observation >= deadline` means reached;
- **zero tolerance/leeway** in the first implementation profile.

The integer unit is host/runtime policy and is not a standardized timestamp or duration unit.

Every deadline-sensitive operation receives an explicit trusted time observation from the host. A mismatched time-domain handle fails closed. Reached deadlines are irreversible for the object incarnation.

No background timers, clocks, scheduler, async runtime, or provider-time dependency is required.

### Host policy remains outside semantic mechanisms

The authority aggregate executes semantic decisions but does not decide application policy.

The host/application remains responsible for deciding:

- whether to create a player;
- whether a verified principal may be associated with a PlayerId;
- whether to establish an Assignment and whether it starts Pending or directly Usable;
- which logical gameplay destination a trusted placement decision selected;
- whether a PlayerId should receive an AdmissionGrant;
- which exact MatchRequest cohort and matching intent is authorized;
- how matchmaking candidates are selected/scored;
- whether/how a committed Match is later used to establish an Assignment.

The core validates and commits the resulting semantic operation. It must not add a matchmaking algorithm, roster policy, dedicated-server allocator, admission policy, or Match-to-Assignment cardinality rule.

### Use local opaque handles for open application representations

RO2 should avoid inventing portable schemas for concepts whose representations are explicitly open.

Recommended implementation representations:

- logical gameplay destination: fixed-size opaque host-supplied local handle;
- matching inputs: bounded opaque byte string retained immutably on MatchRequest;
- external authority/subject: bounded opaque byte strings;
- AdmissionGrant presentation: **not implemented as a credential format in RO2**; semantic redemption operates on a resolved AdmissionGrantId inside the trusted in-process boundary.

The destination handle is not a process/container/endpoint/provider ID by definition; it is merely RO2's host-local reference to the application-defined logical destination.

Connection material, credential bytes, wire serialization, database rows, provider identifiers, HTTP/gRPC payloads, and cryptographic formats remain absent.

### Conservative bounded retention is preferable to premature compaction

RO1 permits terminal-state compaction when non-reuse and stale-artifact guarantees remain truthful, but does not require compaction.

RO2 should choose the simplest truthful bounded model:

- retain created semantic objects, including terminal objects, inside the authority aggregate for that aggregate's lifetime;
- bound each retained collection with validated finite capacity;
- use monotonic non-reusing local identity allocation;
- reject new work when a configured partition capacity is exhausted.

This is deliberately conservative and not a production scaling strategy. It avoids introducing tombstone lifetimes, persistence epochs, compaction protocols, or stale-result retention heuristics before RO4 evidence exists.

A later production realization may compact terminal state if it independently preserves non-reuse and stale-result guarantees.

## Finite implementation policy

The first authority aggregate requires a validated finite configuration. Exact numeric values are implementation/profile policy, not normative RunenOnline constants.

The configuration should cover at least:

### Identity

- maximum external-authority bytes;
- maximum external-subject bytes;
- maximum principal associations per PlayerId;
- maximum retained PlayerIds in this in-process partition;
- maximum retained principal associations in this partition.

No identity batch API is required in RO2, so per-operation identity fan-out is one.

### Assignment

- maximum retained Assignments in the partition;
- maximum finite Pending-Assignment lifetime in the configured time-value units.

RO2 accepts one Assignment establishment per operation, does not accept destination-selection request blobs, and uses a fixed-size destination handle; therefore separate batch/input/connection-material limits are not required by the first API.

### Admission

- maximum retained AdmissionGrants;
- maximum AdmissionGrant lifetime;
- maximum live/retained grants per PlayerId;
- maximum live/retained grants per AssignmentId.

Grant issuance is one grant per operation. No external serialized presentation is accepted by RO2, so a presentation-byte limit belongs to a later credential/service realization.

### Matchmaking

- maximum retained MatchRequests;
- maximum MatchRequest lifetime;
- maximum cohort cardinality;
- maximum matching-input bytes;
- maximum simultaneously Pending MatchRequests attributable to one PlayerId;
- maximum candidate MatchRequest count;
- maximum committed Match roster cardinality;
- maximum retained committed Matches.

MatchRequest creation is one request per operation. Candidate data is caller-owned until commit; the core validates candidate cardinality before proportional lookup/work.

### Concurrency and working state

The first realization retains no asynchronous operation queue, retry registry, proposal queue, verifier queue, or background candidate work. Exclusive mutable operation entry means only one semantic mutation is committed at a time per aggregate.

Therefore the initial implementation can satisfy several working-state bounds structurally with zero retained pending work beyond the normative objects themselves. If delivery adds any retry/idempotency/candidate cache, it must gain an explicit finite bound before acceptance.

## Operation model

The exact function names are not public API authority, but RO2 delivery should expose operations equivalent to the following responsibilities.

### Identity

- create a fresh PlayerId;
- accept bounded verified external-principal evidence;
- establish one principal association;
- resolve a verified principal as Associated or Unassociated.

No retirement/unlink/reassociation operations.

### Assignment

- establish Pending with one fixed deadline;
- establish directly Usable with one fixed destination;
- resolve a Pending Assignment to one destination before deadline;
- End a Pending or Usable Assignment;
- observe state using explicit trusted time so reached Pending deadlines cannot remain live.

Duplicate same-destination completion may be idempotent; conflicting completion fails. Ended and expired state is terminal.

### Admission

- issue a fresh AdmissionGrant for one PlayerId + currently Usable Assignment + finite deadline;
- redeem by resolved AdmissionGrantId using explicit trusted time;
- return success at most once;
- reject redemption after deadline, after Assignment Ended, after prior redemption, on unknown/cross-domain identity, or on binding conflict.

The first implementation does not model reconnect or reusable presentation authority.

### Matchmaking

- establish one immutable finite MatchRequest with exact cohort, opaque bounded inputs, and finite deadline;
- End a Pending request through trusted policy;
- atomically commit one finite candidate set into exactly one Match;
- observe request state with explicit trusted time.

Match commit validates all requests and all cohorts before mutation, including expiry, domain, duplicate request IDs, overlapping players, configured request/roster limits, and fresh MatchId availability.

The resulting Match stores the exact consumed MatchRequestIds, exact cohort contributed by each request, and exact unique roster. It has no Active/Ended gameplay lifecycle.

## Error/result restraint

RO1 intentionally leaves detailed public failure taxonomies open. RO2 still needs deterministic distinctions to test invariants, but it must not turn diagnostic granularity into new portable semantics.

Use operation-local result types with only the distinctions required for executable behavior, for example:

- success / already-committed-idempotent result where allowed;
- unknown identity/object;
- authority-domain mismatch;
- trusted-time-domain mismatch;
- expired/terminal/not-current state;
- conflicting binding or transition;
- malformed/duplicate candidate input;
- resource limit exceeded;
- identity/time counter exhaustion.

Detailed provider, credential, placement, cancellation, or failure reason enums remain deferred.

## Atomicity and concurrency assurance

RO1 requires semantic atomicity, not one synchronization mechanism.

RO2 should realize this through exclusive `&mut`-style authority mutation and prevalidation. Deterministic tests should prove race laws by exercising both possible operation orders rather than relying on thread timing:

- Assignment End versus pending resolution;
- grant redemption versus Assignment End;
- first versus duplicate/concurrent grant redemption;
- MatchRequest End versus Match commit;
- overlapping Match candidate A then B versus B then A.

Exactly one valid terminal outcome must win in each ordering. Failure of the second operation must leave all unrelated state unchanged.

A later concurrent/distributed realization must reproduce these same semantic outcomes using its own locking/transaction/consensus mechanism.

## Executable assurance map

RO2 delivery should organize tests by normative responsibility rather than by provider or copied application flow.

### Identity / trust

- equal local values in different AuthorityDomainHandles are not equal semantic IDs;
- local allocators never reuse and fail on exhaustion before wrap;
- external authority and subject size limits reject before retention;
- unrecognized external authority cannot become trusted through caller labels;
- one principal cannot resolve ambiguously;
- duplicate equal association is idempotent;
- conflicting association fails without mutation;
- per-player and partition capacities are enforced.

### Time

- not-reached versus reached comparison at the exact boundary;
- mismatched TimeDomainHandle fails closed;
- once an object is treated expired it cannot become live through a smaller later observation;
- configured maximum lifetimes reject oversized requested deadlines before object creation.

### Assignment

- direct Usable establishment requires one fixed destination;
- Pending establishment requires finite deadline;
- pre-deadline resolution succeeds once;
- reached deadline prevents later success;
- End wins against later resolution;
- stale result for old AssignmentId cannot affect a newer Assignment;
- conflicting destination evidence cannot replace a fixed destination;
- partition capacity rejection leaves state unchanged.

### Admission

- issuance requires current Usable Assignment;
- grant binding is exact and immutable;
- grant maximum lifetime and fan-out limits are enforced;
- expiry before redemption is permanent;
- Assignment End before redemption blocks redemption;
- first redemption succeeds once;
- repeated/concurrent-order redemption does not create another authorization;
- ending Assignment after successful redemption does not rewrite the committed result.

### Matchmaking

- request cohort is non-empty, unique, same-domain, finite, and immutable;
- matching inputs are bounded and immutable;
- request deadline is finite and terminal expiry is irreversible;
- per-player Pending-request fan-out is enforced;
- trusted End versus Match commit has exactly one winner;
- duplicate request IDs in a candidate fail before mutation;
- overlapping cohorts fail rather than deduplicate;
- one request cannot be consumed twice by overlapping candidates;
- Match commit is all-or-nothing across every selected request;
- roster and candidate limits reject before commit;
- committed Match contents are immutable and exactly equal to consumed request cohorts;
- direct Assignment APIs remain usable without MatchRequest/Match.

### Cross-cutting resource/rollback

For each configured finite limit, test the exact boundary and one-over-boundary case. Every rejected operation must preserve previously committed state and accounting.

No property-testing dependency is required initially. Table-driven and bounded deterministic transition-sequence tests are sufficient unless implementation defects demonstrate a need for stronger tooling.

## Competing models rejected

### Separate crate per specification directory

Rejected. Semantic ownership does not imply independent build/dependency/reuse ownership and would create package ceremony/cycles before evidence.

### Service-first architecture

Rejected. HTTP/gRPC endpoints, auth service, matchmaking service, allocation service, and database processes are RO4 realization questions, not RO2 semantic-core prerequisites.

### Generic `Repository` / transaction traits before implementation

Rejected. They would either be database-shaped, too weak to express cross-object atomicity, or become a speculative abstraction around one in-memory realization.

### Pure stateless transition functions only

Rejected as the sole RO2 product model. Individual state transitions can be pure internally, but the accepted semantics include uniqueness, bounded retained collections, assignment-currentness at grant redemption, per-player MatchRequest fan-out, and atomic multi-request Match consumption. A coherent first executable realization therefore needs owned authority state around those transitions.

### Provider IDs as RunenOnline IDs

Rejected. It collapses authority-domain/non-reuse law into provider behavior and prevents provider replacement.

### Bare unscoped integer semantic IDs

Rejected for the first implementation because cross-domain equality would be too easy to misinterpret. Pairing a local implementation domain handle with the local incarnation value preserves the accepted scope rule without standardizing wire identity.

### Global process ID allocator

Rejected. It makes process lifetime hidden identity authority and complicates deterministic standalone use. Allocation state belongs to the authority aggregate.

### Runtime clock/background expiry tasks

Rejected. Explicit trusted observations are deterministic, provider-neutral, and sufficient to implement irreversible deadline semantics.

### Cryptographic AdmissionGrant format in RO2

Rejected. Signature algorithms, bearer/proof-of-possession choices, key distribution, introspection, and presentation encoding are explicitly open/provider-realization concerns. RO2 implements the semantic issued/redemption state only.

### Terminal-state compaction in the first implementation

Rejected for RO2 unless capacity evidence forces it. Bounded conservative retention is simpler and preserves stale/non-reuse guarantees without inventing tombstone policy.

## Proposed RO2 delivery boundary

If this investigation is accepted, create one bounded implementation child under #18 that:

1. adds exactly `crates/runen-online`;
2. updates `ARCHITECTURE.md` to record the one-product-crate RO2 package decision;
3. implements one bounded in-process authority aggregate and the accepted RO1 operations above;
4. adds explicit validated finite policy and host-supplied authority/time-domain handles;
5. adds deterministic unit/integration assurance covering all accepted RO1 invariants that the typed in-process core can realize;
6. adds one concise non-normative verification map only if useful to avoid losing requirement-to-test traceability;
7. adds no provider, service, persistence, RunenNet, Runenwerk, ECS, async-runtime, serialization, cryptographic credential, or matchmaking-algorithm dependency.

RO2 should remain one implementation delivery unless the actual diff demonstrates an independently reviewable boundary that materially improves correctness or reviewability. Do not pre-create multiple crates or delivery slices for symmetry.

## Explicit deferrals

The first RO2 core does not decide:

- public stable API naming/ergonomics;
- public/wire/storage serialization of any ID;
- cross-process serialized authority-domain identity;
- credential verification or AdmissionGrant presentation format;
- proof-of-possession versus bearer presentation;
- database schema, persistence API, transaction provider, event sourcing, or compaction strategy;
- HTTP/gRPC/service topology;
- server allocation/provider APIs;
- connection material format;
- Match-to-Assignment cardinality/policy;
- matchmaking algorithm, queues, MMR, team/role schema, backfill, party/lobby;
- post-admission gameplay/network membership, reconnect, kick, migration;
- RunenNet/Runenwerk composition;
- production horizontal partition routing.

These remain open or later-stage concerns. RO2 implementation choices must not be cited as portable semantic authority for them.

## Conclusion

RO1 is sufficiently complete to implement, but not sufficiently specific to justify provider/service architecture. The minimum long-lived path is one standalone `runen-online` product crate containing a bounded in-process semantic authority realization with explicit host inputs and deterministic state-machine assurance.

This model exercises every currently accepted semantic obligation while keeping the future clean: RO3 can prove ordinary plain-Rust consumption; RO4 can introduce production persistence/auth/service/allocation realizations against proven semantics; RO5 can compose RunenNet/game servers without either framework becoming hidden authority.