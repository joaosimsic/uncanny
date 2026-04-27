FROM rust:1.86-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    cmake \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /opt
RUN git clone https://github.com/ggerganov/llama.cpp.git
WORKDIR /opt/llama.cpp
ARG CMAKE_BUILD_PARALLEL_LEVEL=2
RUN cmake -B build -DGGML_OPENMP=ON && cmake --build build --config Release -j${CMAKE_BUILD_PARALLEL_LEVEL}

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libgomp1 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /opt/llama.cpp/build/bin/ /app/bin/
COPY --from=builder /app/target/release/ryzen5-llama-env /app/ryzen5-llama-env

ENV LD_LIBRARY_PATH=/app/bin
ENTRYPOINT ["/app/ryzen5-llama-env"]
