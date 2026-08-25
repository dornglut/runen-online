# RO4 first production realization boundary

Status: **non-normative investigation**

This record supports issue #29 under RO4 parent #28. It does not define RunenOnline semantics. The accepted `spec/` tree remains the sole authority for portable behavior; provider protocols, database behavior, package choices, Rust APIs, dependency versions, and failure types below are realization evidence only.

Accepted investigation base: `66ac66c615f3b96b8eb36b6ede5831e004c1cd4c`.

## Question

What is the smallest first production realization justified by the proven RunenOnline core, and what package/state/failure boundary lets that realization add real production capability without making one provider, protocol, storage mechanism, service transport, or the RO2 in-memory container semantic authority?

## Decision

The first RO4 delivery should be a **bounded OpenID Connect Core ID-token verification adapter** in one new optional crate:

```text
crates/runen-online-oidc
```

The adapter verifies an already-obtained OIDC ID token against trusted host configuration and then crosses the existing RunenOnline verified-external-principal boundary. It does not perform browser login, authorization-code exchange, discovery/network fetching, session management, access-token authorization, or account persistence.

This is the smallest production capability with an already-accepted semantic handoff. It requires no new normative specification and no change to the provider-neutral `runen-online` core API.

The other RO4 candidates remain deferred rather than rejected permanently.

## Why authentication verification is first

### 1. The semantic boundary already exists

`spec/core/identity.md` explicitly defines a trusted host/provider-adapter boundary for verified external-principal evidence. It also requires the adapter's subject identity semantics to prevent one external-principal value from silently being reassigned while a RunenOnline association relying on it remains valid.

OIDC Core provides a direct realization of that requirement for its ordinary subject profile:

- the issuer (`iss`) scopes the identity authority;
- the subject (`sub`) is locally unique and never reassigned within the issuer;
- ID-token validation includes issuer, audience, expiration, and signature checks;
- nonce verification can bind the authentication result to the host's OIDC flow.

The first adapter maps the **exact configured issuer URL string** to RunenOnline external-authority bytes and the verified `sub` value to external-subject bytes. The owning `Authority` must still recognize that issuer representation in its trusted external-authority configuration. Cryptographic verification therefore cannot silently create RunenOnline trust.

Sources checked for this investigation:

- [OpenID Connect Core 1.0](https://openid.net/specs/openid-connect-core-1_0-18.html)
- [`openidconnect` 4.0.1 documentation](https://docs.rs/openidconnect/4.0.1/openidconnect/)
- [`IdTokenVerifier` 4.0.1 documentation](https://docs.rs/openidconnect/4.0.1/openidconnect/struct.IdTokenVerifier.html)

The OpenID Foundation is currently reviewing an **Ephemeral Subject Identifier** extension. That profile is deliberately outside the first adapter because RO4B must rely only on subject identities suitable for durable RunenOnline principal association; it must not infer that every future OIDC subject profile satisfies the existing non-reassignment requirement.

### 2. It has a real independent dependency boundary

The provider-neutral core is currently std-only. OIDC verification necessarily brings protocol parsing and cryptographic dependencies that ordinary RunenOnline consumers do not need.

A separate `runen-online-oidc` crate therefore has concrete package evidence:

```text
runen-online-oidc
    -> runen-online
    -> std
```

The dependency direction MUST NOT reverse. Applications that do not use OIDC keep the std-only semantic core and pay no authentication dependency cost.

The selected first delivery uses `openidconnect` **without default HTTP features**. The current crate supports constructing an ID-token verifier directly from a client ID, issuer and JSON Web Key Set, and allows an explicit verification-time function. Network discovery is therefore not required to prove the authentication boundary.

### 3. It is independently useful

A production host that already obtains and refreshes OIDC provider metadata/JWKS can use this adapter to convert a successfully verified ID token into RunenOnline verified-principal evidence. The host can then resolve or associate the principal through the existing core.

The adapter is useful without persistence, service transport, matchmaking, allocation, RunenNet, Runenwerk, or an engine. Those concerns do not have to be manufactured to justify the first production capability.

## Why durable persistence is not first

Durable persistence is important, but the current implementation does not expose a truthful small storage adapter boundary.

RO2 deliberately realizes semantic identity using process-local authority-domain capability identity and trusted time using a process-local comparison-domain handle. The normative specs permit production realizations to preserve domain/time context out of band and do not require a public serialized domain or timestamp representation. Therefore this is **not a normative gap**.

It is, however, a substantial implementation-realization gap:

- persisted semantic IDs must recover enough authority-domain context after restart to preserve equality/non-reuse truth;
- ID allocation state must survive restart without wrap/reuse;
- deadline values must remain comparable under a declared trusted time realization without confusing provider/database time with RunenOnline semantic time;
- principal uniqueness, AdmissionGrant single redemption/currentness, and multi-request Match commit require durable atomicity/concurrency rules;
- the current in-memory maps/indexes are implementation structure, not a database schema;
- wrapping the current aggregate in a generic `Repository` or transaction trait would either leak the in-memory topology or underspecify the semantic atomic operations.

PostgreSQL, for example, offers transactions, locking, and serializable isolation, so a conforming durable implementation is technically plausible. But database capability does not decide the RunenOnline storage operation boundary. A later persistence investigation should start from the semantic atomic operations and restart guarantees, then select storage/package topology.

Evidence:

- [PostgreSQL current transaction processing documentation](https://www.postgresql.org/docs/current/transactions.html)
- [PostgreSQL current transaction isolation documentation](https://www.postgresql.org/docs/current/transaction-iso.html)

## Why service transport is not first

A service transport requires a service interface and message representation. gRPC, for example, normally begins with an explicit service definition and payload schema, and RPC completion can be ambiguous across the client/server boundary: one side can report failure after the other has already committed work.

RunenOnline intentionally has no accepted wire/storage/service schema yet. Exposing the RO2 `Authority` methods over gRPC or HTTP now would make provisional Rust method/error/container shapes pressure the first remote API before production state/idempotency boundaries are established.

A later service realization must start from accepted remote operation/idempotency semantics rather than serializing the current in-process API by symmetry.

Evidence:

- [gRPC core concepts and RPC lifecycle](https://grpc.io/docs/what-is-grpc/core-concepts/)

## Why server allocation is not first

The current Assignment contract deliberately uses an application-defined logical destination rather than a universal dedicated-server resource.

Current Agones evidence reinforces that separation. `GameServerAllocation` atomically selects an eligible GameServer and returns provider-specific resource/address/port metadata; Agones also documents eventual-consistency behavior around selectable allocation state. Those are valid provider mechanics, not portable Assignment identity or connection-material semantics.

RunenOnline has not accepted a provider-resource/connection-material representation or a production mapping from provider allocation results to logical destinations. Building an Agones adapter now would therefore either expose raw provider identities as the destination contract or require additional application/game-server integration decisions that belong to later accepted work.

Evidence:

- [Agones GameServerAllocation](https://agones.dev/site/docs/reference/gameserverallocation/)
- [Agones allocator service](https://agones.dev/site/docs/advanced/allocator-service/)

## Selected package and dependency boundary

RO4B should add exactly one new production adapter crate:

```text
crates/runen-online-oidc/
    Cargo.toml
    src/lib.rs
```

and update the workspace/lockfile plus `ARCHITECTURE.md` only enough to record this justified optional adapter package and dependency direction.

Direct production dependencies should be limited to:

- local `runen-online`;
- `openidconnect` 4.0.1 with `default-features = false` so the first adapter owns no HTTP client, async executor, discovery fetcher, or redirect policy;
- `serde_json` for bounded parsing of host-supplied JWKS JSON;
- `chrono` only as an internal bridge from host-supplied `std::time::SystemTime` to the OIDC library's deterministic expiration-time verifier.

No `reqwest`, Tokio, Axum, tonic, database client, provider SDK, or RunenNet/Runenwerk dependency is justified by this slice.

The OIDC dependency is an implementation decision. OIDC is not required for RunenOnline conformance.

## Selected verification profile

The first adapter intentionally supports a narrow production profile rather than every OIDC feature.

### Trusted configuration

A verifier configuration contains:

- exact expected OIDC issuer URL;
- exact expected OIDC client ID/audience;
- finite maximum raw ID-token bytes;
- finite maximum JWKS JSON bytes;
- finite maximum retained JWK count.

Construction receives bounded JWKS JSON from the trusted host. The adapter performs no discovery/network fetch. Empty/oversized/malformed configuration or an empty/oversized key set fails before verifier activation.

Provider metadata/JWKS acquisition, HTTPS policy, caching and key-refresh scheduling remain host responsibilities in RO4B. Replacing one verifier with another constructed from refreshed trusted JWKS is permitted application configuration; the adapter itself retains no background refresh/retry queue.

### Cryptographic/token profile

RO4B supports:

- signed OIDC Core ID tokens;
- **RS256 only** (`RsaSsaPkcs1V15Sha256`) in the first slice;
- signature verification always enabled;
- exact issuer match required;
- exact client audience match required;
- exactly **one** audience value, equal to the configured client ID;
- expiration validation at an explicit host-supplied `SystemTime` converted internally for the OIDC verifier;
- explicit nonce policy per verification: either `Absent` or `Exact(expected)`.

RO4B MUST NOT expose an option to disable signature, issuer, audience, expiration or nonce policy checks.

The single-audience restriction is deliberate. The current `openidconnect` crate documents that automatic `azp` verification is unsupported. Rejecting multiple-audience ID tokens prevents the first adapter from accepting a token profile whose authorized-party rule it does not completely verify.

The first slice does not support:

- HMAC/shared-secret ID-token signatures;
- encrypted ID tokens/JWE;
- multiple audiences / `azp` processing;
- ephemeral-subject extensions;
- custom provider claims as RunenOnline identity;
- access tokens as identity evidence;
- UserInfo as the identity source;
- authorization-code/device/implicit flow orchestration;
- token refresh/revocation/session/logout management.

These are separately justified extensions, not compatibility requirements.

### OIDC time is not RunenOnline deadline time

The host-supplied `SystemTime` used to verify OIDC `exp` is part of the authentication realization. It is **not** a `TrustedTime` observation for Assignment, AdmissionGrant, or MatchRequest deadlines and MUST NOT be reused or described as RunenOnline's canonical clock.

This separation prevents an authentication protocol timestamp representation from becoming semantic control-plane time authority.

## Adapter handoff

The adapter should expose one small verifier object and an explicit nonce expectation. Exact Rust names remain implementation-local, but the responsibilities are equivalent to:

```text
OidcVerifier::from_jwks_json(config, jwks_json)

OidcVerifier::verify(
    authority,
    raw_id_token,
    nonce_expectation,
    verification_time,
) -> VerifiedExternalPrincipal
```

Verification order should fail closed:

1. enforce raw token size before parsing;
2. parse the OIDC ID token;
3. verify RS256 signature using configured JWKS;
4. require exact issuer;
5. require exactly one exact client audience;
6. verify expiration at the supplied host time;
7. enforce `Absent` or exact nonce expectation;
8. extract only verified `iss` + `sub` identity;
9. call the owning RunenOnline `Authority` verified-principal acceptance boundary using exact issuer bytes and subject bytes.

Step 9 preserves RunenOnline trust/representation policy. A cryptographically valid token from an issuer not recognized by that `Authority` still fails.

No PlayerId is created, associated or resolved automatically by OIDC verification. Those remain separate RunenOnline/application operations.

## Failure boundary

RO4B needs adapter-local typed failures sufficient for deterministic handling, without creating a normative RunenOnline authentication taxonomy.

It should distinguish only what the host must act on differently, for example:

- invalid adapter configuration;
- JWKS/token input exceeds configured bound;
- malformed JWKS/token;
- unsupported token profile;
- cryptographic/claim verification failure;
- RunenOnline trusted-principal acceptance failure.

Underlying library/provider diagnostic text may be retained as source/debug information but is not a portable semantic failure enum.

There is no network ambiguity inside RO4B because it performs no provider I/O. Unknown/rotated key, invalid signature, expired token, wrong issuer/audience, or nonce mismatch all fail closed. Key refresh and a later retry are host operations against a newly configured verifier; a failed verification never creates RunenOnline identity state by itself.

## Deterministic assurance

RO4B should use fixed test keys/tokens and explicit verification times; no live OIDC provider or network is required for repository acceptance.

At minimum test:

- valid RS256 token with exact issuer, single audience, unexpired time and exact nonce produces `VerifiedExternalPrincipal`;
- the verified result can be associated/resolved through `runen-online` without the adapter creating PlayerId/application policy;
- cryptographically valid token from an issuer not trusted by the `Authority` fails at the RunenOnline boundary;
- wrong issuer fails;
- wrong audience fails;
- multiple audiences fail even when one equals the client ID;
- expired token fails at exact host-supplied verification time;
- bad signature / unknown key fails;
- expected nonce mismatch fails;
- `Absent` nonce policy rejects a token carrying a nonce;
- oversized token fails before parsing;
- oversized JWKS and excessive JWK count fail before activation;
- failure creates no principal association or PlayerId implicitly;
- no network/runtime feature is enabled in the adapter dependency graph.

Focused adapter tests do not replace exact-head `cargo validate`.

## Architecture update required by delivery

`ARCHITECTURE.md` should record only the durable package fact established by evidence:

- `crates/runen-online` remains the provider-neutral semantic core;
- `crates/runen-online-oidc` is an optional OIDC verification adapter that depends on the core;
- OIDC protocol/crypto dependencies do not enter the core;
- OIDC issuer/subject/token/JWKS representations remain adapter realization data, not universal RunenOnline identity/wire authority;
- no broader auth/provider/service package family is ratified by this one adapter.

No normative `spec/` change is required for RO4B.

## Deferred production work

### Persistence

Needs a dedicated investigation into a durable semantic-operation boundary, authority-domain restoration/non-reuse, trusted deadline realization, transactions/concurrency and restart recovery. Do not add a generic repository trait as a side effect of OIDC work.

### OIDC discovery/key refresh

Host-managed in RO4B. A later adapter extension may own HTTPS discovery/JWKS refresh only with explicit SSRF/redirect, cache, refresh, outage and stale-key policy. The current `openidconnect` documentation explicitly warns that HTTP clients used for discovery should not follow redirects because of SSRF risk.

### Service transport

Defer until remote operation, wire representation and ambiguous-commit/idempotency boundaries are accepted from proven production behavior.

### Server allocation

Defer until an accepted integration boundary exists for provider allocation/resource/connection material to application-defined logical destination and Assignment policy.

## Rejected alternatives

### Put OIDC behind a feature in `runen-online`

Rejected. It would make protocol/crypto dependencies part of the semantic-core package surface even though OIDC is optional and independently reusable.

### Add a generic `runen-online-auth` crate first

Rejected. One concrete OIDC adapter does not prove a useful generic authentication package or trait beyond the already-existing verified-principal semantic boundary.

### Own OIDC discovery and HTTP in the first slice

Rejected. Token verification is independently useful and deterministic without network I/O. Discovery immediately introduces SSRF, redirects, TLS, cache/refresh, outage and async/blocking runtime policy that is not needed to prove the adapter handoff.

### Start with PostgreSQL persistence

Deferred. It is broader than one provider adapter and current implementation mechanics do not yet provide a small truthful storage contract.

### Start with gRPC/HTTP service API

Deferred. It would force wire/service schemas and ambiguous remote-commit policy before the production state boundary is proven.

### Start with Agones allocation

Deferred. It would couple the first production slice to dedicated-server provider identities and connection data that are intentionally not Assignment semantics.

## Decision-complete RO4B slice

The next delivery issue may now be written without an unresolved capability or package boundary:

1. add `crates/runen-online-oidc` and workspace/lockfile entries;
2. add only the direct dependencies listed above, with `openidconnect` HTTP/default features disabled;
3. implement bounded static-JWKS RS256 ID-token verification with exact issuer, single audience, expiration and explicit nonce policy;
4. convert only verified issuer+subject through the existing `Authority::accept_verified_external_principal` boundary;
5. add deterministic fixed-key/fixed-time tests including failure rollback and core trust-set rejection;
6. update `ARCHITECTURE.md` only for the justified optional adapter package/dependency direction;
7. run focused adapter tests and canonical `cargo validate` on the exact reviewed head.

Stop if implementation proves that the existing verified-principal boundary cannot express the selected handoff without changing portable semantics. In that case route the reproduced gap back to specification/architecture authority rather than inventing a compatibility layer.

## Conclusion

The first production realization should be a narrow OIDC Core ID-token verifier adapter, not persistence, service transport or server allocation.

This choice uses an already-accepted semantic seam, gives the first real provider/protocol integration, keeps the std-only core independent, and avoids pretending that the process-local RO2 aggregate is already a durable database/service model. Persistence remains the most important broader production problem, but it requires its own semantic-operation/restart/transaction investigation rather than being smuggled into the first provider slice.
