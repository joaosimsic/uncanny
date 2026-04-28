# llm-benchmark

CLI benchmark for measuring local LLM latency and throughput with `llama-cpp-4`.

## Features

- Direct llama.cpp integration via Rust bindings (`llama-cpp-4`)
- TTFT split into prompt decode + first-token decode
- Per-token flow timeline (`decode_us`, `since_generation_start_us`, cumulative TPS)
- TTFT, TPS (mean/median), ITL variance, ITL P99
- Aggregated summary metrics in JSON (`p50`, `p95`, `max`)
- ITL aggregate percentiles (`p50`, `p90`, `p95`, `p99`)
- Warmup iterations (excluded from measured runs)
- Memory usage tracking (before, peak during run, after)
- CPU usage sampling and Linux thermal zone sampling (when available)
- Cooldown between iterations to reduce thermal throttling
- Configurable llama CPU threads (`n_threads`)
- Configurable GPU offload layers (`gpu_layers`)
- Configurable sampler and seed for reproducible comparisons
- Multi-prompt workload profiles in one benchmark run
- Drift metrics (early vs late token cadence/throughput deltas)
- Optional JSONL flow event output for plotting
- JSON report output for post-analysis

## Build

```bash
cargo build --release
```

## Run

```bash
cargo run --release -- \
  --model-path /path/to/model.gguf \
  --prompt "You are a concise assistant." \
  --prompt "Summarize local inference tradeoffs in bullets." \
  --warmup-iterations 1 \
  --iterations 5 \
  --n-threads 4 \
  --gpu-layers 0 \
  --sampler greedy \
  --seed 1234 \
  --max-tokens 128 \
  --jsonl-output report.flow.jsonl \
  --cooldown-secs 2.5 \
  --output report.json
```

## Configuration

By default, the tool auto-discovers a config file in common locations, including:
`llm-benchmark.toml` (current dir) and `tools/llm-benchmark/llm-benchmark.toml`.

- Precedence is: `CLI flag` > `config file` > `built-in defaults`.
- To use another file, pass `--config /path/to/config.toml`.
- Typical quick run (no extra flags):

```bash
cargo run --release --
```

If `model_path` is omitted, the tool auto-discovers the first `.gguf` found in:
`models/`, `../models/`, then `../../models/`.

### Config fields

The config file is TOML. Example:

```toml
model_path = "../../models/Llama-3.2-1B-Instruct-Q4_K_M.gguf"
prompts = [
  "You are a concise assistant. Answer in one short paragraph.",
  "Summarize local inference tradeoffs in bullets.",
]
warmup_iterations = 1
iterations = 5
n_threads = 4
gpu_layers = 0
max_tokens = 128
cooldown_secs = 2.0
output = "report.json"
jsonl_output = "report.flow.jsonl"
sampler = "greedy"
seed = 1234
temperature = 0.8
top_k = 40
top_p = 0.95
timeout_secs = 30.0
```

Field reference:

- `warmup_iterations`: Number of warmup runs to execute before measured runs.
- `iterations`: Number of measured runs stored in `runs`.
- `model_path`: Path to the GGUF model file.
- `prompts`: List of prompts used as workload profiles.
- `prompt`: Single-prompt shorthand (used if `prompts` is absent).
- `n_threads`: Number of CPU threads used by llama context.
- `gpu_layers`: Number of model layers requested for GPU offload (`0` = CPU only).
- `max_tokens`: Maximum generated tokens per run.
- `cooldown_secs`: Delay between runs to reduce thermal carry-over.
- `output`: JSON output path.
- `jsonl_output`: Optional token-level flow output path.
- `sampler`: `greedy` or `stochastic`.
- `seed`: Random seed for stochastic sampling reproducibility.
- `temperature`, `top_k`, `top_p`: Stochastic sampling controls.
- `timeout_secs`: Optional per-run generation timeout.

## Output

The generated JSON includes:

- Environment/config fields (`model_path`, `n_threads`, `gpu_layers`, etc.)
- Runtime fingerprint (`model_sha256`, crate version)
- `summary` with aggregate metrics across measured runs:
  - `ttft_us`: `p50`, `p95`, `max`
  - `tps`: `p50`, `p95`, `max`
  - `itl_us`: `p50`, `p95`, `max`
  - `itl_percentiles_us`: `p50`, `p90`, `p95`, `p99`
  - `drift`: early-vs-late TPS/ITL deltas
- `profiles`: per-workload profile summaries
- `runs` with per-iteration raw metrics and samples

## Metrics explained

### Per-run metrics (`runs[*]`)

- `prompt_decode_us`: Prompt ingestion/decode latency in microseconds.
- `first_token_decode_us`: First generated-token decode latency in microseconds.
- `ttft_us`: `prompt_decode_us + first_token_decode_us`. Lower is better.
- `tps_mean`: Average generated tokens/sec across the whole generation window.
- `tps_median`: Median instantaneous TPS derived from token decode latencies.
- `itl_samples_us`: Inter-token decode latencies (microseconds), one sample per generated token.
- `itl_p99_us`: P99 inter-token latency for the run. Lower is better for smoothness.
- `itl_variance_us2`: Variance of inter-token latencies. Lower means more consistent pacing.
- `generated_tokens`: Number of tokens generated before EOG or `max_tokens`.
- `memory_before_kib` / `memory_peak_kib` / `memory_after_kib`: Process memory in KiB.
- `avg_cpu_usage_percent` / `peak_cpu_usage_percent`: CPU usage sampled during the run.
- `thermal_samples`: Time series of CPU thermal readings (if available).
- `token_flow`: per-token timeline points for cadence analysis.
- `run_status`, `error`, `timed_out`: failure/timeout accounting fields.

### Aggregate metrics (`summary`)

- `ttft_us.p50/p95/max`: Median, tail, and worst TTFT across successful measured runs.
- `tps.p50/p95/max`: Distribution of per-run mean TPS across measured runs.
- `itl_us.p50/p95/max`: Distribution of per-run ITL P99 values.
- `measured_runs`: Number of runs included after warmup exclusion.
- `failed_runs`: Number of runs that failed.
- `timed_out_runs`: Number of runs that hit timeout.
- `generated_tokens_total`: Sum of generated tokens across measured runs.

### Percentiles (`p50`, `p95`, `max`)

- `p50` (50th percentile): The median value. About half of measured runs are better and half are worse.
- `p95` (95th percentile): Tail behavior. 95% of measured runs are at or below this value; worst 5% are above it.
- `max`: The single worst observed value.

For latency metrics (`ttft_us`, `itl_us`), lower is better. In practice:
`p50` describes typical behavior, `p95` describes user-visible spikes, and `max` describes the worst outlier.

### Interpretation tips

- **Responsiveness:** prioritize `ttft_us.p95` for conversational feel.
- **Steady speaking speed:** use `tps.p50` and `itl_us.p95` together.
- **Jitter/stutter risk:** rising `itl_variance_us2` and `itl_us.max` indicate uneven output cadence.
- **Thermal throttling signs:** degrading TPS + rising latency over later runs.
- **Flow degradation:** positive `drift.itl_early_late_delta_us` and negative
  `drift.tps_early_late_delta` indicate slowing generation over time.

## Validation recipes

- Compare short/medium/long profile prompts and inspect `prompt_decode_us` deltas.
- Re-run with fixed `seed` and ensure cadence envelopes remain stable.
- Stress for thermal effects (higher `iterations`, lower `cooldown_secs`) and verify drift signals.

## Notes

- `--n-threads` defaults to physical cores (`num_cpus::get_physical()`).
- `--warmup-iterations` defaults to `1`.
- `--gpu-layers` defaults to `0` (CPU-only inference).
- `mlock` is requested during model load (`with_use_mlock(true)`).
- Thermal readings depend on `/sys/class/thermal` availability and exposed CPU zones.
- This crate sets `LLAMA_BUILD_SHARED_LIBS=0` in `.cargo/config.toml` to avoid
  `libllama.so.0` runtime linker errors.
