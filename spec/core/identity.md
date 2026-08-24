# Core Identity and External Principal Trust

Status: **provisional incomplete normative**

This document owns the minimum RunenOnline player-identity and external-principal trust semantics. It does not define authentication protocols, gameplay assignment, admission grants, matchmaking, game simulation, persistence technology, or realtime networking.

## Authority domain

A **RunenOnline authority domain** is the semantic scope within which RunenOnline player identities and principal associations are interpreted.

An authority domain is a conceptual trust and identity namespace. This revision does not require it to have a public serialized identifier.

A RunenOnline authority domain MUST NOT be inferred merely from an MMO realm or shard, provider project or title identifier, hostname, database, deployment name, game server, or networking session.

A `PlayerId` value is meaningful only together with its owning authority-domain context. If an implementation exports or exchanges a `PlayerId` outside that context, it MUST preserve enough authority-domain context, either in the value or out of band, to prevent identities from unrelated domains being treated as equal merely because their local representations match.

Cross-authority-domain identity migration or equivalence is not defined by this revision.

## Player identity

`PlayerId` identifies one RunenOnline-local player/account incarnation within one authority domain.

A `PlayerId` is opaque. This revision does not define its numeric width, textual encoding, serialization, allocation algorithm, or whether the value is host-supplied or implementation-generated.

Within one authority domain:

- two references to the same claimed `PlayerId` value MUST identify the same player incarnation;
- a claimed `PlayerId` value MUST NOT later be reassigned to a different player incarnation;
- an implementation MUST reject or otherwise fail before an identity-generation collision, counter wrap, or value reuse could make an older identity refer to a different player;
- a `PlayerId` MUST NOT be interpreted as an external authentication principal or credential, gameplay placement or session identity, realtime-network participant/session identity, process identity, endpoint identity, or transport connection.

This revision does not require an `Active`/`Retired` player lifecycle. If an implementation or later profile retires or deletes player state, that operation MUST NOT make the old `PlayerId` value identify a different player incarnation.

A `PlayerId` MAY exist without an external-principal association. Whether a particular profile or application permits guest, device, anonymous, or application-created players is not defined by this revision.

## External principal identity

An **external authority** identifies a host-recognized authority whose verification mechanism can establish an external subject.

An **external subject** identifies one subject within one external authority.

An **external principal** is the ordered pair:

```text
(external authority, external subject)
```

A subject value by itself is not an external principal and MUST NOT be resolved without its external-authority scope.

The concrete representation of external authority and subject values is not defined by this revision. A conforming implementation MUST document and enforce finite maximum accepted representations for each before retaining or processing attacker- or remotely influenced values proportionally to their size.

An external authority becomes trusted only through host/configuration policy outside untrusted caller input. Supplying an arbitrary authority label MUST NOT create a trusted authority.

An external-authority adapter MUST provide subject identity semantics sufficient for one external-principal value not to be silently reassigned to a different external subject while a RunenOnline association relying on that value remains valid. Provider data that can be reassigned without an incarnation distinction cannot, by itself, satisfy this requirement.

Changes to the configured set or meaning of trusted external authorities are not defined by this revision.

## Verified external-principal boundary

RunenOnline core consumes **verified external-principal evidence** from a trusted host or provider adapter.

The verification mechanism is outside this document. This revision does not standardize OAuth, OpenID Connect, JWT, passwords, platform SDK authentication, signatures, refresh tokens, cookies, device credentials, or any other credential format.

A host or adapter that supplies verified evidence asserts that:

1. the external authority is trusted by the owning RunenOnline authority domain;
2. the external subject was verified according to that authority's accepted mechanism;
3. the resulting external-principal identity is the authority-scoped pair defined above.

Raw credentials or unverified caller claims MUST NOT cross this boundary as verified external-principal evidence merely because they contain values resembling an authority or subject.

Evidence that does not satisfy this boundary MUST be rejected before principal association or resolution. RO1A does not define credential-verification failure outcomes.

Successful external verification or principal resolution establishes identity evidence only. It MUST NOT by itself create gameplay placement, gameplay admission authority, game-session membership, realtime-network membership, or transport authorization.

## Principal association

A **principal association** maps one external principal to one `PlayerId` within one RunenOnline authority domain.

For associations defined by this revision:

- one external principal MUST be associated with at most one `PlayerId`;
- one `PlayerId` MAY have multiple associated external principals when permitted by implementation/application policy;
- multiple associations for one `PlayerId` MUST each remain distinct by their full external-principal pair;
- repeating an attempt to establish the same external-principal-to-`PlayerId` pair MUST NOT create a second semantic association or a second player identity;
- an attempt to associate an external principal already associated with a different `PlayerId` MUST fail rather than create ambiguous resolution.

A principal association MUST NOT change the identity of either side. The external principal remains externally scoped, and the `PlayerId` remains RunenOnline-local.

This revision defines establishment and resolution of associations only. Association removal, unlinking, reassociation, merge, recovery, provider migration, and deletion workflows are not defined by this revision. A provider or application MUST NOT infer normative reassociation behavior from this omission.

## Resolution

Given verified external-principal evidence, identity resolution has only these semantic outcomes in this revision:

- **Associated** — the external principal has one association to a `PlayerId`;
- **Unassociated** — no association exists.

These names are semantic outcomes, not required public enum spellings.

Resolution MUST NOT return more than one `PlayerId` for one external principal.

Creation of a new `PlayerId`, establishment of a new principal association, and resolution of an existing association are distinct operations even if one convenience API later combines them.

## Resource and amplification requirements

RunenOnline identity is durable application state, not a transient networking queue. This revision therefore does not require one fixed normative maximum for the total durable `PlayerId` population or total durable principal-association population of an authority domain.

A conforming implementation MUST nevertheless define finite limits wherever one remotely influenced operation or one identity can amplify retained or live work. At minimum it MUST document and enforce finite limits for:

- accepted external-authority representation size;
- accepted external-subject representation size;
- external-principal associations per `PlayerId`;
- principal or association items accepted by one atomic/batched operation, if batching exists;
- live pending/retry/conflict state retained to resolve concurrent identity or association operations, if such state exists.

Any tombstone, deduplication, collision-detection, or non-reuse evidence retained specifically by the implementation MUST have a storage/lifetime strategy consistent with the non-reuse guarantee. This document does not require that strategy to be an in-memory bounded registry.

Operational storage capacity, administrative quotas, database partitioning, and long-term archival policy are not defined by this revision.

## Non-equivalence rules

Within RunenOnline semantics:

```text
external principal
    != PlayerId
```

A credential that proves an external principal is also not a `PlayerId`.

No identity outside the concepts defined by this document becomes a `PlayerId` merely through equal representation or implementation convenience. A later normative owner may define explicit relationships while preserving `PlayerId` identity semantics.

RunenNet participant/session identities and transport connections remain separate external networking concepts; this document does not redefine them.

## Open specification items

The following are **not defined by this revision**:

- a serialized or public identifier for the RunenOnline authority domain;
- mutation of the authority domain's configured trust set;
- player retirement/deletion lifecycle beyond the non-reassignment rule;
- whether the first implementation profile must support guest/device players;
- external-principal unlinking, reassociation, merge, recovery, or account-provider migration;
- cross-authority-domain player migration or equivalence;
- authentication/credential verification protocols and their failure taxonomy;
- cryptographic identity formats;
- public wire or storage representation;
- database schema, transaction model, or persistence API;
- long-term audit/history requirements;
- gameplay assignment and admission authority;
- matchmaking, party, lobby, social, progression, economy, or game simulation semantics.

These open items are not permission for provider behavior, implementation behavior, or another repository to become RunenOnline semantic authority.
