# uncanny — overview

Robot head with human expression and AI-powered talk. Mechanically exposed, behaviorally human-mimic. Designed to sit in the uncanny valley — interaction feels uncomfortable by design.

Primary language: Portuguese (English support possibly later).

## Nav

| Doc | What |
|---|---|
| [architecture.md](architecture.md) | Hexagonal layers, data flow, tick rates, threading |
| [subsystems/perception.md](subsystems/perception.md) | Vision + Hearing + Semantic input → `PerceptionPacket` |
| [subsystems/fusion.md](subsystems/fusion.md) | `PerceptionPacket` → `SocialContext` (incongruence detection) |
| [subsystems/behavior.md](subsystems/behavior.md) | Motor/voice ports — `EyeController`, `VoiceEmitter` |
| [subsystems/speech.md](subsystems/speech.md) | TTS via Sherpa-ONNX + Piper |
| [hardware.md](hardware.md) | BOM + rationale |
| [constraints.md](constraints.md) | Testable targets (latency, FPS, memory) |
| [decisions.md](decisions.md) | ADRs (Rust+C, Qwen vs Llama, ONNX stack) |
| [glossary.md](glossary.md) | UPI, DoA, AED, SER, Q4_K_M, etc. |
| [roadmap.md](roadmap.md) | Component status table |
| [research.md](research.md) | Reference links and inspiration |
| [benchmark.md](benchmark.md) | LLM benchmark CLI (currently the only working code) |
