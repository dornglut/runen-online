# Maturity and Compatibility Policy

This document owns RunenOnline's **current support, maturity, and compatibility claim** and the criteria required before that claim may be strengthened.

It does **not** define RunenOnline semantics, project sequencing, package/dependency ownership, verification policy, release mechanics, or work authorization. Those concerns remain with their existing repository and Dornglut owners.

## Current claim

RunenOnline is **provisional pre-1.0**.

The current support claim is deliberately narrow:

- the specification is `0.1-provisional`: accepted semantic authority that changes only through explicit normative revision;
- the currently supported semantic surface is the normative surface indexed by `spec/README.md` and evidenced by `docs/verification/supported-contracts.md`;
- `runen-online` and `runen-online-oidc` are currently package version `0.1.0`, but their public Rust APIs do not carry an unstated backward-compatibility promise;
- passing `cargo validate` proves the reviewed repository revision satisfies the repository acceptance gate; it does not promise compatibility with another revision;
- implementation-local Rust representations, errors, OIDC details, proof mappings, and other realization choices remain non-portable unless a separate owner explicitly standardizes them;
- no general wire, storage, credential, provider, release, or cross-version compatibility guarantee is currently established beyond separately accepted contracts.

Package version, specification version, semantic authority, executable assurance, and compatibility promises are separate concerns.

## Assurance is not compatibility

RunenOnline distinguishes evidence about one revision from promises between revisions:

- **normative semantic authority** defines required portable behavior for the current specification revision;
- **executable assurance** demonstrates that a reviewed implementation preserves that authority under exercised scenarios;
- **Rust API/package compatibility** concerns continuity of downstream source/API use across revisions;
- **wire/storage compatibility** concerns continuity of external representations;
- **provider compatibility** concerns guarantees of a concrete realization;
- **release/distribution compatibility** concerns promises attached to an explicit version and distribution policy.

Passing tests or validation establishes assurance for the reviewed revision only. It does not automatically establish any of the compatibility claims above.

## Strengthening the claim

A stronger maturity or compatibility claim requires separately accepted work for the exact scope being strengthened. At minimum, that work must establish:

1. **Defined support boundary** — the claimed semantic surface has explicit normative owners; support-critical open items are resolved or explicitly outside the claim.
2. **Current assurance** — the claimed behavior has executable evidence and passes canonical validation on the reviewed head.
3. **Public-surface boundary** — the exact Rust/package/provider compatibility surface to protect is identified without promoting implementation accidents into portable semantics.
4. **Package/dependency evidence** — reusable packages and production adapters have accepted ownership and explicit capability/failure boundaries.
5. **Security and maintenance evidence** — supported production realizations have an appropriate path for material dependency/security maintenance.
6. **Consumer evidence where claimed** — integration compatibility claims have executable downstream proof rather than inferred compatibility.
7. **Version/distribution decision when needed** — before promising compatibility across published releases, version scope, breaking-change treatment, and release/distribution mechanics are explicitly accepted.

These are gates for strengthening a claim. They are not a requirement to resolve every future online-game concern before ordinary pre-1.0 development continues.

This document does not define a 1.0 readiness checklist.

## Change routing

This document routes concerns to existing owners; it does not authorize work.

| Change | Existing owner / authority path |
| --- | --- |
| Portable provider-neutral behavior | owning `spec/` artifact through explicit normative revision |
| Rust API/type/error refactor with unchanged semantics | live issue/work authority plus current executable assurance |
| New reusable crate or provider adapter | `ARCHITECTURE.md` package/dependency ownership plus separately accepted capability evidence |
| Persistence/service/fleet/concurrent realization | investigate the concrete mechanism; new portable ordering/failure/authority/replay/resource rules go to `spec/` |
| RunenNet, game-server, or Runenwerk integration | independent integration demand plus existing cross-repository and identity/lifecycle boundaries |
| Broader online-game capability | separately accepted demand under `ROADMAP.md`, then semantic ownership investigation as required |
| Performance/scaling optimization | implementation work if observable semantics are unchanged; normative investigation if resource/order/failure semantics change |
| Stronger compatibility/stability claim | this document's evidence gates, followed by separately accepted version/release mechanics when relevant |

Live work remains governed by `AGENTS.md` and Dornglut's authority/work process.

## Current extension state

RO7 currently selects **no broader online-game extension**. The repository had no independently accepted extension demand at the RO7 stage boundary.

Future feature families, provider adapters, persistence/service/fleet realizations, scaling work, ecosystem integrations, or similar extensions require separately accepted demand under `ROADMAP.md`. This document does not admit or approve them.

## Not established by this policy

The current maturity claim does not establish:

- a 1.0 release or readiness commitment;
- a formal cross-release versioning or breaking-change scheme;
- crates.io publication or repository release automation;
- deprecation windows or supported-version lifetimes;
- stable wire, storage, or AdmissionGrant credential formats;
- provider compatibility guarantees beyond separately accepted realization contracts;
- a public conformance profile;
- Runenwerk integration.

Any such commitment requires separately accepted future work under its proper owner.
