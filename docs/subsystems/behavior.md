# Behavior

Output side of the hexagonal core. Maps `SocialContext` (from [fusion.md](fusion.md)) to motor and voice commands via ports.

> Status: **design only.** Will live in `src/ports/`.

---

## Ports

### `trait EyeController`
Drives servos for eye + eyelid + eyebrow movement (Arduino-side actuation).

```rust
trait EyeController {
    fn look_at(&mut self, x: f32, y: f32);   // angular target, radians
    fn blink(&mut self);                     // single full blink
    fn saccade(&mut self);                   // micro-jitter, idle filler
}
```

### `trait VoiceEmitter`
Drives TTS (see [speech.md](speech.md)).

```rust
trait VoiceEmitter {
    fn speak(&mut self, text: &str, emotion_tint: EmotionTint);
}
```

### Adapter targets
- Real: serial bridge to Arduino (servos) + Sherpa-ONNX/Piper TTS.
- Sim: Godot stub (see [../architecture.md](../architecture.md) §Simulation).

---

## State → Behavior Mapping

| `SocialContext` cue | Behavior |
|---|---|
| `dissonance > 0.6` | Analytical Stare — slow head-tilt, reduced blink rate |
| `is_active = false` | Idle — random saccades, occasional blink |
| `engagement < 0.3` | Search — head sweep toward last DoA bearing |
| Aligned + positive valence | Delayed mechanical mimicry (smile servo) |
| Aligned + negative valence | Mirrored brow furrow, lowered gaze |

---

## Notes
- Motor command rate gated by hysteresis in fusion (300ms minimum hold).
