# Assignment Lifecycle

Status: **provisional incomplete normative**

This document owns the minimum provider-neutral RunenOnline assignment identity and lifecycle. It uses the authority-domain rules from [Core identity and external principal trust](../core/identity.md) and the deadline rules from [Core control-plane time and deadlines](../core/time.md).

It does not define matchmaking, player rosters, gameplay simulation, dedicated-server orchestration, provider allocation state machines, transport connections, or per-player admission-grant semantics.

## Assignment identity

`AssignmentId` identifies one assignment incarnation within one RunenOnline authority domain.

An AssignmentId is fixed when the Assignment is established. If destination resolution is asynchronous, later placement outcomes refer to that already-established AssignmentId. If the destination is already known, the Assignment may be established directly as Usable under the same identity rules.

An AssignmentId is opaque. This revision does not define its width, serialization, allocation algorithm, or public API representation.

Within one authority domain:

- one AssignmentId MUST identify at most one assignment incarnation;
- an AssignmentId MUST NOT later be reused for a distinct assignment incarnation;
- a realization MUST fail before collision, counter wrap, or reuse could make delayed assignment evidence apply to another incarnation.

`AssignmentId` is not a `PlayerId`, matchmaking identity, gameplay-session identity, RunenNet session/participant identity, process/container/pod/VM identity, provider allocation/session identity, network endpoint, or transport connection.

## Logical gameplay destination

A **logical gameplay destination** is the authority target to which an Assignment grants the possibility of later gameplay-entry authorization.

The destination identity is host/application-defined. This revision does not define its public representation, endpoint format, process topology, or provider identifier.

A destination MAY be realized by, for example:

- one newly allocated dedicated game-server authority;
- one room hosted by a multiplexed server process;
- an already-running persistent shard or zone;
- a peer-hosted or relay-mediated authority;
- another application-defined gameplay authority.

These examples are illustrative only.

An Assignment does not own a player roster. Zero, one, or many players may later receive separately owned admission authority for one Assignment.

Multiple Assignments MAY resolve to the same logical gameplay destination when the host/application permits that topology.

## Connection material is not destination identity

Addresses, ports, DNS names, relay routing data, connection hints, provider resource names, or similar material are realization data associated with reaching a logical gameplay destination. They are not AssignmentId and are not by themselves the destination identity.

Connection material MAY change during one Usable Assignment only when the host preserves the same logical gameplay destination and does not broaden or reinterpret the Assignment's gameplay-entry scope.

Changing to a distinct logical gameplay destination requires a fresh AssignmentId.

The exact rule by which an application proves continuity of one logical gameplay destination across endpoint, process, or infrastructure changes is not defined by this revision.

## Lifecycle

The semantic lifecycle is:

```text
establish
  ├─> Pending
  │     ├─> Usable
  │     └─> Ended
  └─> Usable
        └─> Ended
```

These names are semantic states, not required public enum spellings.

## Establishment

Every new Assignment MUST establish:

- one fixed AssignmentId;
- its owning authority-domain context;
- exactly one initial semantic state: Pending or Usable.

An Assignment MAY be established directly as Usable when trusted control-plane policy already knows the logical gameplay destination.

An Assignment MAY be established Pending only when destination resolution remains outstanding.

Establishing the same AssignmentId more than once MUST NOT create multiple semantic Assignment incarnations.

## Pending

A Pending Assignment grants no gameplay admission authority.

Before any asynchronous placement result can be accepted for a Pending Assignment, it MUST have one fixed finite Pending deadline.

When that deadline is reached before a successful transition to Usable, the Assignment is semantically Ended. A realization MAY materialize that terminal state lazily, but it MUST NOT later accept a success result or otherwise treat the Assignment as Pending or Usable.

A Pending Assignment MAY become Ended earlier through trusted control-plane policy.

## Usable

An Assignment is Usable only after exactly one logical gameplay destination is fixed for that Assignment incarnation.

A Pending Assignment MAY transition to Usable only through one trusted successful destination-resolution decision applied to that exact AssignmentId before its Pending deadline is reached.

An Assignment established directly as Usable MUST fix its logical gameplay destination atomically with establishment.

Once Usable:

- the logical gameplay destination MUST NOT be replaced by a distinct destination under the same AssignmentId;
- later duplicate resolution evidence for the same destination MUST NOT create another semantic Assignment;
- later conflicting resolution evidence MUST be rejected;
- the Assignment MAY be used by the separately owned admission authority to issue per-player entry grants.

A Usable Assignment is not required to have a universal expiry or lease. It MAY remain Usable for an application-defined long lifetime, including persistent-world use.

## Ended

Ended is terminal.

A Pending or Usable Assignment MAY become Ended through trusted control-plane policy. The reason taxonomy is not defined by this revision.

An Ended Assignment:

- MUST NOT become Pending or Usable again;
- MUST NOT change its previously fixed logical destination, if any;
- MUST NOT be used as authority for new gameplay-entry grants;
- MUST NOT accept delayed destination-resolution success.

Ending an Assignment does not, by itself, eject already admitted players, terminate gameplay simulation membership, close transport connections, or terminate RunenNet membership. Those post-admission lifecycles are outside this document.

## Destination-resolution evidence and stale results

Every asynchronous outcome capable of changing a Pending Assignment lifecycle MUST be attributable to the exact AssignmentId whose unresolved destination it concerns.

A realization MUST reject destination-resolution success when any of the following is true:

- the referenced AssignmentId is unknown under the applicable retention/non-reuse policy;
- the Assignment is already Ended;
- the Assignment is already Usable with a conflicting destination;
- the Pending deadline has been reached;
- the outcome belongs to another Assignment incarnation.

A delayed outcome for an old AssignmentId MUST NOT mutate, complete, or replace a newer Assignment merely because both attempts originated from similar inputs or target the same provider resource.

## Retry and replacement

Reissuing provider RPCs, polling, or retrying transport delivery MAY remain part of one still-Pending AssignmentId when those operations are only attempts to determine the single outcome of that existing Assignment incarnation.

A retry or replacement that is permitted to create a different semantic Assignment outcome after the prior Assignment has Ended MUST use a fresh AssignmentId.

Provider retry identifiers, request IDs, queue IDs, and allocation objects are not AssignmentId unless a later realization deliberately maps them while preserving all Assignment semantics.

## Relationship to admission

Assignment establishes destination availability, not player authorization.

The separately owned [Admission grants](../admission/grant.md) specification defines whether and how one `PlayerId` may enter one Usable Assignment.

An Assignment becoming Ended prevents future gameplay-entry authorization through that Assignment but does not retroactively revoke a gameplay admission already successfully committed.

## Resource and amplification requirements

Assignment may introduce remotely triggerable live destination-resolution work. A conforming realization MUST define and enforce finite policy for:

- Assignment creation fan-out in one operation or batch;
- the maximum finite Pending lifetime when Pending is used;
- remotely influenced destination-selection input and destination/connection-material representation sizes before proportional work or retention;
- each implementation-owned partition, queue, or registry retaining Pending Assignments or destination-resolution correlation state;
- stale-result/idempotency working evidence retained specifically to preserve Assignment lifecycle guarantees.

When a bounded live-state partition is full, the realization MUST reject, shed, or backpressure additional work rather than grow that partition without bound.

This revision does not require one fixed global maximum across a horizontally scaled control plane. Aggregate capacity MAY grow by adding independently bounded partitions.

Long-term Assignment history or audit retention is not required. A realization MAY compact terminal state when its non-reuse and stale-result mechanisms remain sufficient to make the guarantees above truthful.

## Non-equivalence

```text
AssignmentId
    != logical gameplay destination
    != provider allocation/session identifier
    != endpoint/process identity
```

Equal representation or one-to-one mapping in a particular realization does not collapse these semantic domains.

## Open specification items

The following are **not defined by this revision**:

- public AssignmentId serialization;
- public logical-destination identity representation;
- connection-information format;
- destination-selection inputs such as region, mode, build, capacity, or latency;
- provider placement/allocation API shape;
- Ended reason taxonomy;
- a universal Usable lease or expiry;
- destination migration/continuity rules beyond preserving one logical destination identity;
- matchmaking-to-assignment composition;
- post-admission player membership, reconnect, kick, or migration semantics;
- long-term Assignment history/audit storage.

These open items are not permission for provider behavior to redefine Assignment semantics.
