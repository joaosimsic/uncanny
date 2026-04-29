# Emotion Perception

## 1. Acoustic Stream (Prosody & Tone)
The primary channel for real-time emotional "vibes," focusing on how words are spoken.

* **Model:** `SenseVoice-Small` (Running via `sherpa-onnx`).
* **Input:** 16kHz Mono PCM via `cpal` (Sliding window buffer).
* **Detection Features:**
    * **Emotional Tokens:** Immediate parsing of `<|HAPPY|>`, `<|SAD|>`, `<|ANGRY|>`, `<|NEUTRAL|>` tags.
    * **Audio Event Detection (AED):** Identification of non-verbal cues: laughter, sighs, crying, or heavy breathing.
* **Processing:** Raw tokens are converted into a normalized **Arousal** (energy) and **Valence** (positivity) score.

---

## 2. Visual Stream (Kinetics & Facial Geometry)
Focuses on physical markers. Rather than a "black-box" classifier, it uses landmark geometry to ensure low latency on the iGPU.

* **Model:** `RetinaFace` (Detection) + 5-point facial landmarking.
* **Implementation:** `ort` (ONNX Runtime) with OpenVINO acceleration.
* **Key Geometric Metrics:**
    * **Brow Furrow:** Vertical distance between inner eyebrows (indicates confusion or anger).
    * **Mouth Curvature:** Calculation of lip corner height relative to the nose tip (indicates smile/frown).
    * **Ocular Aperture:** Vertical eye opening (indicates surprise or suspicion).
    * **Gaze Vector:** Determining if the user is maintaining eye contact or averting their gaze.

---

## 3. Semantic Stream (Textual Intent)
Analyzes the logical meaning and "weight" of the transcribed text.

* **Model:** `Qwen 2.5 3B` (via `llama-cpp-2`).
* **Method:** Sentiment extraction performed during the "Thinking" phase.
* **Contextual Analysis:**
    * Detection of "Polarity" (positive vs. negative words).
    * Identification of sarcasm or irony by comparing text complexity against the Acoustic Stream's tone.

---

## 4. Technical Implementation & Optimization

### Latency Strategy
* **Asynchronous Buffering:** Vision and Hearing run in dedicated threads. The Vision stream targets **15 FPS** (sufficient for micro-expressions), while the Acoustic stream processes at **30 FPS**.
* **Hardware Offloading:**
    * **Vision/Hearing:** Execution on the RX Vega 7 iGPU via OpenVINO.
    * **Thinking:** Execution on CPU (Ryzen 5) to prevent iGPU VRAM starvation.

### Data Structure (Output to Core)
The detection layer produces a standardized `PerceptionPacket` every 100ms:

```rust
struct PerceptionPacket {
    acoustic_valence: f32, // -1.0 to 1.0
    acoustic_arousal: f32, // 0.0 to 1.0
    visual_valence: f32,   // -1.0 to 1.0
    user_engagement: f32,  // 0.0 to 1.0 (Gaze)
    timestamp: std::time::Instant,
}
```

---

## 5. Integration Notes
* **Portuguese Support:** SenseVoice is specifically configured to prioritize Portuguese phonemes for higher accuracy in tone detection within the local language.
