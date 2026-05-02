# Roadmap

Status snapshot. Update as components move stages.

| Component | Status | Notes |
|---|---|---|
| LLM benchmark CLI | **done** | `tools/llm-benchmark/src/main.rs` — runs benchmark workloads against any configured GGUF model. See [benchmark.md](benchmark.md). |
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
| LLM model decision | open | Final production model selection is still pending validation against hardware and behavior goals. |

## Next concrete steps

Phase ordering is canonical in [../plan.md](../plan.md); this list is the short-horizon view.

1. Buy ReSpeaker + Arduino + servos.
2. Bench candidate models on the baseline PC in [hardware.md](hardware.md) and compare latency/stability tradeoffs.
3. Stub `src/ports/{eye,voice}.rs` with the trait definitions from [subsystems/behavior.md](subsystems/behavior.md).
4. First sim adapter (P3 in [../plan.md](../plan.md)): print-to-stdout `EyeController` so fusion can be exercised end-to-end without hardware. Deferrable until target HW arrives.
