# Decisions (ADRs)

Light-weight architecture decision log. Each entry: **what, why, status.**

---

## ADR-001: Rust core + C for Arduino

**Decision:** Domain core (perception aggregator, fusion, behavior, speech adapter) in **Rust**. Arduino firmware in **C**.

**Why:**
- Rust gives memory safety, ergonomic concurrency, mature ONNX (`ort`) and llama (`llama-cpp-2`) bindings — without C++.
- Arduino ecosystem is C-native; no upside to forcing Rust on the µC.
- Avoids Python (deployment + perf issues) and C++ (ergonomics).

**Status:** accepted.

---

## ADR-002: LLM model — Qwen 2.5 3B Q4_K_M

**Decision:** Production "Thinking" model is **Qwen 2.5 3B Instruct Q4_K_M** via `llama-cpp-2`.

**Why:**
- Larger context-handling capacity than 1B models, important for KV-cache-managed conversational state.
- Good multilingual coverage including Portuguese.
- Falls within the 4 GB RAM budget at Q4.

**Caveat:** the [benchmark CLI](benchmark.md) currently runs **Llama-3.2-1B**. That benchmark predates this ADR; rerun against Qwen 3B to confirm fit on the Docker `cpus: 2.0 / mem_limit: 4g` profile. If Qwen 3B blows the budget, fall back to Qwen 2.5 1.5B before reverting to Llama.

**Status:** accepted, validation pending.

---

## ADR-003: ONNX (`ort` + OpenVINO) for vision / hearing

**Decision:**
- Vision: RetinaFace + ArcFace + (optional) MiniXception via `ort`.
- Hearing: SenseVoice-Small via `sherpa-onnx` (also ONNX-based).
- Speech: Piper via Sherpa-ONNX.
- Acceleration: OpenVINO targeting the RX Vega 7 iGPU.

**Why:**
- Keeps the CPU free for the LLM (which is CPU-bound under llama.cpp).
- Single runtime family (ONNX) for all non-LLM models simplifies adapter code.
- OpenVINO has documented AMD-iGPU paths.

**Status:** accepted.

---

## ADR-004: Audio I/O — 16 kHz mono PCM via `cpal`

**Decision:** Capture and feed at **16 kHz mono PCM** via the `cpal` Rust crate, sliding-window buffer.

**Why:**
- Matches SenseVoice native input rate (no resample).
- `cpal` is the cross-platform standard in Rust audio.
- Sliding window enables full-duplex without chunked-utterance lag.

**Status:** accepted.
