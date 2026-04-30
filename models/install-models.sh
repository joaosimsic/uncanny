#!/usr/bin/env bash
# Download model weights described in docs/decisions.md and subsystem docs.
# LLM: GGUF for llama.cpp. Vision/hearing/speech: ONNX (+ Piper JSON).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LLM_DIR="${ROOT}/models/llm"
VISION_DIR="${ROOT}/models/vision"
HEARING_DIR="${ROOT}/models/hearing"
SPEECH_DIR="${ROOT}/models/speech"

# --- Qwen 2.5 3B Instruct Q4_K_M (ADR-002) — GGUF for llama.cpp ---
QWEN_GGUF_URL="https://huggingface.co/bartowski/Qwen2.5-3B-Instruct-GGUF/resolve/main/Qwen2.5-3B-Instruct-Q4_K_M.gguf"
QWEN_GGUF_NAME="Qwen2.5-3B-Instruct-Q4_K_M.gguf"

# --- SenseVoice (perception acoustic stream) — Sherpa-ONNX bundle ---
SENSEVOICE_URL="https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17.tar.bz2"
SENSEVOICE_ARCHIVE="sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17.tar.bz2"

# --- Piper PT-BR (speech.md) — ONNX + config from rhasspy/piper-voices ---
PIPER_ONNX_URL="https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/pt/pt_BR/edresson/low/pt_BR-edresson-low.onnx"
PIPER_JSON_URL="https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/pt/pt_BR/edresson/low/pt_BR-edresson-low.onnx.json"

# --- Vision — ONNX Model Zoo (baseline until RetinaFace ONNX is pinned) ---
ULTRAFACE_URL="https://media.githubusercontent.com/media/onnx/models/main/validated/vision/body_analysis/ultraface/models/version-RFB-320.onnx"
ARCFACE_URL="https://media.githubusercontent.com/media/onnx/models/main/validated/vision/body_analysis/arcface/model/arcfaceresnet100-8.onnx"

download() {
  local url="$1"
  local dest="$2"
  if [[ -f "$dest" ]]; then
    echo "[skip] exists: $dest"
    return 0
  fi
  echo "[get] $url"
  mkdir -p "$(dirname "$dest")"
  curl -fL --retry 3 --retry-delay 2 -o "$dest" "$url"
}

mkdir -p "$LLM_DIR" "$VISION_DIR" "$HEARING_DIR" "$SPEECH_DIR"

download "$QWEN_GGUF_URL" "${LLM_DIR}/${QWEN_GGUF_NAME}"

download "$ULTRAFACE_URL" "${VISION_DIR}/version-RFB-320.onnx"
download "$ARCFACE_URL" "${VISION_DIR}/arcfaceresnet100-8.onnx"

download "$PIPER_ONNX_URL" "${SPEECH_DIR}/pt_BR-edresson-low.onnx"
download "$PIPER_JSON_URL" "${SPEECH_DIR}/pt_BR-edresson-low.onnx.json"

SENSE_DEST="${HEARING_DIR}/${SENSEVOICE_ARCHIVE}"
SENSE_ROOT="${HEARING_DIR}/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17"
if [[ -d "$SENSE_ROOT" ]]; then
  echo "[skip] SenseVoice extracted at $SENSE_ROOT"
elif [[ -f "$SENSE_DEST" ]]; then
  echo "[extract] $SENSE_DEST -> $HEARING_DIR"
  tar -xjf "$SENSE_DEST" -C "$HEARING_DIR"
else
  echo "[get] $SENSEVOICE_URL"
  download "$SENSEVOICE_URL" "$SENSE_DEST"
  echo "[extract] $SENSE_DEST -> $HEARING_DIR"
  tar -xjf "$SENSE_DEST" -C "$HEARING_DIR"
fi

echo "Done. LLM default for tools/llm-benchmark: models/llm/${QWEN_GGUF_NAME}"
