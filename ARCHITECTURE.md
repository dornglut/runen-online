# Repository Architecture

This document owns the structure and dependency boundaries of the RunenOnline repository. It does not define online-service semantics.

## Product boundary

RunenOnline is a standalone Rust framework for provider-neutral online-game control-plane semantics outside the realtime multiplayer networking core.

RunenNet is a sibling standalone framework, not RunenOnline's architectural host. Runenwerk, dedicated game servers, and other applications are downstream consumers or integration hosts.

Provider-neutral semantic packages MUST NOT require any of the following merely to define RunenOnline semantics:

- RunenNet;
- Runenwerk;
- a concrete ECS or engine runtime;
- a concrete database, cache, backend platform, or authentication provider;
- a concrete server-fleet or orchestration provider;
- HTTP, gRPC, or another concrete service transport.

Later adapters or integrations may depend on concrete providers and sibling frameworks only when accepted contracts justify that dependency. Such realizations MUST NOT silently redefine provider-neutral RunenOnline semantics.

Authoritative game rules and simulation policy remain application/game-server concerns unless a separately accepted framework explicitly owns them.

## Cross-repository composition

RunenOnline and RunenNet own distinct semantic domains. A consumer may compose both through explicit mappings, but neither framework's semantic core depends on the other merely because a multiplayer product uses both.

The durable organization-level boundary is owned by Dornglut Engineering ADR 0006. Exact RunenOnline identity, admission, coordination, allocation, persistence, or provider semantics are not established by this architecture document and require their own normative owners when accepted.

## Provider law

Control-plane semantics are established before provider realization. Database schemas, authentication-provider behavior, backend-product APIs, server orchestrators, deployment topology, and service transports are implementation or integration evidence unless a normative RunenOnline specification explicitly defines a contract they realize.

A provider adapter may expose provider capabilities or limitations, but it MUST NOT promote incidental provider behavior into portable RunenOnline semantics.

## Implementation packages

RO0 intentionally does not ratify a product crate or multi-service topology.

Creating identity, auth, lobby, matchmaking, allocation, persistence, server, protocol, adapter, or provider crates/services before independent ownership is demonstrated would make package shape precede semantic and operational evidence.

Later implementation work may introduce only the minimum package decomposition justified by accepted semantics, build/dependency boundaries, deployment obligations, or independently reusable adapters.

## Top-level artifact areas

- `spec/` — normative RunenOnline specification artifacts;
- `docs/` — non-normative architecture, verification, decisions, research, and guides when real material exists;
- `tools/` — repository tooling only;
- future `crates/` or application/service packages — product implementation only after package ownership is accepted;
- future `examples/` — standalone proofs and consumer-facing examples when required by accepted work.

## Integration boundary

RunenOnline may later expose explicit contracts consumed by game servers, RunenNet integrations, Runenwerk, or other hosts. Integration code must preserve identity and lifecycle distinctions instead of making an external account, match assignment, server allocation, RunenNet participant/session, or transport connection interchangeable by implementation convenience.

The exact identities and transitions are specification questions, not architecture defaults.
