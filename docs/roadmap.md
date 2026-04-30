# Roadmap

Status snapshot. Update as components move stages.

| Component | Status | Notes |
|---|---|---|
| LLM benchmark CLI | **done** | `src/main.rs` — runs Llama-3.2-1B Q4_K_M. See [benchmark.md](benchmark.md). |
| Hardware procurement | TBD | PC owned. ReSpeaker, webcam, Arduino, servos: not bought. |
| Perception adapter — vision | designed | RetinaFace + ArcFace via `ort`. No code yet. |
| Perception adapter — acoustic | designed | SenseVoice via `sherpa-onnx`. No code yet. |
| Perception adapter — spatial (DoA) | designed | ReSpeaker beamforming. No code yet. |
| Perception aggregator | designed | Builds `PerceptionPacket`. No code yet. |
| Fusion engine | designed | Weighted aggregation + dissonance. No code yet. |
| Behavior layer + ports | designed | `EyeController`, `VoiceEmitter` traits. No code yet. |
| Speech adapter (TTS) | designed | Piper via Sherpa-ONNX. No code yet. |
| Eye / motor adapter (Arduino) | designed | Serial bridge. No code, no firmware. |
| Godot sim | not started | Empty section in earlier doc. Needs scene + adapter stubs. |
| LLM model decision | open | ADR-002 picks Qwen 2.5 3B; benchmark still on Llama 1B. Reconcile. |

## Next concrete steps

1. Buy ReSpeaker + Arduino + servos.
2. Bench Qwen 2.5 3B Q4_K_M on the baseline PC in [hardware.md](hardware.md) to confirm ADR-002 (Docker is optional test tooling only).
3. Stub `src/ports/{eye,voice}.rs` with the trait definitions from [subsystems/behavior.md](subsystems/behavior.md).
4. First sim adapter: print-to-stdout `EyeController` so fusion can be exercised end-to-end without hardware.
