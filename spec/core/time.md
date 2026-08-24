# Core Control-Plane Time and Deadlines

Status: **provisional incomplete normative**

This document owns only the minimum trusted time and deadline semantics used by RunenOnline control-plane lifetimes. It does not define scheduling, timers, wall-clock synchronization, persistence, transport timing, simulation ticks, or provider-specific lease systems.

## Trusted time observations

A **trusted time observation** is authority-controlled evidence used to decide whether a RunenOnline deadline has been reached.

Untrusted client input MUST NOT become semantic time merely because it contains a timestamp, duration, sequence value, provider date, or claimed clock reading.

The representation of trusted time is not defined by this revision. A conforming realization MAY use monotonic process time, wall-clock time, centralized authority state, provider time, replicated state, or another mechanism if it preserves the semantic rules below.

## Deadline

A **deadline** is a fixed finite boundary after which the operation or authority governed by that deadline is no longer live.

A deadline MUST be established from trusted authority/host policy. Once fixed for one object incarnation, it MUST NOT be extended or moved later by untrusted input or by retrying the same semantic operation.

Whether later trusted policy may replace an object with a fresh incarnation carrying a different deadline belongs to the owner of that object's lifecycle, not to this document.

A deadline's epoch, numeric unit, precision, serialized representation, and public API type are not defined by this revision.

## Expiry truth

For a deadline-governed object, a trusted time observation determines whether the deadline is:

- **not reached** — the object may remain live if all of its other lifecycle requirements are satisfied;
- **reached** — the deadline-governed authority or operation is expired.

These names are semantic descriptions, not required public enum spellings.

Once one object incarnation is authoritatively treated as having reached its deadline, that incarnation MUST NOT later become live again.

Deadline expiry is irreversible for that incarnation. Recreating similar authority after expiry requires whatever fresh identity or lifecycle transition its normative owner defines.

## Comparison contract

A deadline and the trusted observation used to evaluate it MUST belong to a comparison domain whose ordering semantics are defined by the realization.

A realization MUST NOT compare unrelated clock domains, epochs, provider timestamps, or opaque counters as if they were semantically ordered without an explicit trusted conversion or mapping.

If a realization permits clock skew, leeway, uncertainty, or another tolerance around deadline comparison, that tolerance MUST be:

- finite;
- documented as part of the realization's expiry contract;
- applied consistently by components authorized to make the same class of acceptance decision;
- included when reasoning about the effective maximum lifetime promised by the owning semantic contract.

Unbounded, caller-controlled, or silently component-specific deadline tolerance is not conforming.

## Distributed realizations

A distributed realization does not need a globally linearizable clock.

It MUST, however, preserve one declared expiry contract for every component authorized to make a deadline-sensitive acceptance decision for the same semantic object class.

A component MUST NOT accept an object when, under that declared expiry contract and its trusted time/state evidence, the object's effective deadline is reached.

If a component cannot establish trustworthy deadline truth required for an acceptance decision, it MUST fail closed for that acceptance rather than extend authority because its clock, cache, provider, or synchronization source is unavailable.

Centralized introspection, synchronized trusted clocks, replicated authority state, destination-local authority, bounded leases, or other mechanisms MAY realize these rules. No mechanism is standardized here.

## Finite lifetime requirement

When another normative owner requires a **finite lifetime**, its conforming realization MUST establish a deadline whose effective distance from creation/issuance, including any permitted comparison tolerance, is bounded by a documented finite implementation or profile policy.

A representation such as an effectively infinite timestamp, sentinel maximum, or renewable deadline does not satisfy a finite-lifetime requirement unless the owning normative contract explicitly permits renewal.

## Non-equivalence

RunenOnline control-plane time is distinct from:

- RunenNet simulation ticks, delivery ordering, or replication cursors;
- transport packet or connection sequence values;
- database transaction identifiers;
- provider resource versions;
- scheduler task order;
- application simulation time.

Equal numeric representation does not make those domains interchangeable.

## Open specification items

The following are **not defined by this revision**:

- one public RunenOnline clock or timestamp type;
- wall-clock versus monotonic-clock requirements;
- epoch, unit, precision, or serialization;
- clock-synchronization protocol;
- one required skew/leeway value;
- retry/backoff scheduling;
- timer APIs;
- renewable leases;
- long-term timestamp/audit storage.

These open items are not permission to weaken a finite-lifetime or irreversible-expiry requirement defined by another normative owner.
