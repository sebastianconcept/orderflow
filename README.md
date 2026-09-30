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

### OFL1 codec reference

Measure encode, decode, journal walk, and datagram ingress on the `protocol` crate. Save a baseline on the same machine you will use for later comparisons, and write the CPU model and `rustc --version` next to that save. Baselines stay under `target/` and are not committed.

```bash
just bench                     # ~2 min: 3 command + 1 event kind × stream/datagram, 10k journal/datagram
just bench-full                # includes journal_walk_1m (~70 MiB mixed stream, cold-cache stress)
just bench-save ofl1-macbook   # full bench, including journal_walk_1m; name is yours
just bench-compare ofl1-macbook # same full bench, diffed against that name
```

**Reading Criterion output**

| Group | What it measures | How to read it |
| --- | --- | --- |
| `encode_frame` | One frame encode: header plus fields written into the caller buffer | `thrpt` is bytes per second. `frames` is how many of that command or event the same timing writes per second. |
| `decode_frame` | One frame decode into `Copy` types (no heap) | `thrpt` is bytes per second. `frames` is how many of that command or event the same timing parses per second. |
| `journal_walk_10k` | Journal header + sequential frame decode + payload parse | ~50 µs total (~10 GiB/s thrpt): in-cache replay throughput. |
| `journal_walk_1m` | Same walk on ~1M frames | ~5 ms total (~10 GiB/s): memory bandwidth / LLC miss stress; 10 samples, noisy. |
| `datagram_ingress/10k` | 10k UDP-style datagrams decoded in a loop | Faster than stream walk per byte because there is no single contiguous journal scan. |

The `change:` block on `just bench` compares to the **previous** `cargo bench` result in `target/criterion/`, not to a named snapshot. `just bench-save` and `just bench-compare` run the full bench, including `journal_walk_1m`; `just bench` does not. Use `just bench-save <name>` once, then `just bench-compare <name>` for an intentional diff against that snapshot. Treat a timing regression as real when the median moves more than about 10% and the confidence intervals do not overlap.

Heap use is an exact gate: `cargo test -p protocol --test alloc_budget` (every command and event kind encodes and decodes with zero allocations).

## Documentation

Project planning and architecture docs (ai-squads) live outside this repo:

`/Users/seb/docs/orderflow/`

Includes `mission.md`, `components.md`, `flows.md`, `roadmap.md`, and environment notes.

## License

**Proprietary — not open source.** All rights reserved. See [LICENSE](LICENSE).

Unauthorized copying, modification, distribution, or use is prohibited without
written permission from Sebastian Concept.
