# Constraints

Testable targets. Update as measured.

| Constraint | Target | Rationale |
|---|---|---|
| Boot time | ≤ 60 s | Plug-and-play. Power-on to listening within a minute. |
| End-to-end latency | ≤ 800 ms | User-speech-end → robot-speech-start. Anything slower kills conversational flow. |
| Vision tick | 15 FPS | Sufficient for micro-expression detection, low iGPU pressure. |
| Acoustic tick | 30 FPS | Sliding window over 16 kHz PCM. |
| Perception emit | 10 Hz (100 ms) | Aggregation cadence to fusion. |
| Fusion tick | 20 Hz (50 ms) | Smooth motor commands without flicker. |
| Hysteresis hold | 300 ms | Minimum emotion-state persistence before motor change. |
| Mood decay | 10 % / s | Decay toward neutral when no input. |
| RAM budget | ≤ 4 GB resident | Matches `docker-compose.yml` `mem_limit: 4g`. |
| CPU budget | ≤ 2 cores active for LLM | Matches `docker-compose.yml` `cpus: 2.0`. |
| Primary language | Portuguese | English support possibly later. |

## Open
- Motor command rate cap.
- Per-stage latency budget breakdown (ASR / fusion / LLM / TTS).
