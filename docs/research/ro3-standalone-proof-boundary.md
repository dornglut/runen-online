# RO3 standalone proof boundary

Status: **non-normative investigation**

This record supports issue #24 under RO3 parent #23. It does not define RunenOnline semantics or public API stability. The accepted `spec/` tree remains the authority for portable behavior; `ARCHITECTURE.md` owns repository/package boundaries.

Accepted investigation base: `5541925ce1c4ad10e229ea6e6466fa4f60b39bf6`.

## Question

What is the smallest truthful plain-Rust consumer proof that demonstrates ordinary use of the accepted RO2 core without adding a provider, service, runtime, sibling-framework dependency, or speculative package boundary?

## Current evidence

RO2 leaves one dependency-free product library crate at `crates/runen-online`.

Its public surface already exposes the pieces a plain Rust host needs for the RO3 proof:

- authority-domain and trusted-time construction handles;
- explicit finite `AuthorityLimits`;
- `Authority` construction with a host-configured trusted external-authority set;
- PlayerId creation plus verified-principal acceptance, association, and resolution;
- direct Pending and Usable Assignment operations;
- logical destination handles supplied by the host;
- AdmissionGrant issuance, redemption, and observation;
- optional MatchRequest establishment, Match commit, and immutable Match observation;
- typed states/outcomes/errors sufficient for deterministic proof assertions.

The core does not need a consumer-facing API change to express the proof scenarios below.

Repository validation already runs `cargo test --workspace --all-targets --locked` and Clippy over all targets. A Cargo example therefore participates in the repository build/lint gate. To make canonical validation execute the proof behavior rather than merely compile its `main`, the example can expose one local `run_proof()` function and include an example-local `#[cfg(test)]` test that calls the same function. The normal human-facing command calls `run_proof()` from `main`.

## Competing proof shapes

### 1. Cargo example in the existing product package — selected

Shape:

```text
crates/runen-online/examples/standalone.rs
```

Why it is sufficient:

- a Cargo example is a separate target and consumes `runen_online` through the library's public Rust surface;
- it cannot rely on the product crate's private modules merely because it lives in the same package;
- it is directly runnable as ordinary Rust code;
- it adds no second package, workspace member, dependency edge, deployment unit, or service identity;
- the existing all-target validation automatically compiles, tests, and lints the target when it contains an example-local test;
- it keeps the proof visibly downstream of the semantic implementation while remaining repository-local and deterministic.

This is enough for RO3. RO3 asks for ordinary plain-Rust consumption, not proof of an independently versioned/deployed package boundary.

### 2. Separate workspace consumer package/application — rejected for RO3

A second package such as `examples/standalone/Cargo.toml` or `apps/standalone-proof` could depend on `crates/runen-online` by path and would prove an additional Cargo package boundary.

That extra boundary is not required by the RO3 gate. It would add workspace/package topology, root-manifest ownership, and a new package identity without a current reuse/deployment need. Nothing in the accepted semantics requires it.

Use a separate package later only if a real consumer or integration demonstrates an independent package boundary.

### 3. Integration test only — rejected as the explicit proof artifact

RO2 integration tests already demonstrate public API access and semantic correctness. Adding another integration test would mostly duplicate assurance and would not provide the explicit runnable consumer-facing proof requested by RO3.

The selected example should reuse deterministic assertions, not duplicate the full RO2 conformance matrix.

### 4. External repository, game server, RunenNet, or Runenwerk host — rejected for RO3

Those choices pull RO5 composition or product/game integration forward and risk making sibling/framework behavior proof authority. They are deliberately later roadmap work.

## Selected host boundary

The example is an ordinary host. It supplies policy and open representations explicitly rather than hiding them in RunenOnline.

The host owns and supplies:

- a fresh local `AuthorityDomainHandle`;
- a fresh local `TimeDomainHandle` and explicit trusted observations;
- every numeric `AuthorityLimits` value used by the proof;
- the trusted external-authority set configured at authority construction;
- already-verified external-principal authority/subject evidence;
- whether and when PlayerIds are created or associated;
- whether an Assignment starts directly Usable or Pending;
- the logical gameplay destination chosen for an Assignment;
- MatchRequest cohorts and opaque matching-input bytes;
- which exact MatchRequests form a candidate Match;
- whether and how an observed Match later causes the host to establish an Assignment;
- which matched players receive AdmissionGrants and when trusted policy Ends an Assignment.

The example must state that its Match-to-Assignment and grant choices are **example host policy only**, not portable cardinality/workflow semantics.

The proof must not implement OAuth/OIDC/JWT verification, matchmaking selection algorithms, server allocation, connection material, persistence, timers, networking, or gameplay membership.

## Minimum proof scenarios

One example target should run two bounded scenarios. They are consumer demonstrations, not normative workflow definitions.

### Scenario A — direct non-matchmade path

1. Construct one bounded authority using host-owned domain/time handles, limits, and trusted issuer configuration.
2. Create a PlayerId.
3. Accept host-verified external-principal evidence, associate it with the player, and verify deterministic resolution.
4. Establish an Assignment directly Usable at a host-chosen logical destination.
5. Issue an AdmissionGrant for the player and Assignment with an explicit finite deadline.
6. Redeem the grant using an explicit trusted observation and assert one successful semantic redemption.
7. End the Assignment and assert the already-redeemed grant remains Redeemed.

This proves that direct Assignment/Admission consumption does not require MatchRequest or Match state.

### Scenario B — optional matchmaking plus host-owned composition

1. Create two host-authorized players.
2. Establish one finite MatchRequest for each using host-chosen cohorts, opaque matching inputs, and deadlines.
3. Have the **host** choose the exact candidate request set and commit one Match.
4. Observe the immutable Match and assert its exact roster/contributions.
5. As an explicit example-only policy decision, have the host establish one Pending Assignment after seeing the Match.
6. Supply a later trusted observation and host-chosen logical destination to resolve that Assignment.
7. Issue one AdmissionGrant for each matched player.
8. Redeem the first grant successfully, then End the Assignment before redeeming the second.
9. Assert the second redemption fails as not usable/current while the first grant remains Redeemed.

This demonstrates optional matchmaking composition, host-owned Match-to-Assignment policy, trusted placement/time input, Assignment currentness, and committed-redemption stability without introducing gameplay/session semantics.

## Proof execution

The selected example should have one internal function such as:

```text
run_proof()
```

Both entry paths call the same implementation:

- `main()` calls `run_proof()` and may print a short success message;
- one `#[cfg(test)]` unit test in the example calls `run_proof()` so `cargo test --workspace --all-targets --locked` executes the proof behavior.

The proof should use deterministic fixed local values and assertions. It should require no environment variables, files, sockets, clock reads, random numbers, threads, async executor, or external services.

Focused developer checks:

```text
cargo run -p runen-online --example standalone
cargo test -p runen-online --example standalone
```

Acceptance remains the repository-owned:

```text
cargo validate
```

## Architecture impact

No `ARCHITECTURE.md` change is required for the selected delivery.

The example is a Cargo target inside the already accepted `crates/runen-online` product package. It does not add a product crate, application package, service package, provider package, deployment obligation, or reusable integration package.

If implementation instead demonstrates that a separate package is necessary, that is a changed package-ownership finding and must return to accepted authority rather than silently expanding RO3 delivery.

## Consumer-surface finding

The current RO2 public Rust surface is sufficient for the proof. RO3 does not currently justify changing product APIs.

If the delivery cannot implement the scenarios above using only public exports, treat that as a reproduced consumer-surface gap. Correct only the minimum gap through the RO3 delivery authority; do not turn it into general API redesign or stabilization.

## Decision-complete RO3 delivery slice

The next delivery should:

1. add exactly `crates/runen-online/examples/standalone.rs`;
2. use only the existing `runen-online` crate and Rust standard library;
3. implement Scenario A and Scenario B as deterministic host-owned proof code;
4. call the same proof function from `main()` and one example-local test;
5. add no new workspace member or dependency;
6. leave normative `spec/` unchanged;
7. leave `ARCHITECTURE.md` unchanged unless implementation reproduces a package-boundary contradiction;
8. validate focused run/test commands and exact-head `cargo validate`;
9. critically review that the example does not imply Match-to-Assignment cardinality, player roster ownership by Assignment, credential/provider behavior, or mandatory matchmaking.

## Delivery acceptance checks

The RO3 proof delivery is acceptable when:

- `cargo run -p runen-online --example standalone` completes successfully;
- canonical validation executes the example-local proof test and passes on the exact reviewed head;
- the direct scenario creates no MatchRequest/Match dependency;
- the optional matchmaking scenario makes candidate selection and Match-to-Assignment composition visibly host-owned;
- principal verification input, trusted time, destination, matching inputs, and finite limits are explicitly supplied by the host;
- currentness failure is handled deterministically without external runtime/provider behavior;
- the proof uses only public `runen_online` exports;
- no product API, package, provider, service, runtime, or normative semantic expansion is introduced unless a reproduced blocker requires separately reviewed correction.

## Conclusion

RO3 does not need another product or application package. The minimum truthful standalone proof is one executable/tested Cargo example in the existing `runen-online` package. That target is sufficient to prove ordinary public-surface consumption while keeping host policy explicit and leaving provider, game-server, RunenNet, Runenwerk, and production realization work for their accepted later stages.