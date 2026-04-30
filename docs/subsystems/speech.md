# Speech (TTS)

Output voice. Driven by `VoiceEmitter::speak` from [behavior.md](behavior.md).

> Status: **design only.**

---

## Stack
- **Runtime:** Sherpa-ONNX via `ort`.
- **Model:** Piper TTS.
- **Voice:** PT-BR (English voice possibly added later — see [../constraints.md](../constraints.md)).

---

## Prosody Control
Voice is tinted by `SocialContext.fused_arousal` and `fused_valence`:

| Cue | Effect |
|---|---|
| High arousal | Faster rate, higher pitch variance |
| Low arousal | Slower, monotone |
| Negative valence | Lower fundamental, longer pauses |
| Aligned positive | Brighter timbre, slight pitch lift |

Implementation: pre-render Piper output, apply pitch/rate shift on the audio buffer before playback.

---

## Queue & Interrupt
- **Queue:** TTS requests are FIFO with priority hint.
- **Interrupt:** if a higher-priority response arrives mid-utterance (e.g. user starts speaking), cut the current playback at the next phoneme boundary.
- **Latency target:** ≤ 800ms from user-speech-end → robot-speech-start (see [../constraints.md](../constraints.md)).
