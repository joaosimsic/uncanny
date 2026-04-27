MODEL_PATH=models/Llama-3.2-1B-Instruct-Q4_K_M.gguf
SAFE_THREADS=2
SAFE_CTX_SIZE=1024
SAFE_N_PREDICT=64
SAFE_CPUS=2
SAFE_MEMORY=4g
BUILD_PARALLELISM=2

.PHONY: download-model build-safe-image run-docker run-chat-docker run-safe run-local run-chat-local

download-model:
	./scripts/download_model.sh models

build-safe-image:
	docker build --build-arg CMAKE_BUILD_PARALLEL_LEVEL=$(BUILD_PARALLELISM) -t robot-safe .

run-docker:
	docker compose run --rm llama-runner

run-chat-docker:
	docker compose run --rm llama-runner --monitor

run-safe:
	docker run --rm --cpus=$(SAFE_CPUS) --memory=$(SAFE_MEMORY) --memory-swap=$(SAFE_MEMORY) \
		-v "$(PWD)/models:/app/models:ro" robot-safe \
		--model /app/models/Llama-3.2-1B-Instruct-Q4_K_M.gguf \
		--no-chat --threads $(SAFE_THREADS) --ctx-size $(SAFE_CTX_SIZE) --n-predict $(SAFE_N_PREDICT) \
		--prompt "hello"

run-local:
	cargo run --release -- --model $(MODEL_PATH)

run-chat-local:
	cargo run --release -- --model $(MODEL_PATH) --monitor
