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
    acoustic_valence: f32,   // -1.0..=1.0
    acoustic_arousal: f32,   //  0.0..=1.0
    visual_valence: f32,     // -1.0..=1.0
    user_engagement: f32,    //  0.0..=1.0  (gaze on/off camera)
    doa_bearing: Option<f32>,// radians; None if no clear source
    user_id: Option<u32>,    // UPI binding
    timestamp: std::time::Instant,
}
```
