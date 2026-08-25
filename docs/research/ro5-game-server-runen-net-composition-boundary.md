# RO5 game-server and RunenNet composition boundary

Status: **non-normative investigation**

This record supports issue #34 under RO5 parent #33. It does not define RunenOnline or RunenNet semantics. Portable behavior remains owned by each repository's accepted `spec/` tree and Dornglut Engineering ADR 0006 owns the cross-repository boundary.

RunenOnline base: `9dddeb13edd548a9366b3a3dd862c877305142e0`.
RunenNet evidence base: `79a455945fe85933cbd2a74145a20f296f86f55c`.

## Question

What is the smallest executable game-server proof that consumes both RunenOnline and RunenNet while keeping their identity, admission, session, transport, and lifecycle domains explicit and independent?

## Decision

RO5B should add one **non-publishable top-level workspace proof package**:

```text
examples/runen-net-composition/
    Cargo.toml
    src/main.rs
```

The proof package is the game-server/application boundary. It depends on both frameworks; neither production core depends on the other.

Dependency direction:

```text
runen-online-runen-net-proof
    -> local runen-online
    -> runen-net pinned to one exact Git revision
```

The package must set `publish = false` and must not be treated as a reusable production integration crate or semantic owner.

`crates/runen-online/Cargo.toml` must remain free of RunenNet dependencies, including dev-dependencies added only to make the proof convenient. No RunenNet repository change is required.

## Why a top-level proof package

A crate-local Cargo example would require `runen-net` to become a development dependency of the `runen-online` product package. That would make a sibling-framework dependency part of the core package manifest even though the core does not need RunenNet.

A new production integration crate would overstate the evidence. RO5 currently needs one executable consumer proof, not a reusable integration library with a stabilized API or independent release contract.

A top-level package under the already-reserved `examples/` area expresses the correct ownership:

- it is an ordinary downstream consumer;
- its dependencies are independent of product-package dependencies;
- it can be compiled and tested by workspace validation;
- it creates no production package taxonomy;
- it can later be replaced by a real game-server application without compatibility obligations.

## RunenNet dependency pin

RO5B should consume RunenNet from its Git repository at the exact investigated revision:

```text
rev = "79a455945fe85933cbd2a74145a20f296f86f55c"
```

The resulting `Cargo.lock` must retain the resolved exact source revision. This avoids assuming that RunenNet is published to a registry and avoids a moving `main` dependency.

Advancing the proof to a later RunenNet revision is separate maintenance evidence; the proof must not silently follow branch movement.

## Exact composition sequence

The proof should make this host-owned sequence explicit:

```text
1. Host establishes RunenOnline PlayerId / Assignment / AdmissionGrant.
2. A transport connection exists as a RunenNet ConnectionHandle.
3. RunenNet protocol negotiation reaches Established for that connection.
4. The RunenNet Session still has no participant membership.
5. Host presents/resolves the RunenOnline AdmissionGrant to the owning Authority.
6. RunenOnline commits one successful redemption for the bound PlayerId + AssignmentId.
7. The RunenNet Session still has no participant membership.
8. Host game-server policy accepts that redemption as sufficient admission evidence.
9. Host allocates a distinct RunenNet ParticipantId.
10. Host records any application-local mapping it needs.
11. Host calls RunenNet Session::admit_new with the ParticipantId and EstablishedNegotiation.
12. RunenNet creates/binds the participant membership.
```

Steps 8-10 are application policy. Neither framework standardizes them.

The sequence deliberately permits protocol negotiation before gameplay admission. RunenNet already specifies that transport establishment and protocol compatibility do not themselves create participant membership, while RunenOnline specifies that grant redemption does not itself create RunenNet membership.

## Identity mapping

The proof may use a small application-local structure equivalent to:

```text
HostMembership {
    player: RunenOnline PlayerId,
    assignment: RunenOnline AssignmentId,
    session: RunenNet SessionId,
    participant: RunenNet ParticipantId,
}
```

This structure is proof-local application state only.

It does **not** establish:

```text
PlayerId == ParticipantId
AssignmentId == SessionId
```

It also does not establish universal one-to-one cardinality, derivation, shared serialization, shared allocation, or cross-session persistence. The host supplies all RunenNet identities under RunenNet's own non-reuse rules.

## Admission authority separation

A successful RunenOnline redemption is one gameplay-entry authorization event for a specific PlayerId and AssignmentId. It is evidence used by host policy; it is not itself a RunenNet admission operation.

The proof must show observable state between the two operations:

```text
redemption committed
AND
RunenNet live membership count == 0
```

Only the subsequent explicit `Session::admit_new` may create the new RunenNet participant membership.

The reverse separation should also be visible: an Established RunenNet protocol negotiation alone leaves RunenOnline grant state untouched and creates no RunenOnline identity or assignment authority.

## Failure ordering and non-transactional boundary

RO5B should prove a failed pre-redemption path:

```text
Assignment ends before grant redemption
-> grant cannot redeem successfully
-> host does not call Session::admit_new
-> RunenNet session remains without that participant membership
```

RO5B must also prove the opposite failure ordering because redemption and network admission are **two distinct semantic commits**:

```text
RunenOnline redemption succeeds
-> RunenNet Session::admit_new fails
-> RunenOnline redemption remains committed
-> grant does not become redeemable again
-> no synthetic rollback is applied to either framework
```

The admission failure may be induced deterministically by an already-used ParticipantId, a full membership partition, or another existing RunenNet rejection. The proof should choose the smallest case that does not require changing either framework.

The host may perform ordinary prechecks to reduce such failures, but RO5 must not imply that prechecks create atomicity across the two frameworks. A failed RunenNet admission after successful redemption is an application-level partial-progress condition that later production integration may need to recover or compensate according to separately accepted policy.

RO5 therefore must not invent a shared transaction, rollback hook, or cross-framework commit protocol. A later remote-service realization must separately address ambiguous failure between these two semantic commits if required.

## Post-admission lifecycle separation

After a successful redemption and RunenNet admission:

- AdmissionGrant expiry does not invalidate the already-committed redemption;
- Assignment ending does not automatically remove the RunenNet membership;
- participant removal/session closure remains RunenNet authority or explicit host policy;
- a game may choose to react to Assignment end, but that reaction is not implied by either framework.

RO5B should prove this by ending the Assignment after admission and asserting that the RunenNet participant remains Bound.

## Connection loss and reconnect/rebind

The first RunenNet profile already owns connection-loss retention and explicit connection replacement.

RO5B should prove:

```text
successful redemption
-> RunenNet admission
-> connection loss with RetainForRecovery
-> participant becomes Unbound
-> Assignment may end
-> replaying the one-shot AdmissionGrant does not create new authorization
-> a second connection completes RunenNet protocol negotiation
-> explicit RunenNet bind_replacement restores the same ParticipantId
```

The rebind is authorized by host/RunenNet recovery policy, **not by reusing the AdmissionGrant**. The proof therefore must not introduce a reconnect token or reinterpret `AlreadyRedeemed` as reusable entry authority.

If the host instead chooses `Terminate`, a later fresh participant admission is a new host decision and may require fresh RunenOnline authorization according to application policy; RO5B does not standardize that policy.

## Protocol/transport scope

No live socket, QUIC adapter, network process, or RunenNet transport crate is required.

The proof should use the transport-independent RunenNet core public surface:

- `ConnectionHandle` as a local transport-lifetime handle;
- `NegotiationManager` and compatible offers to establish protocol negotiation;
- `Session` for membership/binding lifecycle.

This is sufficient because RO5 proves semantic composition at the game-server boundary, not production transport operation.

## Deterministic RO5B assurance

The executable proof should contain testable functions and cover at minimum:

1. **Successful explicit handoff**
   - protocol negotiation establishes before membership;
   - valid RunenOnline grant redeems;
   - redemption alone creates no RunenNet membership;
   - host allocates ParticipantId and explicitly admits it;
   - proof-local mapping retains exact distinct IDs.

2. **Failed authorization does not admit**
   - Assignment ends before redemption;
   - redemption fails;
   - no participant membership is created.

3. **Admission failure does not roll back redemption**
   - redemption commits successfully;
   - RunenNet admission is made to fail deterministically;
   - grant remains Redeemed/AlreadyRedeemed;
   - no participant membership is created by the failed admission.

4. **Post-admission independence**
   - Assignment ends after successful redemption/admission;
   - redeemed grant remains redeemed;
   - RunenNet participant remains Bound until explicit networking/game policy changes it.

5. **Recovery is RunenNet lifecycle, not grant replay**
   - connection loss retains membership;
   - replay of the grant reports the already-committed redemption rather than fresh authorization;
   - replacement connection negotiates;
   - `bind_replacement` rebinds the same ParticipantId without a second grant redemption.

6. **No hidden framework mutation**
   - protocol negotiation alone does not alter RunenOnline state;
   - RunenOnline redemption alone does not alter RunenNet state.

The proof should expose one `run_proof()` used by both `main` and a package-local test so canonical workspace `--all-targets` validation executes the scenario.

## Package and documentation changes permitted in RO5B

RO5B may change only what is required to register and explain the proof:

- root `Cargo.toml` workspace membership;
- `Cargo.lock`;
- `examples/runen-net-composition/Cargo.toml`;
- `examples/runen-net-composition/src/main.rs`;
- minimal `ARCHITECTURE.md` wording recording the non-production proof artifact if review shows the existing future-`examples/` wording is insufficient.

It must not change:

- `crates/runen-online` product source or manifest;
- `crates/runen-online-oidc`;
- normative `spec/` files;
- RunenNet repository source/spec;
- Runenwerk.

## Explicit non-goals

RO5B does not define:

- a universal game-server interface;
- connect-ticket or AdmissionGrant presentation format;
- remote redemption/introspection transport;
- distributed atomicity between redemption and network admission;
- cross-framework rollback/compensation semantics;
- PlayerId-to-ParticipantId or AssignmentId-to-SessionId serialization/cardinality;
- reconnect credential semantics;
- production QUIC/socket behavior;
- server allocation;
- persistence;
- Runenwerk integration.

## Conclusion

No new RunenOnline or RunenNet semantic primitive is required for RO5B.

The minimum truthful proof is a dedicated downstream example package that depends on both existing public cores and makes the application-owned bridge visible. That shape proves the organization boundary in ADR 0006 without turning either sibling framework into the other's semantic or package authority.