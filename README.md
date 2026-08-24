# RunenOnline

RunenOnline is a standalone, provider-neutral Rust framework for online-game control-plane semantics outside the realtime multiplayer networking core.

The cross-repository boundary with RunenNet is defined by [Dornglut Engineering ADR 0006](https://github.com/dornglut/engineering/blob/main/adrs/0006-separate-realtime-networking-from-online-control-plane.md).

The repository is being established under [RO0 — authority, control-plane boundary, and repository architecture](https://github.com/dornglut/runen-online/issues/1).

No backend provider, database, service transport, RunenNet integration, Runenwerk integration, or existing product behavior is authoritative here unless and until its contract is deliberately accepted by RunenOnline.
