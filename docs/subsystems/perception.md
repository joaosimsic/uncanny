# Perception

Detection layer. Four streams (acoustic, visual, semantic, spatial) feed a unified `PerceptionPacket` consumed by [fusion.md](fusion.md).

## 1. Acoustic Stream (Prosody & Tone)
Real-time emotional "vibes" — *how* words are spoken.

- **Model:** `SenseVoice-Small` via `sherpa-onnx`.
- **Input:** 16kHz mono PCM via `cpal`, sliding-window buffer.
- **Features:**
    - **Emotional tokens:** parse `<|HAPPY|>`, `<|SAD|>`, `<|ANGRY|>`, `<|NEUTRAL|>`.
    - **Audio Event Detection (AED):** non-verbal cues — laughter, sighs, crying, heavy breathing.
- **Output:** normalized `acoustic_valence` (positivity) + `acoustic_arousal` (energy).
- **Locale:** PT phonemes prioritized.

---

## 2. Visual Stream (Kinetics & Facial Geometry)
Landmark geometry over black-box classifier — keeps latency low on the iGPU.

- **Model:** RetinaFace (detection) + 5-point facial landmarks.
- **Runtime:** `ort` (ONNX Runtime) with OpenVINO acceleration.
- **Metrics:**
    - **Brow furrow:** vertical distance between inner eyebrows (confusion / anger).
    - **Mouth curvature:** lip-corner height relative to nose tip (smile / frown).
    - **Ocular aperture:** vertical eye opening (surprise / suspicion).
    - **Gaze vector:** eye contact vs aversion.
- **Identity:** ArcFace embeddings for persistence across frames (re-recognize a user who left and returned).

---

## 3. Semantic Stream (Textual Intent)
Logical meaning of transcribed text.

- **Model:** Qwen 2.5 3B via `llama-cpp-2` (see [decisions.md](../decisions.md) ADR-002).
- **Method:** sentiment extraction during the Thinking phase.
- **Analysis:**
    - Polarity (positive vs negative words).
    - Sarcasm / irony detection by cross-checking text against acoustic tone (see [fusion.md](fusion.md) §Incongruence).

### Cadence & freshness

Unlike acoustic/visual (continuous, high-rate), semantic is **event-driven**: a value lands only after the LLM finishes a Thinking pass on a completed utterance — typically every few seconds, not every 100 ms. The aggregator therefore cannot assume a fresh semantic value on every tick.

The adapter exposes:

```rust
trait SemanticSource {
    fn latest_valence(&self) -> Option<f32>;   // None until first utterance is scored
    fn last_update(&self) -> Option<Instant>;  // monotonic time of last successful pass
}
```

The aggregator copies `latest_valence` into `PerceptionPacket::semantic_valence` verbatim and converts `last_update` into `semantic_age_ms`. Staleness gating (i.e., "this signal is too old to weight") is **fusion's** call — see [fusion.md](fusion.md) §Staleness — keeping the aggregator a dumb projector.

> Raw transcribed text never leaves the adapter. The domain core sees a scalar valence only; LLM tokenization, prompting, and parsing all stay below the port boundary per [architecture.md](../architecture.md).

---

## 4. Spatial Stream (Direction of Arrival)
Where is the speaker?

- **Hardware:** ReSpeaker 4-Mic Array.
- **Method:** DoA from mic-array beamforming, fused with RetinaFace pixel coords.
- **Output:** angular bearing of the active speaker → drives head-turn motor commands and the **Unified Person Identification (UPI)** binding (DoA + face → one person ID).

---

## 5. Optimization

- **Threading:** vision and hearing in dedicated threads. Vision @ 15 FPS, acoustic @ 30 FPS (see [constraints.md](../constraints.md)).
- **Offloading:**
    - Vision + hearing: RX Vega 7 iGPU via OpenVINO.
    - Thinking: CPU (Ryzen 5) — keeps iGPU VRAM free.

---

## 6. Output Contract — `PerceptionPacket`

Emitted every 100ms. Canonical struct used throughout the domain layer:

```rust
struct PerceptionPacket {
    acoustic_valence: f32,         // -1.0..=1.0
    acoustic_arousal: f32,         //  0.0..=1.0
    visual_valence: f32,           // -1.0..=1.0
    user_engagement: f32,          //  0.0..=1.0  (gaze on/off camera)
    semantic_valence: Option<f32>, // -1.0..=1.0; None until first LLM pass
    semantic_age_ms: u32,          //  ms since last semantic update; u32::MAX if never
    doa_bearing: Option<f32>,      //  radians; None if no clear source
    user_id: Option<u32>,          //  UPI binding
    timestamp_secs: f64,           //  seconds since aggregator epoch (serializable)
}
```

### Field semantics

| Field | Cadence | None / sentinel meaning |
|---|---|---|
| acoustic_* | every tick (10 Hz) | always populated; silence ⇒ valence ≈ 0, arousal ≈ 0 |
| visual_* | every tick | always populated; no face ⇒ valence ≈ 0, engagement = 0 |
| semantic_valence | event-driven (per utterance) | `None` ⇒ no LLM pass has completed yet |
| semantic_age_ms | every tick | `u32::MAX` ⇒ never updated; otherwise ms since last update |
| doa_bearing | event-driven (per voiced segment) | `None` ⇒ no clear source localized |
| user_id | event-driven (UPI) | `None` ⇒ unbound |
| timestamp_secs | every tick | seconds since aggregator construction (monotonic) |
