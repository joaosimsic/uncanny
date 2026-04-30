# uncanny

Robot head with human expression and AI-powered talk. Mechanically exposed (no skin), behaviorally human-mimic. Goal: live in the uncanny valley — interaction is supposed to feel uncomfortable.

**Status:** design phase. Only working component is the LLM benchmark runner (`tools/llm-benchmark/src/main.rs`).

## Docs

- [docs/index.md](docs/index.md) — overview + nav
- [docs/architecture.md](docs/architecture.md) — hexagonal layers, data flow
- [docs/roadmap.md](docs/roadmap.md) — what's done vs designed vs TBD
- [docs/benchmark.md](docs/benchmark.md) — current LLM benchmark CLI

## Models (weights)

Large `.gguf` / `.onnx` files are not in git. After clone, run `./models/install-models.sh` (see [models/README.md](models/README.md)).

## Quick start (benchmark only)

```bash
./models/install-models.sh
cd tools/llm-benchmark
cargo run --release --
```

See [docs/benchmark.md](docs/benchmark.md) for full options.
