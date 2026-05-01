# Implementation Plan

> **Status:** plan, not ADR. Lives at root next to [SCHEDULING.md](SCHEDULING.md). Anchors to [docs/architecture.md](docs/architecture.md), [docs/roadmap.md](docs/roadmap.md), [docs/constraints.md](docs/constraints.md), [docs/hardware.md](docs/hardware.md).

This is the order of work to take the repo from "design + benchmark CLI" to a running pipeline. It is shaped around a specific constraint: **development happens on a powerful PC, but the target machine (Ryzen 5 7430U / 16 GB / Vega 7) is not yet owned.** Code must stay portable and any number that depends on the target silicon is deferred until that silicon exists.

---

## 0. Ground rules driven by the dev/target asymmetry

Restating because every later phase depends on it.

| Rule | Why |
|---|---|
| Dev PC is **not** authoritative for performance numbers. | Any tps / TTFT / FPS measured on the dev box is a smoke test, never a production target. The 800 ms budget in [docs/constraints.md](docs/constraints.md) is a 7430U budget. |
| Code must compile and run on Linux x86_64 with iGPU absent. | Dev PC likely has dGPU (or different iGPU). All adapter code must degrade gracefully when OpenVINO / specific accelerators aren't available — CPU fallback or explicit `#[cfg]`. |
| No core-pinning, NUMA, or affinity hardcoded. | Target has 6c/12t Zen 3 mobile; dev PC topology differs. Use thread counts from config, not literals. See [SCHEDULING.md](SCHEDULING.md) §3. |
| Memory budget validation is target-only. | Dev PC won't surface the 16 GB pressure. Don't optimize blindly on dev — log allocations and revisit when target arrives. See [SCHEDULING.md](SCHEDULING.md) §5. |
| Thermal observability matters only on target. | Stub the sampler now, wire it in later. The benchmark CLI already has the code — reuse, don't rewrite. See [SCHEDULING.md](SCHEDULING.md) §7. |
| Validate via simulation while dev. | Deterministic replay (Godot stubs + fixture inputs) is the *only* faithful test bed without target HW. See [docs/testing.md](docs/testing.md). |

Translation: build correctness-first on the dev PC. Save every "does it actually fit?" question for the target laptop. Resist the urge to micro-optimize on hardware you won't ship on.

---

## 1. Phase map

Each phase has a clear "done" gate and an explicit dependency on either dev-only or target-only validation.

```
P1. Scaffold        ─── dev only
P2. Domain core     ─── dev only (sim + unit tests)
P3. Stub adapters   ─── dev only (Godot sim, fixture I/O)
P4. CPU adapters    ─── dev (smoke) + target (perf)
P5. iGPU adapters   ─── target only for real validation
P6. Hardware bridge ─── target + physical sensors/actuators
P7. Scheduler glue  ─── target only
P8. Soak & tune     ─── target only
```

Phases 1–4 can fully proceed before the target PC arrives. Phase 5 is partially feasible (write the code, defer the perf claims). Phases 6–8 require the actual machine plus the BOM in [docs/hardware.md](docs/hardware.md).

---

## 2. Phase P1 — Scaffold (dev only)

Goal: make the empty `crates/core` and `crates/embedded` and `crates/godot` actually do something. Today they are placeholder `lib.rs`.

### Tasks

1. **Workspace deps.** Add common deps to `crates/core/Cargo.toml`: `serde`, `thiserror`, `arc-swap`, `crossbeam-channel` (per [SCHEDULING.md](SCHEDULING.md) §8). No `tokio` in core.
2. **Domain types.** In `crates/core/src/`:
   - `types.rs` — `PerceptionPacket` (per [docs/subsystems/perception.md](docs/subsystems/perception.md) §6) and `SocialContext` (per [docs/subsystems/fusion.md](docs/subsystems/fusion.md) §5). Plain `#[derive(Clone, Copy, Debug)]` POD structs.
   - `ports/mod.rs` — re-export.
   - `ports/eye.rs` — `trait EyeController` per [docs/subsystems/behavior.md](docs/subsystems/behavior.md).
   - `ports/voice.rs` — `trait VoiceEmitter` + `EmotionTint` enum.
   - `ports/perception.rs` — `trait AcousticSource`, `trait VisualSource`, `trait SemanticSource`, `trait SpatialSource`. Each returns latest atomic snapshot.
3. **Slot abstraction.** `slots.rs` — `arc_swap::ArcSwap<T>` wrapper for the latest-value slot pattern in [docs/architecture.md](docs/architecture.md) §Threading. One module, ~30 lines, exhaustively unit-tested for the lock-free read invariant.
4. **Errors.** `error.rs` — `thiserror`-based domain error type. No I/O errors leak into the core; adapters wrap.

**Done when:** `cargo build --workspace` passes; `cargo test -p uncanny-core` passes a few struct-construction and slot round-trip tests.

---

## 3. Phase P2 — Domain core (dev only)

Pure logic. Zero hardware. All testable on dev PC with deterministic inputs.

### Tasks

1. **Perception aggregator** in `crates/core/src/perception/aggregator.rs`. Reads four port snapshots, builds `PerceptionPacket`, emits at 10 Hz per [docs/architecture.md](docs/architecture.md) §Tick Rates. Use a `tick(now: Instant)` function so tests can drive a virtual clock — *no `sleep`, no real time in the core*.
2. **Fusion engine** in `crates/core/src/fusion/`. Implement weighted aggregation + dissonance per [docs/subsystems/fusion.md](docs/subsystems/fusion.md):
   - `weights.rs` — initial constants (45/35/20). Tunable via config later.
   - `engine.rs` — `tick(packet: PerceptionPacket) -> SocialContext`.
   - `hysteresis.rs` — 300 ms minimum-hold per [docs/constraints.md](docs/constraints.md). Drive by `Instant`-as-parameter, not real clock.
   - `decay.rs` — 10 %/s mood decay.
3. **Behavior mapper** in `crates/core/src/behavior/mapper.rs`. `SocialContext` → `EyeCommand` / `VoiceCommand` per the table in [docs/subsystems/behavior.md](docs/subsystems/behavior.md) §State → Behavior Mapping.
4. **Domain loop** in `crates/core/src/runtime.rs`. Coordinates the three above. Sync, single-threaded, deterministic. Adapter threads call into it; the loop never starts threads itself.

### Tests

- Golden traces: a CSV / JSONL of `PerceptionPacket` sequences in `crates/core/tests/fixtures/` that exercise each row of the behavior table. Snapshot the resulting `EyeCommand` stream. This is the regression net.
- Hysteresis: assert no motor commands fire if state flips faster than 300 ms.
- Decay: assert mood returns to neutral within ~10 s of `is_active = false`.

**Done when:** golden traces pass; the domain crate has zero I/O dependencies; `cargo test -p uncanny-core` covers every behavior table row.

---

## 4. Phase P3 — Stub adapters & Godot sim (dev only)

Goal: full pipeline runs end-to-end with stubs. No microphone, no webcam, no Arduino. This is the "it lives" milestone and it should happen before any real adapter code.

### Tasks

1. **Print-stdout `EyeController`** in `crates/godot/src/stub/eye.rs`. Logs `look_at`, `blink`, `saccade` to stderr. Trivial. Listed as "first sim adapter" in [docs/roadmap.md](docs/roadmap.md) §Next concrete steps.
2. **Print-stdout `VoiceEmitter`** in `crates/godot/src/stub/voice.rs`. Same idea — logs the would-be utterance.
3. **Fixture perception sources** in `crates/godot/src/stub/sources.rs`. Replay a JSONL trace as `PerceptionPacket` inputs. Reuse the golden-trace fixtures from P2.
4. **Sim binary.** `crates/godot/src/bin/sim.rs` (or a `main.rs` if we make `uncanny-godot` a `[[bin]]`). Wires stub sources → core runtime → stub outputs. Reads a fixture path from argv.
5. **Godot scaffold (defer if Godot is not installed on dev).** Empty Godot project under `tools/sim-godot/` (new directory). Wires the same stubs through `gdext` later. **Acceptable to skip until we have target HW** — see [docs/roadmap.md](docs/roadmap.md) row "Godot sim".

**Done when:** `cargo run -p uncanny-godot --bin sim -- fixtures/sarcasm.jsonl` produces the expected eye/voice command stream on stderr. The full hexagon is exercised on dev hardware.

---

## 5. Phase P4 — CPU adapters (dev smoke + target perf)

Code that uses CPU only. Compiles & runs everywhere. Real perf numbers are deferred to target.

### Tasks

1. **`llama-cpp` semantic adapter.** New crate `crates/embedded/src/adapters/llm.rs`. Wraps `llama-cpp-2` (current LLM benchmark uses `llama-cpp-4` — pick one and align with [docs/decisions.md](docs/decisions.md) ADR-002; document the version choice in an ADR if it differs). Streams tokens, exposes `SemanticSource`. Reuse prompt/cancellation patterns proven in `tools/llm-benchmark/src/runner.rs`.
2. **`cpal` audio capture.** `crates/embedded/src/adapters/audio_in.rs`. 16 kHz mono PCM ring buffer per [docs/decisions.md](docs/decisions.md) ADR-004. Push to a slot consumed by the (future) ASR worker.
3. **`cpal` audio playback.** `crates/embedded/src/adapters/audio_out.rs`. Drains a TTS PCM buffer to default output device.
4. **Sampler reuse.** Lift the `system_monitor.rs` thermal/CPU sampler from `tools/llm-benchmark/` into a shared module under `crates/embedded/src/observe/`. Keep it dormant on dev PC (zones differ); enable on target. Per [SCHEDULING.md](SCHEDULING.md) §7.

### Validation strategy

- **Dev PC:** smoke test only. Confirm tokens stream out of the LLM, audio in/out work with default devices, no crashes.
- **Target PC (later):** rerun `tools/llm-benchmark` to reconfirm the per-stage numbers in [SCHEDULING.md](SCHEDULING.md) §4 are achievable. Sweep `n_threads` 2–6 and pick the empirical sweet spot. Lock the result into `llm-benchmark.toml`.

**Done when:** dev-PC smoke passes; benchmarks queued for target-PC delivery.

---

## 6. Phase P5 — iGPU adapters (write-now, validate-on-target)

Vision and hearing via `ort` + Sherpa-ONNX, accelerated by OpenVINO targeting Vega 7. Per [docs/decisions.md](docs/decisions.md) ADR-003 and [docs/subsystems/perception.md](docs/subsystems/perception.md).

### Tasks

1. **ONNX vision adapter.** `crates/embedded/src/adapters/vision.rs`. Loads RetinaFace + ArcFace from `models/vision/`. Computes the four landmark metrics (brow, mouth, ocular aperture, gaze). Backend selection: OpenVINO if available, CPU otherwise. Write the OpenVINO selection behind a feature flag so dev PC can compile without it.
2. **Sherpa-ONNX hearing adapter.** `crates/embedded/src/adapters/hearing.rs`. Wraps SenseVoice; emits acoustic valence + arousal + AED tags. Same OpenVINO/CPU fallback pattern.
3. **Sherpa-ONNX TTS adapter.** `crates/embedded/src/adapters/tts.rs`. Piper PT-BR per [docs/subsystems/speech.md](docs/subsystems/speech.md). Implements `VoiceEmitter`. Queue + interrupt logic per the same doc.
4. **Backend selection module.** `crates/embedded/src/observe/backend.rs`. At startup, detect OpenVINO availability and the GPU device; log the chosen path. One source of truth for all three adapters.

### Validation strategy

- **Dev PC:** code compiles; runs against the installed weights with CPU EP; correctness asserted against fixture inputs ("does this image produce the expected face landmarks?"). Performance numbers from dev PC are explicitly **not** trusted.
- **Target PC:** the real validation. Run vision @ 15 FPS, hearing @ 30 FPS sustained, watch thermals. This is where [SCHEDULING.md](SCHEDULING.md) §4's iGPU contention rule (vision drops during TTS) gets verified.

**Done when:** dev-PC correctness tests pass on fixtures; target-PC validation queued.

---

## 7. Phase P6 — Hardware bridge (target only)

Requires the physical BOM in [docs/hardware.md](docs/hardware.md). None of this can be meaningfully done before the target PC + sensors arrive.

### Tasks

1. **Webcam capture.** Pick a Linux V4L2 crate (`v4l` or `nokhwa`); decide once target arrives. Feed frames to the vision adapter from P5.
2. **ReSpeaker DoA.** Spatial-source adapter — talks to the 4-Mic Array, exposes bearing. Cross-check with face pixel coords for the UPI binding from [docs/subsystems/perception.md](docs/subsystems/perception.md) §4.
3. **Arduino serial bridge.** `crates/embedded/src/adapters/eye_serial.rs`. Implements `EyeController` over a serial port (`serialport` crate). Bounded `crossbeam_channel` for backpressure per [SCHEDULING.md](SCHEDULING.md) §8.
4. **Arduino firmware.** New directory `firmware/uncanny-eyes/` (C, per [docs/decisions.md](docs/decisions.md) ADR-001). Receives commands, drives servo PWM. Out of scope for the Rust workspace; tracked in [docs/roadmap.md](docs/roadmap.md).

### Validation strategy

- Bring up each device individually (manual smoke).
- Run sim adapters in parallel with real adapters and diff the command streams as a sanity check on the new I/O layer.

---

## 8. Phase P7 — Scheduler glue (target only)

Wire everything into the topology in [SCHEDULING.md](SCHEDULING.md) §3. Until target HW exists, only structural code is meaningful — actual thread counts and pinning are guesses.

### Tasks

1. **Thread launcher.** `crates/embedded/src/runtime/launcher.rs`. Starts the threads from [SCHEDULING.md](SCHEDULING.md) §3, each owning one adapter, each writing to one slot. Domain loop runs in its own thread at 20 Hz.
2. **Pipeline scheduler** (vision-during-TTS gate). `crates/embedded/src/runtime/coscheduling.rs`. Implements [SCHEDULING.md](SCHEDULING.md) §4's "vision drops to 5 FPS while TTS renders" rule. State machine; one `AtomicU8` reading the current iGPU consumer.
3. **Backpressure policy.** `crates/embedded/src/runtime/backpressure.rs`. Implements the drop rules in [SCHEDULING.md](SCHEDULING.md) §6. Counters surfaced via the observability sampler.
4. **Cancellation.** Hook an `AtomicBool` into the LLM token callback for mid-utterance interrupt per [SCHEDULING.md](SCHEDULING.md) §8 and [docs/subsystems/speech.md](docs/subsystems/speech.md) §Queue & Interrupt.

**Done when:** the binary reaches the 800 ms end-to-end budget on target hardware, with thermals stable for ≥ 15 min of conversation.

---

## 9. Phase P8 — Soak & tune (target only)

This is the part that actually answers the open questions in [SCHEDULING.md](SCHEDULING.md) §9 and [docs/constraints.md](docs/constraints.md) §Open.

### Tasks

1. **Thermal soak.** 30-min continuous conversation. Sample CPU temp, freq per core, mem, iGPU busy. Verify the §6 degradation triggers fire correctly when temp > 90 °C. Confirm the actual 7430U `tjmax` and back off ~10 °C as suggested in [SCHEDULING.md](SCHEDULING.md) §9.
2. **Latency budget breakdown.** Fill in the per-stage hard caps in [docs/constraints.md](docs/constraints.md) §Open and [SCHEDULING.md](SCHEDULING.md) §4. Drive from real measurements, not estimates.
3. **Memory ceiling check.** Confirm the §5 estimate. Adjust KV cache size to fit the picked context length.
4. **Promote resolved opens to ADRs.** Per the convention in [SCHEDULING.md](SCHEDULING.md) §9. Each answered question becomes a one-paragraph ADR in [docs/decisions.md](docs/decisions.md).

---

## 10. Things to **not** do until target HW arrives

A short list because it's tempting and wrong.

- Don't tune `n_threads` on the dev PC. The 7430U's memory bandwidth is the bottleneck, not core count; dev-PC sweeps are noise.
- Don't pick a final OpenVINO version or build flags. Vega 7 paths are AMD-iGPU-specific; dev-PC dGPU/iGPU answers won't transfer.
- Don't write thermal-throttle policy code with hard temp thresholds. The 90 °C number in [SCHEDULING.md](SCHEDULING.md) §7 is a placeholder until target `tjmax` is read.
- Don't pin threads to cores in production code. Document intent only.
- Don't lock the LLM model. Per ADR-002 caveat in [docs/decisions.md](docs/decisions.md), final model selection waits for target-PC benchmark sweep.
- Don't optimize allocations or KV cache sizes preemptively. Dev PC won't surface the real pressure.

If a task feels like it needs target-PC numbers to be meaningful, **stop and put it in a "target-only" backlog** instead of guessing.

---

## 11. Repo deltas to land in P1 (concrete)

These are the actual file creations that turn this plan into running code. Aiming for one PR.

- `crates/core/Cargo.toml` — add `serde`, `thiserror`, `arc-swap`, `crossbeam-channel`.
- `crates/core/src/lib.rs` — re-export modules.
- `crates/core/src/types.rs` — `PerceptionPacket`, `SocialContext`, `EmotionTint`.
- `crates/core/src/error.rs` — `DomainError` with `thiserror`.
- `crates/core/src/slots.rs` — `Slot<T>` wrapping `ArcSwap`.
- `crates/core/src/ports/{mod,eye,voice,perception}.rs` — trait definitions.
- `crates/core/src/perception/aggregator.rs` — stub with `tick()` signature.
- `crates/core/src/fusion/{mod,engine,weights,hysteresis,decay}.rs` — stubs.
- `crates/core/src/behavior/mapper.rs` — stub.
- `crates/core/src/runtime.rs` — domain loop scaffold.
- `crates/core/tests/golden.rs` — load fixture, assert behavior stream.
- `crates/core/tests/fixtures/sarcasm.jsonl` — first hand-authored trace.
- `crates/embedded/Cargo.toml` — add (only) `cpal` for now; rest deferred.
- `crates/godot/src/stub/{eye,voice,sources}.rs` — print-stdout stubs.
- `crates/godot/src/bin/sim.rs` — wires stubs through the core runtime.

After P1 lands, [docs/roadmap.md](docs/roadmap.md) flips three rows from "designed" to "in progress" and the "first sim adapter" Next-concrete-step is checked off.

---

## 12. Cross-references

- [README.md](README.md) — project intent.
- [docs/index.md](docs/index.md) — full doc nav.
- [docs/architecture.md](docs/architecture.md) — hexagonal layers and tick rates.
- [docs/roadmap.md](docs/roadmap.md) — component status table; update as phases land.
- [docs/constraints.md](docs/constraints.md) — testable targets.
- [docs/hardware.md](docs/hardware.md) — target machine spec.
- [docs/decisions.md](docs/decisions.md) — ADRs to extend as opens close.
- [docs/subsystems/perception.md](docs/subsystems/perception.md), [fusion.md](docs/subsystems/fusion.md), [behavior.md](docs/subsystems/behavior.md), [speech.md](docs/subsystems/speech.md) — subsystem contracts.
- [docs/testing.md](docs/testing.md) — sim vs benchmark.
- [docs/benchmark.md](docs/benchmark.md) — LLM benchmark CLI.
- [SCHEDULING.md](SCHEDULING.md) — scheduling, threading, thermal, memory budget.