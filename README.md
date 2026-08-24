# RunenOnline

RunenOnline is a standalone, provider-neutral Rust framework for online-game control-plane semantics outside the realtime multiplayer networking core.

Maturity: provisional pre-1.0 authority and interfaces; semantics and public APIs are not yet stable.

RunenNet is a sibling standalone framework for realtime multiplayer networking. Runenwerk and game/server applications are downstream consumers and integration hosts, not RunenOnline semantic authority.

## Repository authority

- [Specification](spec/README.md) — normative RunenOnline semantics.
- [Repository architecture](ARCHITECTURE.md) — package and dependency boundaries.
- [Roadmap](ROADMAP.md) — sequencing and acceptance gates.
- [Repository testing](TESTING.md) — canonical mechanical validation.
- [Documentation architecture](docs/documentation-architecture.md) — documentation ownership and dependency direction.

Organization contribution and security defaults are maintained by [`dornglut/.github`](https://github.com/dornglut/.github).

## License

RunenOnline is available under the [GNU Affero General Public License v3.0 only](LICENSE). A separate commercial license may be available from copyright holder(s) authorized to grant it; see [LICENSING.md](LICENSING.md).
