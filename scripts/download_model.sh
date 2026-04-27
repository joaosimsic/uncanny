#!/usr/bin/env bash
set -euo pipefail

MODEL_DIR="${1:-models}"
MODEL_NAME="Llama-3.2-1B-Instruct-Q4_K_M.gguf"
MODEL_URL="https://huggingface.co/bartowski/Llama-3.2-1B-Instruct-GGUF/resolve/main/${MODEL_NAME}?download=true"

mkdir -p "${MODEL_DIR}"
echo "Downloading ${MODEL_NAME} into ${MODEL_DIR}..."
curl -L "${MODEL_URL}" -o "${MODEL_DIR}/${MODEL_NAME}"
echo "Done."
