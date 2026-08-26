# First production consumer extension boundary

Status: **non-normative investigation**

This record supports issue #48. It does not define RunenOnline semantics, authorize an implementation, or make Lantern Atlas deployment choices portable RunenOnline requirements.

Accepted investigation base: `6fb36fa4d97917ee8a0ddca8bde324f55f47730d`.

## Question

What is the first new RunenOnline capability actually required by the first concrete production consumer, rather than merely desirable from the existing roadmap vocabulary?

## Concrete consumer

The evaluated consumer is **Lantern Atlas** with this initial deployment boundary:

- browser/SvelteKit client;
- one Rust application backend embedding RunenOnline;
- PostgreSQL for durable application/account state;
- solo-first gameplay with optional multiplayer/guild interaction;
- no dedicated game-server fleet in the first slice;
- no RunenNet dependency until a later realtime gameplay path independently requires it.

The application owns characters, progression, guilds, expeditions, economy, browser transport, HTTP/WebSocket APIs, and persistence of game-domain state. Those choices are evidence for demand only.

## Decision

The first independently justified RunenOnline extension demand is **durable authority identity and principal-association continuity across backend restart**.

This is not yet a decision-complete implementation boundary. The next accepted work should therefore be a focused investigation of the durable identity/restart realization boundary, not a PostgreSQL implementation, generic repository abstraction, service/RPC layer, server allocator, RunenNet adapter, or new roadmap stage.

No `RO8` stage is justified by this investigation. `ROADMAP.md` already admits separately justified extensions after RO7, and one consumer-driven persistence realization does not establish a new durable multi-stage sequence by itself.

## Current capability map

| Consumer need | Current state | Classification |
| --- | --- | --- |
| Verify an already-obtained external identity token | Supported by optional `runen-online-oidc` within its bounded profile | Existing realization |
| Accept verified principal evidence and resolve/associate it | Supported by `runen-online` while one `Authority` remains alive | Existing semantic/core behavior |
| Create and use `PlayerId` inside one authority process | Supported and deterministically assured | Existing semantic/core behavior |
| Direct or matchmade Assignment and AdmissionGrant | Supported in-process and proven independently | Existing semantic/core behavior; not first consumer blocker |
| Browser HTTP/WebSocket API | No RunenOnline API is required; the host can call the core in-process | Host/application responsibility |
| Browser OIDC login flow, discovery, JWKS refresh | Outside the current bounded OIDC adapter and can remain host/operations owned for the first consumer | Host/operations responsibility |
| Dedicated-server fleet allocation | Not required by the initial consumer | No current demand |
| RunenNet participant/session composition | Existing proof exists, but the initial consumer does not require it | Deferred consumer demand |
| Preserve one RunenOnline player/account identity and principal association across backend restart | Not truthfully realizable through the current public implementation surface | **First blocking RunenOnline realization gap** |
| Persist Assignment, AdmissionGrant, MatchRequest, and Match state across restart | Not currently realized, but not required before durable identity for this consumer | Deferred broader persistence question |
| Multi-process/distributed authority execution | Not currently realized and not required by the initial single-backend deployment | Deferred scaling question |

## Why restart-safe identity is the first blocker

### The consumer requires durable account continuity

Lantern Atlas persists account/game state across backend restarts. RunenOnline identity is explicitly durable application state: the same authority-domain `PlayerId` incarnation must not silently become a different player merely because the hosting process restarted.

The application can keep its own game-domain account identifier, but that does not remove the RunenOnline requirement once RunenOnline `PlayerId` and external-principal association are used as the control-plane identity. Treating those values as disposable per-process state would make RunenOnline identity continuity weaker than the persistent account continuity the consumer is relying on.

### The current implementation is intentionally process-local

`AuthorityDomainHandle` creates a fresh process-local allocation identity. `Authority::new` consumes that handle, starts empty maps, and starts every semantic ID allocator from its initial local value. The public implementation exposes no restore/import path for:

- authority-domain identity;
- existing `PlayerId` values;
- external-principal associations;
- allocator non-reuse state.

Consequently a restarted process cannot reconstruct the same current `Authority` domain and identity state by reading host database rows and calling existing public APIs. Creating a fresh domain is a new semantic domain, not restoration of the old one.

### PostgreSQL is not the missing semantic authority

The identity specification deliberately does not require a public serialized authority-domain or `PlayerId` representation, database schema, or persistence API. It already requires enough domain context to prevent unrelated identities from comparing equal and requires non-reassignment/non-reuse.

RO4 therefore correctly classified persistence as an implementation-realization gap rather than a missing database-shaped semantic contract. PostgreSQL transactions and durable rows can realize a solution, but they do not determine:

- the durable authority-domain representation;
- the ownership of ID allocation/non-reuse state;
- the restore/bootstrap boundary;
- principal-association atomicity and uniqueness behavior;
- whether the existing in-process `Authority` remains the production authority or becomes one realization behind a different host-facing boundary.

A generic `Repository` trait around the current maps would make implementation topology into API authority before those questions are resolved.

## Gap ownership

### RunenOnline realization gap

The concrete blocker is preserving accepted identity semantics across process restart. This requires a production realization boundary owned by RunenOnline because it must preserve RunenOnline authority-domain identity, `PlayerId` non-reuse, and external-principal association meaning.

### Existing semantics, not new portable rules

Current evidence does **not** demonstrate a need to standardize:

- a universal authority-domain wire identifier;
- a universal `PlayerId` storage encoding;
- PostgreSQL schemas or SQL isolation levels;
- one transaction API;
- one application account model.

A later investigation may find a genuinely portable restart invariant missing from `spec/`, but #48 does not establish one.

### Host/application responsibility

The host can continue to own:

- browser endpoints and sessions;
- OIDC authorization flow and token acquisition;
- JWKS acquisition/refresh around the existing verifier;
- game-domain account/profile/progression data;
- application-specific mapping and policy that does not redefine RunenOnline identity.

### RunenNet responsibility

None is introduced by the first consumer slice. If realtime multiplayer later requires RunenNet, the accepted RO5 composition proof remains the starting boundary.

### Deferred RunenOnline questions

These are real but do not block the first consumer before identity restart continuity:

- durable Assignment/AdmissionGrant/Match persistence;
- concurrent/multi-process authority execution;
- remote RunenOnline service/RPC semantics and ambiguous-commit recovery;
- dedicated-server allocation/fleet realizations;
- standardized observability/history;
- stable wire/storage compatibility promises.

## Selected next issue boundary

Create one investigation issue, after #48 is accepted, with an outcome equivalent to:

> **Investigate durable authority identity and restart realization boundary**

It must determine the smallest production design that can restore one authority domain's player identities and external-principal associations without violating non-reuse or uniqueness semantics.

### Required questions

The investigation must decide:

1. what stable implementation-owned authority-domain identity is required for restart without making it a universal public/wire semantic identifier;
2. how `PlayerId` allocation/non-reuse state survives restart and concurrent durable writes;
3. what durable records are minimally required for player identity and principal association;
4. what atomic operations must be preserved for create/associate/resolve;
5. whether the production boundary is a new optional persistence adapter, a new realization of authority operations, or a refactor of the current core surface;
6. how the current in-process `Authority` and deterministic tests remain valid evidence without becoming the storage schema;
7. what deterministic restart/failure/concurrency proof is required before implementation;
8. whether resolving those questions exposes a missing provider-neutral invariant that must first be added to the owning normative identity specification.

### Explicit non-goals

The next investigation must not preselect:

- PostgreSQL or a Rust database library as portable authority;
- a generic repository/ORM/transaction trait;
- persistence for Assignment, AdmissionGrant, MatchRequest, or Match unless identity restoration proves inseparable from them;
- service/RPC transport;
- browser API design;
- server allocation;
- RunenNet or Runenwerk integration;
- a stable public wire/storage format;
- `RO8` naming.

### Stop conditions

Stop and route to normative investigation if a conforming durable realization cannot be specified without adding a provider-neutral identity/restart invariant. Stop and broaden package/architecture work only if concrete dependency/reuse evidence shows that the production identity realization cannot remain within the current package boundaries.

## Repository authority implications

For #48 itself:

- no `spec/` change is justified;
- no `ARCHITECTURE.md` change is justified;
- no `ROADMAP.md` change is justified;
- no `MATURITY.md` change is justified;
- no product source, manifest, lockfile, or dependency change is justified.

The single research record is sufficient evidence for the issue decision. Any next implementation authority must come from the separately accepted follow-up investigation.
