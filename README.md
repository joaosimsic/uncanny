# Ryzen 5 + 8GB Llama Environment (Rust)

This repository runs a **Llama-3.2-1B Instruct** model quantized to **4-bit** (`Q4_K_M`) using a Rust CLI wrapper around `llama.cpp`.

It also includes a Docker profile constrained to:

- `cpus: 2.0` (safer default profile)
- `mem_limit: 4g`

## Requirements

- Docker + Docker Compose (recommended)
- or Rust + C++ build toolchain (for local run)
- `curl` (for model download)

## 1) Download the 4-bit model

```bash
make download-model
```

This downloads:

- `models/Llama-3.2-1B-Instruct-Q4_K_M.gguf`

## 2) Run in constrained environment (recommended)

```bash
make run-docker
```

This starts the app in a container with safer defaults and runs a short one-shot prompt.

### Interactive chat + live resource usage

```bash
make run-chat-docker
```

This enables:

- multi-turn chat (default behavior)
- live monitor output (`--monitor`) showing CPU and RAM usage of the model process

## 3) Run locally (without container limits)

You need `llama-cli` in `./bin/llama-cli` (from `llama.cpp`), then:

```bash
make run-local
```

Local interactive chat with monitoring:

```bash
make run-chat-local
```

## Custom prompt

Use Docker directly to pass your own prompt:

```bash
docker compose run --rm llama-runner \
  --model /app/models/Llama-3.2-1B-Instruct-Q4_K_M.gguf \
  --prompt "Write a short poem about robotics." \
  --n-predict 120
```

For interactive monitored mode:

```bash
docker compose run --rm llama-runner \
  --monitor \
  --monitor-interval-secs 2
```

To disable chat and run as one-shot completion mode:

```bash
docker compose run --rm llama-runner --no-chat --prompt "Summarize Rust in one paragraph."
```

## Notes

- Quantization format here is `Q4_K_M` (4-bit family).
- Depending on host CPU capabilities and Docker version, exact performance may vary.
- Monitoring uses Linux `/proc` stats from inside the container/process.

## Low-spec safe mode (recommended first run)

If your desktop freezes or reboots while building/running, use this staged flow:

1. Build with constrained compile parallelism:

```bash
make build-safe-image
```

2. Run a conservative one-shot profile:

```bash
make run-safe
```

3. If stable, increase one parameter at a time:

- `--threads`: `2 -> 3 -> 4`
- `--ctx-size`: `1024 -> 1536 -> 2048`

4. Monitor host pressure during tests:

```bash
htop
```

```bash
journalctl -k -b -f
```
