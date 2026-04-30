# Model weights (`models/`)

Git tracks **this layout** and small marker files; **large weights are not committed.**

## What is ignored

Configured in the repo root `.gitignore`:

- `models/**/*.gguf` — quantized LLM weights for **llama.cpp** (same GGUF format family used by `llama-cpp-2` / `tools/llm-benchmark`).
- `models/**/*.onnx` — vision, hearing, and speech networks for **ONNX Runtime** (`ort`), Sherpa-ONNX, and OpenVINO (see [docs/decisions.md](../docs/decisions.md) ADR-003).
- `models/**/*.tar.bz2` — optional SenseVoice (and similar) bundles after download; extracted folders contain ONNX and tokens under `models/hearing/`.

Companion files downloaded next to ONNX (for example Piper `*.onnx.json`) are small and **are** tracked if you add them by hand; the install script places them under `models/speech/`.

## Layout vs documentation

| Directory | Role | Documented in |
|-----------|------|----------------|
| `llm/` | **Qwen 2.5 3B Instruct Q4_K_M** GGUF — “Thinking” / benchmark | [docs/decisions.md](../docs/decisions.md) ADR-002, [perception.md](../docs/subsystems/perception.md) §Semantic |
| `vision/` | Face detection + identity embeddings (+ optional emotion stack later) | [perception.md](../docs/subsystems/perception.md) §Visual — RetinaFace-class detection, ArcFace; MiniXception optional |
| `hearing/` | **SenseVoice** ASR + emotion / AED (Sherpa-ONNX layout) | ADR-003, [perception.md](../docs/subsystems/perception.md) §Acoustic |
| `speech/` | **Piper** TTS PT-BR (Sherpa-ONNX / Piper ONNX) | ADR-003, [speech.md](../docs/subsystems/speech.md) |

**Vision naming:** [perception.md](../docs/subsystems/perception.md) specifies RetinaFace for detection. The install script currently pulls a standard **UltraFace** RFB-320 detector from the [ONNX Model Zoo](https://github.com/onnx/models) plus **ArcFace** ResNet100 — both ONNX — as an offline-friendly baseline until a RetinaFace ONNX export is pinned for this repo.

## Install

From the repository root:

```bash
./models/install-models.sh
```

Requirements: `curl`, `tar`, `bzip2`. Re-running skips files that already exist.

## Runtime stacks (reminder)

- **LLM:** GGUF + llama.cpp (Rust via `llama-cpp-2`).
- **Everything else:** ONNX via `ort`, Sherpa-ONNX for speech pipelines, OpenVINO as in ADR-003.
