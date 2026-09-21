# orderflow

Rust workspace for a **matching engine** and the **order pipeline** around it — built so you can **replay event streams**, **simulate HFT-style workloads**, and **run strategies** against deterministic scenarios.

## Mission

Beyond a production-style matcher, orderflow is meant to be a **simulation and replay platform**:

- Record or import **event sequences** (orders, cancels, market data ticks, fills)
- **Replay** those events through the same matching logic used in live paths
- **Simulate** high-frequency and low-latency scenarios without external infrastructure
- Plug in **strategies** that react to replayed events and emit new orders

The engine and pipeline share stable domain types so live, replayed, and simulated runs stay comparable.

## Status

Early scaffold. Workspace builds; core matching logic and replay tooling are not implemented yet.

| Area | State |
|------|--------|
| Domain types (`Order`, `EngineCommand`, `EngineEvent`, `MatchingEngine`) | Defined in `engine-types` (integer Price i64, Quantity u128) |
| Engines (`engine-tokio`, `engine-gloomio`) | Placeholders (`process` appends no events) |
| Pipeline (`input`, `screener`, `sequencer`, `scheduler`, `output`) | Placeholder stages |
| Event replay / simulation | Planned |

Domain types use integer economic values (`Price` i64 ticks, `Quantity` u128 lots). The wire format is **OFL1**: a compact packed journal with three clocks (`CommandSequence`, `EventSequence`, `JournalSequence`), stream and datagram envelopes, and no sockets yet.

## Workspace layout

```
crates/
  engine-types/     # Order, EngineCommand (NewLimit/NewMarket), EngineEvent, MatchingEngine
  protocol/         # OFL1: stream + datagram envelopes (CLOB 0x01–0x05, events 0x81–0x85)
  engine-tokio/     # Tokio-backed matcher
  engine-gloomio/   # Gloomio-backed matcher (runtime comparison)
  engine-core/      # Engine entry binary
  input/            # Order intake
  screener/         # Pre-match filtering
  sequencer/        # CommandSequence stamp before matching
  scheduler/        # Routing to engine
  output/           # Execution / event output
```

**Intended flow:** `input → screener → sequencer → scheduler → engine → output`

For replay/simulation, the same flow applies: events are fed as if live, with a clock and recorder driven by the scenario rather than wall time.

## Quick start

**Local Rust:**

```bash
cargo build
cargo test
cargo run -p engine-core
```

**Docker (dev / prod compose):**

```bash
just dev-up      # start development stack
just dev-down    # stop
just prod-up     # production-like compose locally
just prod-down
```

Kamal deploy scaffold: `config/deploy.yml` — fill hosts and registry, then `just kamal-deploy`.

Per-crate smoke tests:

```bash
cargo run -p input
cargo run -p screener
cargo run -p sequencer
cargo run -p scheduler
cargo run -p output
```

## Development

```bash
cargo fmt
cargo clippy --all-targets --all-features
```

## Documentation

Project planning and architecture docs (ai-squads) live outside this repo:

`/Users/seb/docs/orderflow/`

Includes `mission.md`, `components.md`, `flows.md`, `roadmap.md`, and environment notes.

## License

**Proprietary — not open source.** All rights reserved. See [LICENSE](LICENSE).

Unauthorized copying, modification, distribution, or use is prohibited without
written permission from Sebastian Concept.
