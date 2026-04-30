# Testing modalities: simulation and benchmark

The project targets a concrete hardware envelope ([hardware.md](hardware.md)) and testable latency and cadence budgets ([constraints.md](constraints.md), [architecture.md](architecture.md) §Tick Rates). Validating against those targets uses **two different kinds of exercise**, which answer different questions and should not be confused.

| Modality | Primary question | Typical artifact |
|---|---|---|
| **Simulation** | Is the domain pipeline correct and reproducible given controlled inputs and stubs? | Deterministic replay, golden traces, regression on perception → fusion → behavior |
| **Benchmark** | Does this machine actually hit TTFT, throughput, memory, and thermal headroom targets? | Measured latency percentiles, CPU/iGPU load, thermals, memory curves |

Both are necessary: simulation isolates logic and pipeline *rules* (tick cadence, what gets dropped vs queued) without pinning you to one thermal day; benchmark grounds assumptions in *real silicon* so end-to-end targets in [constraints.md](constraints.md) and per-component rates in [architecture.md](architecture.md) stay honest as the stack grows.

---

## Simulation (deterministic replay)

**What it is.** In [architecture.md](architecture.md) §Simulation, the Godot-side project replaces real input and output **adapters** with **sim stubs**. The hexagonal domain (perception, fusion, behavior) stays the same binary path as production: swap adapters only, not core logic.

**What it is good for.**

- **Correctness and regression:** fixed inputs drive the same domain decisions every run, which is essential for replay-based tests and for debugging fusion/behavior edge cases without a live user or robot.
- **Pipeline semantics without full hardware:** once stubs can feed atomic snapshots and accept motor/voice commands on a known clock, you can rehearse overlaps that matter in production (e.g. vision cadence vs. robot speech, hysteresis before motor updates) using the cadences documented in [architecture.md](architecture.md) §Tick Rates and [constraints.md](constraints.md). Prefer **drop stale work** rather than growing unbounded queues on the live path—a policy simulation can model without running ONNX at full duty cycle.
- **Alignment with architecture choices:** [architecture.md](architecture.md) §Threading keeps the domain path sync-friendly and avoids an async runtime on the hot path so runs stay **deterministic for Godot replay** §Simulation — simulation is the consumer of that property.

**What it is not.** Simulation does **not** prove that the baseline Ryzen+iGPU setup sustains 15 FPS vision, 30 FPS acoustic paths, or the ≤800 ms speech-to-speech budget under load ([constraints.md](constraints.md)). Stubbed adapters do not run RetinaFace, SenseVoice, or Piper at full cost; thermals and DRAM bandwidth contention do not appear unless you deliberately model them (which is out of scope for “byte-identical domain + stub I/O”).

**Status.** The Godot sim scaffold is not built yet; see [roadmap.md](roadmap.md).

---

## Benchmark (measurement on real hardware)

**What it is.** Today this is anchored on **`tools/llm-benchmark`**, documented in [benchmark.md](benchmark.md): a CLI that runs **real** local inference (GGUF via llama.cpp) on the baseline machine described in [hardware.md](hardware.md) and records TTFT, token throughput, inter-token jitter, RAM usage, CPU sampling, and Linux thermal readings where available.

**What it is good for.**

- **Turning spreadsheet estimates into data:** thread-count sweeps (e.g. `n_threads` for `llama-cpp-2`), TTFT against conversational latency goals in [constraints.md](constraints.md) (including the open per-stage breakdown there), and memory before/peak/after vs. fitting in **16 GB** system RAM alongside other services.
- **Stability under sustained load:** variance and drift in token cadence can flag thermal throttling or an overstretched core allocation — risks that matter when LLM CPU work, ONNX on the iGPU, and audio run together.
- **Repeatable methodology:** for long characterization runs it is reasonable to disable CPU turbo so numbers are comparable run-to-run; treat that as a **benchmark-only** methodology choice, not a production tuning default.

**What it is not.** The current benchmark focuses on the **LLM slice** of the pipeline. It does not yet replace a full multi-thread pipeline soak (vision + ASR + domain + TTS). Whether to extend this CLI or add a separate harness—and to reuse thermal/CPU sampling in the live runtime—is future integration work; see [roadmap.md](roadmap.md).

**Where to run and read outputs.** Build/run instructions and report fields are in [benchmark.md](benchmark.md).

---

## How to use both together

1. Use **benchmark** to lock assumptions: thread counts, TTFT envelope, RAM headroom, and thermal behavior on the laptop you actually ship against.
2. Use **simulation** to prevent regressions in the domain path and to rehearse scheduling policies (drops, hysteresis, turn-taking) with deterministic inputs.
3. When the full pipeline exists, combine them: benchmarks justify per-stage budgets; simulation asserts that the **logic** honoring those budgets is stable; integrated soak tests (future work) close the gap between “LLM only” and “everything at once.”

---

## Cross-references

- [architecture.md](architecture.md) — §Simulation (Godot stubs), §Threading (atomic snapshots), §Tick Rates.
- [benchmark.md](benchmark.md) — LLM benchmark CLI usage and metrics.
- [constraints.md](constraints.md) — numeric targets both modalities ultimately serve.
- [hardware.md](hardware.md) — baseline machine definition.
- [roadmap.md](roadmap.md) — component status and near-term validation steps.
