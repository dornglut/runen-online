# Matchmaking Lifecycle

Status: **provisional incomplete normative**

This document owns the minimum optional RunenOnline matchmaking-request lifecycle and atomic committed-Match semantics. It depends on [Core identity and external principal trust](../core/identity.md) and [Core control-plane time and deadlines](../core/time.md).

It does not define a matchmaking algorithm, queue implementation, party/lobby lifecycle, gameplay placement, gameplay admission, game simulation, or realtime networking.

Matchmaking is optional. A RunenOnline application MAY establish gameplay Assignments without creating any MatchRequest or Match.

## MatchRequest identity

`MatchRequestId` identifies one matchmaking-intent incarnation within one RunenOnline authority domain.

A MatchRequestId is opaque. This revision does not define its width, serialization, allocation algorithm, secrecy, or public API representation.

Within one authority domain:

- one MatchRequestId MUST identify at most one MatchRequest incarnation;
- a MatchRequestId MUST NOT later be reused for a distinct matchmaking intent;
- a realization MUST fail before collision, counter wrap, or reuse could make stale matchmaking evidence apply to another request incarnation.

A fresh matchmaking attempt after a terminal request outcome MUST use a fresh MatchRequestId.

MatchRequestId is not `PlayerId`, `MatchId`, `AssignmentId`, `AdmissionGrantId`, a provider queue/ticket identity, a gameplay-session identity, a RunenNet session/participant identity, or a transport connection.

## Atomic player cohort

Every MatchRequest contains exactly one **atomic player cohort**.

The cohort MUST:

- contain at least one `PlayerId`;
- contain only PlayerIds from the MatchRequest's authority domain;
- contain each PlayerId at most once;
- have a finite maximum cardinality under documented implementation/profile policy.

Cohort membership is fixed for the lifetime of one MatchRequestId.

The cohort means only that if this request is consumed into a Match, all of its players are consumed together. It does not create PartyId, LobbyId, social membership, leader authority, presence, invitation, or ready-check semantics.

## Trusted request establishment

Establishing a MatchRequest is a trusted RunenOnline control-plane operation.

A realization MUST NOT treat an untrusted list of PlayerId values as sufficient authority to enter those players into matchmaking merely because the identifiers are well formed.

The host/application is responsible for trusted policy that authorizes the exact cohort and matching intent before establishing the request.

This document does not define player-consent UI, party leadership, authentication flows, or the policy by which a host proves that it may act for the cohort.

## Matching inputs

A MatchRequest MAY carry host/application-defined inputs used by matchmaking policy.

The semantic matching intent and all request-carried matching inputs are fixed once that MatchRequest is established. Changing cohort membership or request-carried input in a way intended to create a new matchmaking intent requires a fresh MatchRequestId.

This revision does not standardize:

- skill/MMR representation;
- game mode or queue identity;
- region or latency data;
- team/role preference;
- ranking or quality score;
- search expansion/relaxation rules;
- arbitrary attribute schema;
- matchmaking algorithm.

A conforming realization MUST define finite accepted sizes/counts for remotely influenced matching-input representations before performing proportional work or retention.

Algorithmic behavior MAY change as trusted policy observes request age, system conditions, or other policy-owned context without mutating the request's fixed semantic inputs.

## MatchRequest lifecycle

The semantic lifecycle is:

```text
Pending
  ├─> Matched(MatchId)
  └─> Ended
```

`Matched` and `Ended` are terminal for one MatchRequest incarnation. These names are semantic states, not required public enum spellings.

### Pending

A newly established MatchRequest is Pending.

At establishment it MUST have:

- its fixed MatchRequestId;
- its authority-domain context;
- its fixed atomic player cohort;
- its fixed matchmaking inputs;
- one fixed finite deadline.

A Pending request MAY participate in matchmaking evaluation and candidate formation. It has not yet committed any Match membership.

### Deadline expiry

When a Pending MatchRequest's deadline is reached before a successful Match commit, the request is semantically Ended.

A realization MAY materialize that transition lazily, but an expired request MUST NOT subsequently be consumed into a Match.

The request lifetime is finite under the deadline requirements owned by [Core control-plane time and deadlines](../core/time.md).

### Trusted ending

Trusted control-plane policy MAY End a Pending MatchRequest before its deadline, including cancellation or failure policy.

The detailed Ended reason taxonomy is not defined by this revision.

Once Ended, the MatchRequest MUST NOT return to Pending or become Matched.

### Matched

A MatchRequest becomes Matched only as part of one successful atomic Match commit.

The Matched outcome records exactly one MatchId.

Once Matched:

- the request MUST NOT return to Pending or become Ended as an alternative terminal result;
- it MUST NOT be consumed by another Match;
- its cohort and matching inputs remain those of the original request incarnation.

Application behavior after successful matching, including whether players actually proceed to gameplay, is outside this request lifecycle.

## Terminal-race law

Ending a Pending request and consuming that request into a Match are competing terminal operations.

For one MatchRequestId, exactly one terminal result may win:

- Matched with one MatchId; or
- Ended.

A cancellation/end request that races with Match commit does not have priority merely because it was initiated first by a caller. The realization MUST provide an atomic ordering in which one terminal operation commits and the other observes or rejects the already committed outcome.

A deadline reached under the trusted expiry contract before Match commit prevents the request from being matched.

Delayed provider results, stale search candidates, retries, or cached proposals MUST NOT revive or consume a request after it has become terminal.

## Match identity

`MatchId` identifies one immutable committed matchmaking-result incarnation within one RunenOnline authority domain.

A MatchId is opaque. This revision does not define its width, serialization, allocation algorithm, or public API representation.

Within one authority domain:

- one MatchId MUST identify at most one Match incarnation;
- a MatchId MUST NOT later be reused for a distinct Match incarnation;
- a realization MUST fail before collision, counter wrap, or reuse could make stale Match evidence identify another result.

MatchId is not MatchRequestId, PlayerId, AssignmentId, AdmissionGrantId, a provider game/session/allocation identity, a RunenNet session/participant identity, or a transport connection.

## Match candidate

A **Match candidate** is a proposed finite set of one or more MatchRequestIds that matchmaking policy wants to commit as one Match.

Candidate generation, search order, scoring, partitioning, queue rules, and proposal transport are not normative Match semantics.

A candidate MUST be finite before the realization performs work proportional to its request count or resulting player roster.

A Match candidate has no authority to alter MatchRequests until an atomic Match commit succeeds.

## Atomic Match commit

A successful Match commit MUST occur as one semantic atomic operation over all selected MatchRequests.

Before commit, the realization MUST establish that:

1. every selected MatchRequest exists in the same RunenOnline authority domain;
2. every selected MatchRequest is still Pending;
3. no selected MatchRequest has reached its fixed deadline;
4. every selected MatchRequest is selected at most once;
5. every selected request's entire atomic player cohort is included;
6. the selected cohorts do not contain the same PlayerId more than once across the candidate;
7. the resulting request count and player roster satisfy finite implementation/profile limits;
8. the MatchId to be committed is fresh under the Match identity non-reuse rules.

The successful semantic commit MUST establish together:

- exactly one MatchId;
- the exact finite set of consumed MatchRequestIds;
- the exact cohort contributed by each consumed MatchRequest;
- the exact Match player roster, equal to the union of those cohorts;
- Matched(MatchId) as the terminal outcome of every consumed MatchRequest.

No partial Match commit is permitted.

If the full operation cannot commit, none of the candidate's still-Pending MatchRequests become Matched because of that candidate.

Semantic atomicity does not require one particular database transaction, consensus protocol, lock implementation, service topology, or RPC shape. A realization may choose any mechanism that exposes only the all-or-nothing committed outcome above.

## Cohort integrity and unique Match roster

A Match MUST preserve every consumed request's atomic cohort without splitting, dropping, replacing, or adding a player inside that cohort during commit.

The committed Match roster MUST contain each PlayerId at most once.

A candidate containing overlapping cohorts MUST therefore be rejected rather than silently deduplicating players or partially consuming one request.

This unique-roster rule applies within one Match only. A realization MAY permit one PlayerId to participate in multiple simultaneously Pending requests or distinct committed Matches when application policy allows it.

Because MatchRequest creation is remotely influenceable live work, every conforming realization MUST impose a documented finite maximum on the number of simultaneously Pending MatchRequests to which one PlayerId may belong within the authority domain. The mechanism used to enforce that maximum is not standardized; deterministic routing, coordinated quota state, or another bounded design MAY be used.

This per-PlayerId bound does not require a one-active-request policy. The finite maximum MAY be greater than one.

## Immutable Match result

A committed Match is an immutable matchmaking result, not an active gameplay object.

After commit:

- its MatchId is fixed;
- its consumed MatchRequestIds are fixed;
- the cohort attributed to each consumed request is fixed;
- its exact player roster is fixed.

The Match MUST NOT silently add, remove, replace, or backfill players after commit.

This revision does not define Match states such as Active, Started, Ended, Completed, Abandoned, or Failed.

Whether a committed Match is later ignored, archived, used for zero or more placement attempts, or transformed into application-specific game/session setup is downstream policy and does not mutate the matchmaking result.

## Relationship to Assignment

Matchmaking and Assignment are separate semantic domains.

Trusted application policy MAY use a committed Match as input when establishing a later Assignment, but:

```text
MatchId != AssignmentId
```

Match commit does not itself create an Assignment, select a gameplay destination, issue AdmissionGrants, or authorize gameplay entry.

This revision does not standardize Match-to-Assignment cardinality. A later application may retry placement with fresh AssignmentIds, share one gameplay destination among multiple Matches, or never create an Assignment for a Match, while preserving each owner's semantics.

Direct/persistent gameplay flows MAY establish Assignments without any MatchId.

## Concurrency and exactly-once request consumption

Concurrent Match candidates MAY overlap in MatchRequestIds.

At most one successful Match commit may consume a given MatchRequestId.

If one candidate commits first, every conflicting candidate MUST observe or resolve the request as no longer Pending and MUST NOT consume it again.

A duplicate retry of the same already-committed semantic Match operation MAY recover or report its existing committed Match when the realization has sufficient operation identity/evidence to do so. Such recovery MUST NOT create another Match or consume the requests again.

This revision does not define a public idempotency key, operation identifier, or RPC protocol. The realization MUST nevertheless preserve exactly-once semantic consumption of each MatchRequest.

## Stale-result safety

A realization MUST preserve enough authoritative evidence that delayed or replayed matchmaking output cannot consume a request after:

- the request was already Matched;
- the request Ended;
- the request expired;
- a fresh MatchRequestId was created for a later attempt;
- live request state was compacted under the realization's retention strategy.

This does not require indefinite retention of all historical requests or Matches. Non-reusing identifiers, terminal evidence, durable atomic state, bounded tombstones, versioned partitions, or another mechanism MAY be used when the semantic guarantees remain truthful.

## Resource and amplification requirements

Matchmaking introduces remotely influenced cohorts, matching inputs, search work, and atomic multi-request commits.

A conforming realization MUST define and enforce finite policy for:

- MatchRequest cohort cardinality;
- matching-input representation size/count before proportional work;
- MatchRequest creation fan-out in one operation or batch;
- maximum MatchRequest lifetime;
- simultaneously Pending MatchRequests attributable to one PlayerId;
- Match candidate request count;
- committed Match player-roster cardinality;
- each implementation-owned partition, queue, search set, or registry retaining Pending MatchRequests;
- concurrent candidate/commit work within each implementation-owned partition;
- stale-result, terminal, and idempotency evidence retained specifically to preserve request-consumption guarantees.

When a bounded live-state partition or per-PlayerId Pending-request quota is full, the realization MUST reject, shed, or backpressure additional work rather than grow that scope without bound.

This revision does not require one fixed global maximum across all MatchRequests in a horizontally scaled control plane. Aggregate service capacity MAY grow by adding independently bounded partitions while preserving the per-PlayerId bound.

Long-term Match history is not required. A realization MAY compact committed-Match or terminal-request state when identity non-reuse, atomicity, and stale-result guarantees remain truthful.

## Provider and policy boundaries

The following are not defined by the minimum matchmaking core:

- matchmaking search or evaluation algorithm;
- skill/MMR formulas and persistence;
- queue topology or queue identity;
- region/latency policy;
- match-quality scoring;
- expansion, relaxation, timeout heuristics beyond the request's semantic finite deadline;
- team balancing or team/role output representation;
- one-active-request-per-player policy;
- player proposal/acceptance stage;
- provider-specific waiting/search/acceptance/placing/server-allocation states;
- party/lobby, invitations, presence, ready checks, chat, or leader semantics;
- server backfill or join-in-progress mutation of an existing Match;
- gameplay placement/hosting implementation.

A provider realization MAY implement any of these, but provider behavior does not redefine MatchRequest or Match semantics.

## Non-equivalence

```text
MatchRequestId
    != MatchId
    != AssignmentId
    != AdmissionGrantId
```

A Match also is not a gameplay session, RunenNet session, provider server session, process, endpoint, or player membership lifecycle.

Equal representation or one-to-one mapping in one realization does not collapse these semantic domains.

## Open specification items

The following are **not defined by this revision**:

- public MatchRequestId or MatchId serialization;
- standardized matching-input schema;
- player proposal/acceptance lifecycle;
- standardized team/role result representation;
- Match retention/history lifetime;
- Match-to-Assignment policy or cardinality;
- backfill/join-in-progress semantics;
- exact per-PlayerId Pending-request maximum;
- public idempotency key or operation-recovery protocol;
- detailed Ended/failure reason taxonomy;
- matchmaking algorithm or provider API.

These open items are not permission to weaken the immutable Match result, finite-request, per-PlayerId live-work, or exactly-once MatchRequest-consumption rules.
