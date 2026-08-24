# Admission Grants

Status: **provisional incomplete normative**

This document owns the minimum per-player RunenOnline gameplay-entry authority. It depends on [Core identity and external principal trust](../core/identity.md), [Core control-plane time and deadlines](../core/time.md), and [Assignment lifecycle](../assignment/lifecycle.md).

It does not define authentication-provider credentials, matchmaking, gameplay simulation membership, transport connection establishment, RunenNet admission, reconnect, or a concrete token/credential format.

## Admission grant identity

`AdmissionGrantId` identifies one gameplay-entry grant incarnation within one RunenOnline authority domain.

An AdmissionGrantId is opaque. This revision does not define its width, serialization, generation algorithm, secrecy, or public API representation.

Within one authority domain:

- one AdmissionGrantId MUST identify at most one grant incarnation;
- an AdmissionGrantId MUST NOT later be reused for a distinct grant incarnation;
- a realization MUST fail before collision, counter wrap, or reuse could make stale grant evidence authorize another admission.

AdmissionGrantId is not `PlayerId`, `AssignmentId`, an external principal or provider credential, a gameplay-session identity, a RunenNet participant/session identity, a process/endpoint identity, or a transport connection.

## Grant binding

Each AdmissionGrant binds exactly:

- one `PlayerId` from the owning authority domain;
- one `AssignmentId` from that same authority domain;
- one fixed finite validity deadline;
- the authority-domain context necessary to prevent cross-domain acceptance.

The bound Assignment MUST be Usable when the grant is issued.

A grant MUST NOT be retargeted to another PlayerId, AssignmentId, authority domain, or logical gameplay destination after issuance.

A replacement Assignment requires a fresh AdmissionGrant even when it ultimately reaches equivalent infrastructure or was created for the same application intent.

## Representation and authenticity boundary

The grant's public/presentation representation is not defined by this revision.

A realization MAY use opaque random credentials with online lookup, self-contained signed credentials, sender-constrained credentials, destination-local authority, or another mechanism if it preserves all semantics in this document.

Before committing redemption, the authorized verifier MUST establish from trusted issued state or authenticated/integrity-protected grant evidence:

- the exact AdmissionGrantId;
- the bound PlayerId;
- the bound AssignmentId;
- the applicable authority-domain context;
- the fixed validity deadline.

Independent untrusted caller claims MUST NOT replace or widen those bindings.

A raw provider login/session credential MUST NOT become an AdmissionGrant merely by being forwarded to a gameplay destination. External authentication establishes principal evidence; gameplay-entry authority is a separate semantic grant.

Whether AdmissionGrantId itself is secret, or whether separate presentation material carries authority, is not defined by this revision.

## Issuance

Grant issuance is a trusted RunenOnline control-plane authorization operation.

This document does not define application policy for deciding which PlayerId should be allowed to enter an Assignment. It defines the resulting authority once trusted policy authorizes issuance.

A grant MAY be issued only while its bound Assignment is Usable.

At issuance:

- all grant bindings MUST be fixed;
- the validity deadline MUST be fixed;
- the lifetime MUST be finite under [Core control-plane time and deadlines](../core/time.md);
- the lifetime MUST NOT exceed a documented finite implementation/profile maximum.

One Assignment MAY have zero, one, or many AdmissionGrants. One PlayerId MAY have more than one AdmissionGrant when application policy permits it. This revision does not infer roster, capacity, uniqueness-per-player, or membership semantics from grant count.

## Redeemability

An issued AdmissionGrant is semantically **redeemable** only while all of the following hold:

- its fixed validity deadline has not been reached;
- its bound Assignment remains Usable;
- no successful semantic redemption has previously been committed for that AdmissionGrantId.

If the validity deadline is reached before successful redemption, the grant becomes permanently unredeemable.

If the bound Assignment becomes Ended before successful redemption, the grant becomes permanently unredeemable even when its nominal grant deadline is later.

A realization MAY materialize explicit grant states or compute redeemability from authoritative data. Required semantics do not depend on one storage representation.

## Assignment-currentness at redemption

The fact that the bound Assignment remains Usable is part of the authoritative redemption decision.

If an Assignment can become Ended before an issued grant's validity deadline, every component authorized to commit redemption for that grant MUST have a mechanism that prevents redemption after the Assignment becomes Ended.

A self-contained credential that can otherwise be verified without online state is conforming only when its validity regime or accompanying authority-state mechanism still preserves this assignment-currentness rule.

Possible realizations include centralized introspection, replicated assignment state, destination-local authoritative state, bounded authority leases, or another mechanism. None is standardized by this revision.

A stale cache or disconnected verifier MUST NOT broaden the period in which an Ended Assignment accepts a previously issued grant.

## Single semantic redemption

The initial AdmissionGrant model permits at most one successful semantic redemption per AdmissionGrantId.

Redemption MUST be atomic with respect to all components authorized to commit redemption for the same grant.

When concurrent presentations race:

- at most one may commit the successful redemption;
- every other presentation MUST NOT create another independent gameplay-entry authorization.

After successful redemption, later presentation of the same grant MUST NOT authorize another independent gameplay admission.

A realization MAY expose a separate query or recovery mechanism that reports the already committed redemption when a response was lost. Such recovery MUST NOT itself create a new gameplay-entry authorization. A public recovery protocol is not defined by this revision.

The grant is therefore not a reusable reconnect or connection-establishment token. A fresh independent gameplay-entry authorization requires a fresh AdmissionGrant unless a later normative profile explicitly defines another bounded model.

## Redemption outcome

Successful redemption commits one RunenOnline gameplay-entry authorization event for the bound PlayerId and AssignmentId.

That success establishes only that RunenOnline authorization. It does not itself:

- create or identify a transport connection;
- establish RunenNet protocol negotiation, session membership, ParticipantId, or connection binding;
- create application simulation membership;
- guarantee that the gameplay destination will accept every later application-level check;
- define reconnect, kick, migration, disconnect, or post-admission lifetime.

An application MAY map the authorized PlayerId to later gameplay/network membership according to separately owned semantics. Neither identity becomes the other merely because that mapping exists.

Grant expiry or Assignment ending after successful redemption MUST NOT retroactively change the redemption result into a failed redemption. Those later events also do not, by themselves, terminate gameplay/network membership already established by other semantic owners.

## Failed and permanently unredeemable cases

A redemption attempt MUST NOT successfully authorize gameplay entry when any of the following is true:

- the grant evidence cannot be authenticated/resolved to one issued grant under the authority-domain trust context;
- the fixed validity deadline has been reached;
- the bound Assignment is not Usable;
- successful redemption was already committed;
- the presented bindings conflict with trusted issued grant state;
- the grant belongs to another authority domain.

The public failure/reason taxonomy is not defined by this revision.

Individual trusted revocation of one still-live AdmissionGrant before its deadline is also not defined by this revision. Assignment ending remains sufficient to make all still-unredeemed grants bound to that Assignment unredeemable.

## Replay and stale evidence

A realization MUST preserve enough authoritative evidence to ensure that delayed or replayed grant presentation cannot become a fresh successful redemption after:

- the grant was already redeemed;
- the grant expired;
- its Assignment ended;
- a replacement Assignment was created;
- the grant's retained live state was compacted.

This requirement does not mandate indefinite storage of every historical grant. Non-reusing identifiers, bounded replay evidence, signed state, durable redemption records, partitioned tombstones, or another mechanism MAY be used if the semantic guarantees remain truthful.

## Resource and amplification requirements

Admission introduces remotely presentable short-lived authority and potentially concurrent verifier work.

A conforming realization MUST define and enforce finite policy for:

- grant issuance fan-out in one operation or batch;
- maximum AdmissionGrant lifetime;
- accepted grant/presentation representation size before proportional processing;
- AdmissionGrant fan-out per PlayerId and per AssignmentId where those live collections exist;
- each implementation-owned partition, registry, cache, queue, or store holding live grants, concurrent-redemption coordination, replay/idempotency evidence, or stale-grant rejection state;
- pending verifier work caused by concurrent presentations of one grant or one bounded partition.

When a bounded live-state partition is full, the realization MUST reject, shed, or backpressure additional work rather than grow that partition without bound.

This revision does not require one fixed global maximum across a horizontally scaled control plane. Aggregate capacity MAY grow by adding independently bounded partitions.

Long-term admission history or audit retention is not required.

## Non-equivalence

```text
external/provider credential
    != AdmissionGrant
    != gameplay/network membership
```

and:

```text
AdmissionGrantId
    != PlayerId
    != AssignmentId
```

A realization may map or embed these values, but equal representation does not collapse their semantic ownership.

## Open specification items

The following are **not defined by this revision**:

- public AdmissionGrantId or grant serialization;
- mandatory bearer versus proof-of-possession/sender-constrained presentation;
- cryptographic format, signature algorithm, key distribution, or introspection protocol;
- individual per-grant revocation before expiry;
- public recovery protocol after a successful redemption response is lost;
- failure/reason taxonomy;
- assignment/player capacity and roster policy;
- reconnect or reusable session-entry authority;
- post-admission gameplay/network membership lifetime;
- long-term grant/redemption audit storage;
- matchmaking-derived grant policy.

These open items are not permission for provider credential behavior to redefine AdmissionGrant semantics.
