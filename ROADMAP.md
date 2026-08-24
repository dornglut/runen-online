# RunenOnline Roadmap

This document owns project sequencing and acceptance gates. It does not define RunenOnline semantics or repository dependency law.

## RO0 — Authority foundation

Establish the standalone control-plane product boundary, documentation authority, specification conventions, repository architecture, roadmap, licensing, and canonical validation contract. No substantive online-service implementation is introduced in this stage.

**Gate:** authority is unambiguous; provider and sibling-framework independence are explicit; later semantic work can proceed without deriving rules from backend products or existing game-server behavior.

## RO1 — Minimum semantic contracts

Investigate and specify the minimum provider-neutral control-plane model required for a first implementation. Resolve only implementation-critical identity, trust/admission, lifecycle, resource-bound, coordination, assignment, and related contracts justified by accepted work. Keep unresolved areas explicit rather than filling them from provider convenience.

**Gate:** the minimum implementation-critical invariants have normative owners; externally or remotely influenced state has explicit bounded-policy requirements where applicable; provider behavior is not semantic authority.

## RO2 — Minimal semantic core

Implement the accepted RO1 state machines and contracts without requiring RunenNet, Runenwerk, a concrete engine/ECS, production database/backend, authentication provider, fleet orchestrator, or concrete service transport. Add deterministic executable assurance sufficient for the accepted semantic profile.

**Gate:** accepted control-plane behavior and required resource/security invariants are testable independently of production providers and game-engine integration.

## RO3 — Standalone proof

Provide a plain-Rust proof that exercises the accepted RunenOnline core with ordinary host-owned state and deterministic/in-memory realizations only.

**Gate:** the proof requires no RunenNet, Runenwerk, concrete ECS/engine runtime, production database/backend platform, or server-fleet provider.

## RO4 — Production realizations

Introduce the first production authentication, persistence, service-transport, and/or server-allocation realizations only through separately accepted work justified by the proven core. Concrete provider selection belongs to implementation/operations unless a portable semantic contract requires standardization.

**Gate:** production realizations preserve the same accepted RunenOnline contracts and make provider-specific failure/capability boundaries explicit rather than redefining semantics.

## RO5 — Game-server and RunenNet composition proof

Prove explicit composition with a real game-server boundary and, where independently justified, RunenNet. Keep account/player, match/assignment, RunenNet session/participant, allocation, and transport identities distinct through explicit adapters.

**Gate:** a multiplayer product can consume RunenOnline and RunenNet without either repository becoming hidden semantic authority for the other and without requiring Runenwerk.

## RO6 — Hardening and public framework surface

Harden hostile-input, replay, expiry, authorization, concurrency, provider-failure, observability, and operational behavior defined by accepted semantics; refine coherent standalone APIs and documentation around proven use.

**Gate:** supported control-plane contracts have deterministic assurance and production evidence, and the public framework surface does not expose provider or host accidents as semantic requirements.

## RO7 — Maturity and justified extensions

Add broader online-game capabilities, additional provider adapters, performance/scaling work, compatibility/version policy, and ecosystem features only through separately accepted demand.

**Gate:** pre-1.0 stability criteria and extension boundaries are explicit; new features compose without collapsing RunenOnline into RunenNet, Runenwerk, a game simulation framework, or a provider-specific backend.

## Sequencing rule

Stages are dependency gates, not a feature checklist. Later work does not bypass unresolved authority, correctness, security, or provider-neutrality obligations from an earlier gate. Parallel work is acceptable only where ownership and assumptions are independent.
