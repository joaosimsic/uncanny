# Fusion

Domain-layer "Social Truth" processor. Reads `PerceptionPacket` from [perception.md](perception.md), emits `SocialContext` to [behavior.md](behavior.md). Hexagonal core — no I/O, no hardware coupling.

---

## 1. Goal
Identify **congruence** — alignment between what a person says and how they act.

---

## 2. Weighted Aggregation
Inputs are not equal. Weights are dynamic, scaled by adapter confidence scores.

### Valence (positivity)
Weighted average of three streams:

| Stream | Initial weight |
|---|---|
| Acoustic | 45% |
| Semantic | 35% |
| Visual | 20% |

> Initial weights are guesses. Tune empirically on real conversations.

### Arousal (energy)
Driven primarily by SenseVoice (volume / speed) plus facial-landmark jitter (trembling, wide eyes).

---

## 3. Incongruence Detector
Most critical "uncanny" feature.

| Scenario | Semantic | Acoustic | State | Robot behavior |
|---|---|---|---|---|
| Sarcasm | Positive ("Great.") | Flat / negative | Conflict | Slow head-tilt stare |
| Suppressed anger | Neutral ("Fine.") | High energy / tense | Conflict | Rapid blinking, slight eye narrowing |
| Genuine joy | Positive | Positive | Aligned | Delayed mechanical mimicry |

### Dissonance factor

```
D = |V_acoustic − V_semantic|
```

If `D > 0.6`, trigger **Analytical Stare** state — prioritize visual observation over conversational flow.

---

## 4. State Persistence

- **Hysteresis:** detected-emotion change must persist > 300ms before motor commands fire. Avoids flicker.
- **Mood decay:** if user leaves frame or stops talking, perceived mood decays toward neutral at 10%/s.

---

## 5. Output Contract — `SocialContext`

```rust
struct SocialContext {
    fused_valence: f32,    // combined positivity, -1.0..=1.0
    fused_arousal: f32,    // combined energy,      0.0..=1.0
    dissonance: f32,       // |V_a − V_s|,          0.0..=1.0
    engagement: f32,       // gaze-on-me probability
    is_active: bool,       // valid human target present
}
```

---

## 6. Performance

- **Tick:** 20Hz (every 50ms). See [constraints.md](../constraints.md).
- **No I/O wait:** reads latest `PerceptionPacket` from an atomic snapshot updated by detection adapters.
- **Determinism:** same input → same output. Drives Godot-sim testability.
