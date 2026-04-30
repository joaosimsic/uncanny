# LLM Benchmark Runner

Local CLI wrapper around `llama.cpp` to bench a 4-bit quantized model on the baseline hardware in [hardware.md](hardware.md) (Ryzen 5 7430U / 16 GB RAM / iGPU-only). Used to validate the "Thinking" subsystem candidate model before integrating into the full pipeline.

Currently runs **Llama-3.2-1B-Instruct Q4_K_M**. Target model for the production system is Qwen 2.5 3B Q4_K_M — see [decisions.md](decisions.md) ADR-002.

Docker here is test infrastructure only. It is not a source of production constraints.

## Requirements

- Docker + Docker Compose (optional, for reproducible test runs), or
- Rust + C++ build toolchain (local run)
- `curl` (model download)

## 1) Download the 4-bit model

```bash
make download-model
```

Drops `models/Llama-3.2-1B-Instruct-Q4_K_M.gguf`.

## 2) Run in container (test harness)

```bash
make run-docker
```

One-shot prompt with safer defaults.

### Interactive chat + live resource usage

```bash
make run-chat-docker
```

- multi-turn chat (default)
- `--monitor` shows CPU/RAM of model process

## 3) Run locally

Needs `llama-cli` at `./bin/llama-cli` (from `llama.cpp`):

```bash
make run-local
```

Local interactive chat with monitoring:

```bash
make run-chat-local
```

## Custom prompt

```bash
docker compose run --rm llama-runner \
  --model /app/models/Llama-3.2-1B-Instruct-Q4_K_M.gguf \
  --prompt "Write a short poem about robotics." \
  --n-predict 120
```

Interactive monitored:

```bash
docker compose run --rm llama-runner \
  --monitor \
  --monitor-interval-secs 2
```

One-shot completion mode:

```bash
docker compose run --rm llama-runner --no-chat --prompt "Summarize Rust in one paragraph."
```

## Notes

- Quantization: `Q4_K_M` (4-bit family).
- Performance varies with host CPU and Docker version.
- Monitoring uses Linux `/proc` stats from the container/process.

Production sizing and runtime constraints are defined by the target machine in [hardware.md](hardware.md) and [constraints.md](constraints.md), not by container settings.

## Low-spec safe mode (first run)

If desktop freezes/reboots during build/run, stage:

1. Constrained compile parallelism:

```bash
make build-safe-image
```

2. Conservative one-shot:

```bash
make run-safe
```

3. If stable, raise one parameter at a time:

- `--threads`: `2 → 3 → 4`
- `--ctx-size`: `1024 → 1536 → 2048`

4. Watch host pressure:

```bash
htop
journalctl -k -b -f
```
