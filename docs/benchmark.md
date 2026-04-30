# LLM Benchmark Runner

`tools/llm-benchmark` is the benchmark CLI used to measure local LLM latency,
throughput, and stability on the baseline machine in [hardware.md](hardware.md)
(Ryzen 5 7430U / 16 GB RAM / iGPU-only).

The benchmark is model-agnostic and can be used with any local GGUF model.
Any model currently used in local runs should be treated as an isolated test
selection, not a project-wide benchmark default.

## What this tool measures

- TTFT split into prompt decode + first-token decode
- Throughput (`tps_mean`, `tps_median`)
- Inter-token latency consistency (`itl_p99_us`, `itl_variance_us2`)
- Memory before/peak/after run
- CPU usage samples and thermal readings (Linux, when available)
- Drift between early vs late generation cadence
- Optional per-token timeline (`report.flow.jsonl`) for plotting

## Requirements

- Rust toolchain (`cargo`)
- C/C++ build prerequisites for `llama-cpp-4`
- A GGUF model file for **llama.cpp** / `llama-cpp-4` (production intent: **Qwen 2.5 3B Instruct Q4_K_M**, see [decisions.md](decisions.md) ADR-002). Download weights with `./models/install-models.sh` — default path `models/llm/Qwen2.5-3B-Instruct-Q4_K_M.gguf`.

## Build

From repository root:

```bash
cargo build --release --manifest-path tools/llm-benchmark/Cargo.toml
```

Or from the tool directory:

```bash
cd tools/llm-benchmark
cargo build --release
```

## Run

### Quick run (uses config auto-discovery)

```bash
./models/install-models.sh   # once: fetch GGUF (+ other ONNX weights)
cd tools/llm-benchmark
cargo run --release --
```

By default, config is discovered from:

- `llm-benchmark.toml` (current directory)
- `tools/llm-benchmark/llm-benchmark.toml`

Precedence is: `CLI flags` > `config file` > `built-in defaults`.

### Explicit run with overrides

```bash
cd tools/llm-benchmark
cargo run --release -- \
  --model-path ../../models/llm/Qwen2.5-3B-Instruct-Q4_K_M.gguf \
  --prompt "Summarize local inference tradeoffs in bullets." \
  --warmup-iterations 1 \
  --iterations 5 \
  --n-threads 4 \
  --gpu-layers 0 \
  --sampler greedy \
  --seed 1234 \
  --max-tokens 128 \
  --cooldown-secs 2.5 \
  --jsonl-output report.flow.jsonl \
  --output report.json
```

## Configuration

Central defaults live in `tools/llm-benchmark/llm-benchmark.toml`.

Notable fields:

- `model_path`
- `prompts` (multi-profile workload in one run)
- `warmup_iterations`, `iterations`
- `n_threads`, `gpu_layers`
- `max_tokens`, `cooldown_secs`
- `sampler`, `seed`, `temperature`, `top_k`, `top_p`
- `timeout_secs`
- `output`, `jsonl_output`

If `model_path` is not set, the tool auto-discovers the first `.gguf` under:
`models/`, `../models/`, then `../../models/` (recursive; typically `models/llm/`).

## Output artifacts

- `report.json`: aggregated report with environment, summary, profile summaries,
  and per-run raw metrics.
- `report.flow.jsonl` (optional): per-token flow events for cadence plotting.

Key `report.json` sections:

- `summary`: aggregate percentiles (`p50`, `p95`, `max`) for TTFT, TPS, and ITL
- `profiles`: per-prompt profile summaries
- `runs`: per-iteration detailed metrics

## Interpretation notes

- For latency (`ttft_us`, `itl_us`): lower is better.
- `p50` is typical behavior; `p95` is tail latency; `max` is worst outlier.
- Rising ITL variance and negative TPS drift can indicate throttling or unstable
  generation cadence.

Production constraints are defined by target hardware and system docs
([hardware.md](hardware.md), [constraints.md](constraints.md)), not by benchmark
defaults alone.
