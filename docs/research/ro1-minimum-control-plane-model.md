# RO1 minimum control-plane model research

Status: **non-normative investigation**

This record supports issue #3. It does not define RunenOnline semantics. External standards and provider APIs are evidence only; every proposed normative rule requires a separately accepted specification change.

## Question

What is the smallest provider-neutral control-plane model that can take an authenticated or otherwise trusted game user to an authorized gameplay destination across card games, FPS, action games, fighting games, co-op, persistent worlds, and MMO zone/shard topologies without duplicating RunenNet, game simulation, or one backend provider's ontology?

## Existing authority boundary

Current RunenOnline authority defines no control-plane semantic area yet. The accepted roadmap requires RO1 to resolve only the minimum implementation-critical identity, trust/admission, lifecycle, resource-bound, coordination, and assignment contracts.

The sibling boundary is already strong:

- RunenNet `ParticipantId` is distinct from authentication accounts, principals, tickets, platform users, and transport connections;
- authentication, matchmaking, server discovery, connect-ticket issuance, lobby membership, roster policy, and game settings are outside RunenNet session semantics;
- successful application authentication does not replace RunenNet protocol negotiation or RunenNet participant admission.

Therefore RunenOnline should produce control-plane authority that a game/application server may consume before and alongside RunenNet admission. It must not redefine RunenNet admission itself.

Relevant sibling authority:

- [RunenNet core identity](https://github.com/dornglut/runen-net/blob/main/spec/core/identity.md)
- [RunenNet session lifecycle](https://github.com/dornglut/runen-net/blob/main/spec/session/lifecycle.md)
- [RunenNet protocol negotiation](https://github.com/dornglut/runen-net/blob/main/spec/protocol/negotiation.md)
- [Dornglut ADR 0006](https://github.com/dornglut/engineering/blob/main/adrs/0006-separate-realtime-networking-from-online-control-plane.md)

## External evidence

### Authentication identity is not RunenOnline player identity

OpenID Connect identifies an authenticated subject using issuer-scoped claims: `iss` identifies the issuer and `sub` is unique within that issuer. Audience, expiry, signature validation, and optional nonce/replay checks are separate concerns. Pairwise subject identifiers may intentionally differ between clients.

Nakama likewise maps external sign-in identities to its own user account and then issues a separate time-limited session credential. One account can have multiple linked authentication methods.

Portable conclusion: an external provider subject or provider token must not itself become the RunenOnline player identity.

Sources:

- [OpenID Connect Core 1.0](https://openid.net/specs/openid-connect-core-1_0-18.html)
- [Nakama authentication](https://heroiclabs.com/docs/nakama/concepts/authentication/)
- [Nakama session management](https://heroiclabs.com/docs/nakama/concepts/session/management/)

### Gameplay admission authority needs explicit scope and replay policy

OAuth 2.0 Security Best Current Practice recommends audience restriction, least privilege, and sender-constrained tokens where practical to reduce misuse of stolen credentials. These are security principles, not a requirement that RunenOnline use OAuth, JWT, mTLS, or DPoP.

For RunenOnline, gameplay admission authority should be separately scoped from login credentials, finite in lifetime, explicitly targeted, and replay-aware.

Sources:

- [RFC 9700 — OAuth 2.0 Security Best Current Practice](https://www.rfc-editor.org/rfc/rfc9700.html)
- [Nakama matchmaker](https://heroiclabs.com/docs/nakama/concepts/multiplayer/matchmaker/)

### Match formation and gameplay placement are separate concerns

Open Match forms matches from tickets, then its Director can obtain dedicated-game-server allocation separately and assign connection data back to tickets.

Amazon GameLift FlexMatch can similarly produce matchmaking results that then enter game-session placement. A placement can be `PENDING` through multiple placement attempts; connection information is not final until the placement is `FULFILLED`.

PlayFab can deliberately collapse these layers by making a Multiplayer Servers `SessionId` equal to the matchmaking `MatchId`. That is a valid provider realization, not a portable identity law.

Portable conclusion: RunenOnline must preserve the ability to keep logical grouping/match identity distinct from gameplay-placement identity.

Sources:

- [Open Match Director](https://open-match.dev/site/docs/guides/matchmaker/director/)
- [Open Match Director tutorial](https://open-match.dev/site/docs/tutorials/matchmaker101/director/)
- [GameLift GameSessionPlacement](https://docs.aws.amazon.com/gameliftservers/latest/apireference/API_GameSessionPlacement.html)
- [PlayFab Multiplayer Servers integration](https://learn.microsoft.com/en-us/xbox/playfab/multiplayer/matchmaking/multiplayer-servers)

### Dedicated-server allocation is only one placement realization

Agones `GameServerAllocation` atomically selects an eligible GameServer and can move a `Ready` GameServer to `Allocated`. GameLift placement searches hosting capacity and may create multiple candidate game-session attempts before fulfillment. Agones can also allocate an already allocated GameServer when player-capacity rules permit it.

These provider lifecycles differ materially. They also do not cover every topology: a player may enter an already-running MMO shard/zone, a peer-hosted session, a relay-backed topology, or a fixed authority endpoint.

Portable conclusion: the first core needs a generic **assignment/placement result**, not a universal `GameServer`, Fleet, pod, process, or dedicated-server allocation object.

Sources:

- [Agones GameServerAllocation](https://agones.dev/site/docs/reference/gameserverallocation/)
- [Agones player-capacity allocation](https://agones.dev/site/docs/integration-patterns/player-capacity/)
- [Agones GameServer lifecycle](https://agones.dev/site/docs/guides/client-sdks/)
- [GameLift game-session placement](https://docs.aws.amazon.com/gameliftservers/latest/developerguide/queues-intro.html)

### Matchmaking is an optional producer, not a universal prerequisite

Open Match's Assignment is associated with tickets after a match is formed, but GameLift can also place a game session directly without FlexMatch. PlayFab exposes direct multiplayer-server requests in addition to matchmaking-integrated allocation.

Persistent-world and MMO routing commonly starts from a known character/world/shard/zone policy rather than from a fresh matchmaking operation. Private invites and direct-host co-op also need not be matchmade.

Portable conclusion: a RunenOnline core that requires `MatchRequestId -> MatchId` before every gameplay assignment would encode one product topology as universal semantics.

Sources:

- [GameLift StartGameSessionPlacement](https://docs.aws.amazon.com/cli/latest/reference/gamelift/start-game-session-placement.html)
- [PlayFab Request Multiplayer Server](https://learn.microsoft.com/en-us/rest/api/playfab/multiplayer/multiplayer-server/request-multiplayer-server?view=playfab-rest)
- [Open Match Director](https://open-match.dev/site/docs/guides/matchmaker/director/)

## Revised minimum model

The investigation originally treated matchmaking as part of the universal chain. Critical review rejects that model because MMO zone entry, shard routing, direct invites, direct hosting, and non-matchmade co-op would have to manufacture meaningless match objects.

The smallest general gameplay-entry chain is instead:

```text
external authentication / trusted principal evidence
        │
        ▼
verified external principal
        │ resolve under local policy
        ▼
RunenOnline PlayerId
        │
        │ application/control-plane placement decision
        ▼
AssignmentId
        │
        │ issue narrowly scoped authority
        ▼
AdmissionGrant
        │
        │ enter the assigned gameplay authority
        ▼
game server / shard / peer authority / relay-backed authority
        │
        ├─ validate RunenOnline admission authority
        └─ independently negotiate/admit through RunenNet when RunenNet is used
                   │
                   ▼
              ParticipantId
```

`AssignmentId` here does **not** mean "match server". It identifies one placement/admission incarnation for an exact player or cohort and a currently authorized gameplay destination. The assignment may have been produced by matchmaking, world/shard routing, an invite, direct server selection, a pre-existing persistent zone, or another policy.

Matchmaking composes as an optional producer:

```text
PlayerId / exact cohort
        │
        ▼
MatchRequestId
        │
        ▼
MatchId + committed roster/result
        │
        └──────────────┐
                       ▼
                  AssignmentId
                       │
                       ▼
                  AdmissionGrant
```

This makes assignment/admission reusable across genres while keeping matchmaking semantics independently optional.

## Candidate semantic identities

These names are investigation proposals, not accepted public API names.

### Trust namespace / RealmId question

The first profile likely needs an explicit trust namespace so that dev/prod, different titles, tenants, or independently operated control planes cannot accept one another's identities or grants merely because local identifiers collide.

`RealmId` is one candidate spelling. It must not be inferred from a provider project ID, hostname, database name, deployment name, or RunenNet `SessionId`.

RO1A should decide whether an explicit realm identifier is required or whether an equivalent authority/issuer scope is sufficient. This remains open; the investigation must not freeze the name prematurely.

### PlayerId

`PlayerId` should identify the RunenOnline-local player/account incarnation used by control-plane policy.

It is distinct from:

- every external authentication principal;
- provider access/session tokens;
- matchmaking requests or matches;
- assignments;
- RunenNet `ParticipantId`;
- transport connections.

A PlayerId may be durable or guest/ephemeral according to the accepted profile. Its declared lifetime and non-reuse guarantee must be explicit. Link, unlink, merge, recovery, deletion, and provider-specific proof flows are separate policy/extension questions unless a first implementation proves they are required.

### AssignmentId

`AssignmentId` should identify one placement/admission incarnation that binds:

- an exact PlayerId or bounded player cohort;
- one currently authorized gameplay destination/admission target;
- any application-owned source context that justified the placement, without requiring a MatchId;
- one lifecycle incarnation so stale placement results cannot become current after reassignment.

The destination may be realized by:

- a newly allocated dedicated server;
- an already-running multi-match server;
- an MMO shard/zone;
- a relay-backed topology;
- a peer host;
- another application-defined authority topology.

Connection details such as IP addresses, ports, relay credentials, provider session IDs, process IDs, pods, fleets, VM IDs, or region labels are realization data. They are not AssignmentId.

If a destination becomes unusable and policy permits reassignment, a fresh AssignmentId must be created. Old results and admission grants must not silently retarget to the replacement.

### AdmissionGrantId / AdmissionGrant

`AdmissionGrant` should represent explicit, finite gameplay-admission authority for one PlayerId to realize one current AssignmentId under one trust/issuer context.

It must be distinct from:

- external provider/login credentials;
- AssignmentId itself;
- MatchRequestId or MatchId when matchmaking is involved;
- network endpoint identity;
- RunenNet `ParticipantId` or connection identity.

The first normative profile should resolve at least:

- exact subject/player binding;
- exact AssignmentId binding;
- explicit trust/issuer context;
- finite validity window;
- target/audience restriction sufficient to prevent redemption at an unrelated authority;
- explicit invalid/expired/revoked outcomes where the chosen realization exposes them;
- a bounded redemption policy that prevents one grant from creating multiple independent gameplay admissions;
- atomic resolution of concurrent redemption attempts;
- fresh authority after reassignment rather than implicit retargeting.

A grant is a semantic authority object, not necessarily a serialized bearer token. Realizations may use signed credentials, opaque random grants with introspection, sender-constrained proof, or another mechanism if the same contract is preserved.

Whether the first profile requires proof-of-possession/sender binding or permits a bearer-style grant with server-side replay tracking remains an explicit specification decision.

### MatchRequestId — optional matchmaking domain

`MatchRequestId` should identify one finite matchmaking-request incarnation when matchmaking is used. It is not required for direct assignment flows.

A request may contain an exact bounded cohort of PlayerIds. This supports "these players must be matched together" without requiring PartyId or LobbyId in the first profile.

The request needs finite lifetime, bounded attributes/cohort size, deterministic terminal resolution, and stale-result protection.

### MatchId — optional matchmaking domain

`MatchId` should identify one committed logical matchmaking result and its exact accepted roster/output.

It is not:

- an AssignmentId;
- a gameplay endpoint;
- a process or allocation;
- a RunenNet `SessionId`.

A MatchId may be one source of a later AssignmentId, but assignments may also exist with no MatchId at all.

## Minimum lifecycle contracts

### External principal resolution

Authentication or another trusted identity mechanism produces verified external-principal evidence. RunenOnline resolves that evidence to a PlayerId under explicit local policy.

Authentication success alone must not create gameplay assignment, gameplay admission, RunenNet membership, or transport authorization.

The core should not standardize OIDC/JWT/provider verification as its semantic format. Provider adapters may later produce verified evidence into this boundary.

### Assignment

Assignment is the first universal gameplay-entry lifecycle proposed by this investigation.

The portable contract should distinguish at least:

```text
Pending / not usable
        │
        ├─> Usable current assignment
        ├─> Cancelled
        ├─> Expired
        └─> Failed
```

These are conceptual classes, not accepted enum names.

A usable assignment must identify the exact authorized player/cohort and current destination/admission target. Provider-specific states such as `Ready`, `Allocated`, `Reserved`, `FULFILLED`, pod readiness, process registration, or shard-health states remain realization details.

If replacement is allowed, reassignment produces a fresh AssignmentId. A stale completion/result for an older assignment attempt must not replace a newer current assignment.

The first profile should model one current assignment per admitted placement intent/incarnation, not assume one assignment per logical match. Multi-zone transitions, shard transfer, or other complex routing can create successive assignment incarnations rather than forcing MMO topology into match semantics.

### Admission grant

A grant may be issued only for an authorized subject of a current usable AssignmentId.

Grant redemption authorizes application/gameplay admission according to the RunenOnline contract. If RunenNet is used, the application still independently satisfies RunenNet protocol negotiation and session admission and may then record a PlayerId-to-ParticipantId association. Neither identity becomes the other.

A successful grant redemption should represent one bounded gameplay-admission claim/incarnation, not necessarily one transport connection. RunenNet connection replacement/recovery remains a RunenNet/application concern. A wholly new RunenOnline gameplay admission requires fresh authority unless the future profile explicitly permits another bounded redemption model.

### Match request — optional matchmaking lifecycle

When matchmaking is used, a minimum request lifecycle needs semantic distinctions equivalent to:

```text
Pending
  ├─> Matched/Consumed
  ├─> Cancelled
  ├─> Expired
  └─> Failed
```

Only one terminal outcome may win for one MatchRequestId incarnation. Cancellation/expiry and match completion races need atomic resolution. Delayed results for an already terminal request must not revive it or consume a newer request incarnation.

Interactive acceptance, ready checks, queue expansion, backfill, skill formulas, and latency/team optimization are not required by the minimum request contract.

### Match formation — optional matchmaking lifecycle

A committed MatchId records the exact bounded request/player roster and accepted formation output. Match formation does not imply a usable gameplay destination and does not itself authorize connection.

The core should specify output correctness and stale-result behavior, not one matchmaking algorithm.

A committed match may request or influence a subsequent AssignmentId, but the assignment remains a separate lifecycle and identity.

## Resource and hostile-input invariants that should become normative

Portable correctness must not be delegated to PostgreSQL limits, Redis eviction, HTTP middleware, Kubernetes quotas, or provider defaults.

The first semantic slices should require, where the corresponding object exists:

- every remotely influenced retained collection has explicit finite count/size/retention policy;
- PlayerId/principal-association state has a declared finite or intentionally persistent lifetime with non-reuse rules;
- assignment requests/attempts, active assignments, terminal assignment evidence, and reassignment history needed for stale-result rejection are bounded;
- AdmissionGrant lifetime is finite;
- active/recent grant and replay-detection evidence is bounded;
- grant subject/cohort representation is bounded before proportional allocation;
- caller-controlled length/count claims are validated before allocation;
- retries cannot accumulate unbounded historical assignment or grant copies;
- identity generators fail before wrap/reuse can make stale artifacts current;
- cross-trust-domain and cross-assignment artifacts fail closed rather than falling back to the "current" object;
- when matchmaking is used, request lifetime, cohort cardinality, attribute representation, concurrent requests, active/retained requests, and retained terminal evidence are all bounded.

Exact numeric defaults remain implementation/profile policy unless interoperability later requires standardization.

## Idempotency, concurrency, and stale-result model

RunenOnline will cross service/provider boundaries where retries and delayed outcomes are ordinary. The semantic model must therefore not assume exactly-once RPC delivery.

Minimum principle:

> Repeating work for the same semantic incarnation may reproduce the already committed outcome, but it must not create a second semantic incarnation. A new attempt that may produce a different semantic outcome receives fresh identity.

Examples:

- retrying assignment creation/query/cancellation for the same incarnation must not create a second current assignment;
- a delayed provider allocation result for an obsolete AssignmentId cannot replace the current assignment;
- a reassignment after failure uses a fresh AssignmentId;
- duplicate/concurrent AdmissionGrant redemption cannot create multiple independent gameplay admissions beyond the grant's explicit bounded redemption policy;
- when matchmaking is used, a stale matched result for an expired/cancelled MatchRequestId cannot consume a newer request.

The transport-level idempotency key, database transaction, deduplication table, actor, queue, or RPC mechanism is realization detail. The retained evidence needed to make the promised idempotency/replay semantics true must itself be finite and explicit.

## Durable versus ephemeral state

The specification should define semantic lifetime and non-reuse, not database schema.

Likely minimum distinctions:

- PlayerId and accepted external-principal associations may outlive one gameplay admission and need an explicit declared lifetime/non-reuse contract;
- Assignment and AdmissionGrant are bounded control-plane-lifetime objects with terminal/retention semantics;
- MatchRequest and Match are bounded objects only when the matchmaking capability is used;
- long-term match history, replay archives, inventory/economy, rankings, guild/social graph, analytics, moderation, and event-store schemas are not prerequisites for the first core.

PostgreSQL or another database may realize these contracts later; table shape and transaction API are not normative authority.

## Concepts deferred from the minimum core

### Party and lobby

Party/lobby lifecycle is not required to prove identity, assignment, or admission. Matchmaking can initially accept an exact cohort directly. Persistent parties, invitations, leader transfer, lobby discovery, ready checks, presence, chat, join-in-progress policy, and roster UI should remain separate capabilities until evidence requires them.

### Matchmaking as a universal dependency

Rejected. Matchmaking is a major RunenOnline capability, but it is not a prerequisite for MMO zone routing, direct invites, direct-host co-op, pre-existing shards, or explicitly selected servers.

### Matchmaking algorithm

Skill/MMR formulas, queue search, expansion rules, latency optimization, team balancing, backfill, region choice, and match quality remain policy/strategy. The portable capability should own request/result correctness and lifecycle, not one algorithm.

### Dedicated-server allocator ontology

Agones Fleet/GameServer/GameServerAllocation, GameLift fleet/queue/placement, PlayFab build/session, Kubernetes pods, VM/container identities, and cloud region names remain adapter/infrastructure concepts. Assignment is the portable boundary; allocation is one possible producer/realization.

### Universal game-session identity

Rejected. A persistent MMO zone, a peer-hosted fighter, an FPS match, and thousands of card matches multiplexed in one process should not be forced into one process/session identity model. RunenNet separately owns its own SessionId semantics when used.

### General persistence platform

RunenOnline should not standardize SQL, document storage, transactions, event sourcing, caching, or migration tooling merely to define semantic lifetime.

### Social/progression/economy

Friends, guilds/groups, chat, inventory, purchases, currencies, progression, MMR storage, leaderboards, tournaments, notifications, moderation, and LiveOps are valuable later capabilities but are not prerequisites for gameplay assignment/admission.

### Game simulation and realtime networking

Authoritative gameplay state, tick scheduling, ECS/world access, replication, prediction, reconciliation, lag compensation, transport delivery, and RunenNet `SessionId`/`ParticipantId` remain outside RunenOnline's control-plane core.

## Competing models reviewed

### One "online session" identity for everything

Rejected. Authentication identity, assignment, admission authority, matchmaking request/result, hosting destination, RunenNet session, participant, process, and transport connection have different lifetimes and stale-artifact risks.

### Provider account ID as PlayerId

Rejected. OIDC subject values are issuer-scoped and providers use different identity models. RunenOnline-local player identity must remain independently scoped.

### Matchmaking-first universal pipeline

Rejected after critical review. It works for FPS/card-game queues but forces fake MatchRequest/Match objects into persistent worlds, direct invites, shard routing, and other non-matchmade flows.

### MatchId equals game-server/session ID

Rejected as portable law. PlayFab demonstrates that this can be a convenient realization; Open Match, GameLift, and Agones demonstrate separable match and placement/allocation lifecycles.

### Mandatory Party/Lobby before matchmaking

Rejected for the first capability. An exact bounded cohort on a matchmaking request can represent "match together" without adding persistent social/presence lifecycle.

### Dedicated server as universal assignment target

Rejected. It excludes relays, peer hosts, long-lived shards/zones, already-running multiplexed servers, and future authority topologies.

### Provider token as gameplay admission token

Rejected. Login/provider credentials normally have different audience, privilege, lifetime, revocation, and replay properties. Gameplay admission requires its own narrow authority boundary.

### Endpoint as assignment identity

Rejected. Endpoints may rotate, one process may host many assignments, one assignment may be rerouted, and relay/session credentials may change. Endpoint/provider details are realization data, not semantic incarnation identity.

## Proposed normative slice decomposition

The corrected dependency structure is smaller and more general than the original draft.

### RO1A — Player identity, trust scope, and base lifetime/resource law

Specify only the foundation needed by all later control-plane capabilities:

- whether an explicit RealmId/trust-domain identity is required or an equivalent issuer/authority scope is sufficient;
- PlayerId and verified-external-principal association boundary;
- identity incarnation/non-reuse/scoping rules;
- host-supplied or implementation-supplied identity generation requirements without standardizing one numeric format;
- time/deadline vocabulary needed for finite lifetimes;
- base resource-bound and stale-artifact principles shared by later objects.

Do not define matchmaking, assignment, or grants beyond relationships necessary to make identity/trust ownership unambiguous.

### RO1B — Assignment and admission authority

Specify the first universal gameplay-entry capability:

- AssignmentId identity and exact subject/cohort binding;
- provider-neutral pending/usable/terminal assignment boundary;
- reassignment through fresh identity and stale-result rejection;
- provider-neutral destination/realization boundary without standardizing server/process/endpoint ontology;
- AdmissionGrant subject/assignment/trust/audience/lifetime contract;
- bounded redemption, replay, concurrent-redemption, expiry/revocation semantics;
- fresh grants after reassignment;
- explicit handoff to game/application authority and independent RunenNet negotiation/admission when RunenNet is used;
- bounded assignment/grant/replay evidence.

This slice must support direct placement and persistent-world routing without MatchId.

### RO1C — Optional matchmaking request and match formation capability

Only after RO1A and the assignment destination contract are clear, specify matchmaking as one producer of assignments:

- finite MatchRequestId lifecycle;
- exact bounded cohort and bounded attribute envelope;
- cancellation/expiry/failure/matched atomic terminal behavior;
- stale-result/idempotency rules;
- MatchId and exact committed roster/result relationship;
- MatchId-to-Assignment creation/association boundary without equating the identities;
- provider-neutral algorithm boundary;
- bounded active/retained request and match evidence.

Do not introduce party/lobby or one matchmaking algorithm.

### RO1D — Conformance/profile closure only if independently required

After RO1A-C, determine whether a separate normative conformance/profile owner is actually necessary. Do not create RO1D for symmetry with RunenNet. If the specification index and slice-local conformance obligations are sufficient, defer profile taxonomy to RO2 assurance work.

## Recommended dependency order

```text
RO1A player identity / trust
          │
          ▼
RO1B assignment / admission
          │
          ├──────────────> direct invite / direct host / persistent world / MMO routing
          │
          ▼
RO1C matchmaking capability
          │
          └──────────────> MatchRequest -> Match -> Assignment
          │
          ▼
RO1 gate review
          └─ optional RO1D only if separate conformance ownership is demonstrated
```

RO1B depends only on the identity/trust/lifetime foundation. RO1C depends on RO1A and composes its result into the already-defined assignment boundary. Matchmaking therefore cannot become a hidden prerequisite for every RunenOnline consumer.

## Remaining explicit questions

The investigation is decision-complete enough to begin RO1A, but these items must remain explicit specification questions rather than being guessed from provider behavior:

1. whether the trust root is represented by `RealmId`, issuer identity, or another narrower authority scope;
2. the exact declared lifetime/non-reuse model for PlayerId;
3. whether assignments bind exactly one PlayerId or allow a bounded cohort in the first profile;
4. the minimum portable representation of assignment destination/audience without standardizing endpoints;
5. whether the first AdmissionGrant profile requires sender-constrained proof or permits a bearer realization with strong bounded replay tracking;
6. whether one grant authorizes exactly one gameplay-admission incarnation or another explicitly bounded redemption policy;
7. what retained terminal evidence is minimally required for stale-result, replay, and idempotency guarantees;
8. whether MatchId needs any lifecycle beyond immutable committed result + bounded retention in the first matchmaking profile.

None of these require selecting PostgreSQL, Redis, HTTP/gRPC, OIDC/JWT, Nakama, PlayFab, Agones, GameLift, Open Match, RunenNet adapters, or Runenwerk integration before specification.

## Investigation conclusion

RunenOnline should **not** begin as a catalog of auth/lobby/matchmaking/allocation services, and it should not assume that every online game is matchmade onto one dedicated server.

The minimum portable gameplay-entry core is:

> **verified principal resolution -> RunenOnline PlayerId -> current gameplay AssignmentId -> narrowly scoped AdmissionGrant**

Matchmaking is the first major optional producer layered on top:

> **MatchRequestId -> committed MatchId -> AssignmentId**

This corrected model fits queue-based card/FPS games, direct co-op, peer/relay fighting games, multiplexed servers, survival/extraction sessions, and MMO shard/zone placement without collapsing account identity, logical match identity, placement identity, RunenNet session identity, participant identity, process identity, or transport identity.

The first normative work should therefore be **RO1A identity/trust**, followed by **RO1B assignment/admission**, then **RO1C matchmaking**. Parties/lobbies, dedicated-server allocation vocabulary, general persistence abstractions, social/economy features, and concrete provider integrations remain deferred until accepted evidence demonstrates a portable need.
