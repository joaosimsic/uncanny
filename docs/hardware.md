# Hardware

## PC

| Part | Spec | Why |
|---|---|---|
| CPU | Ryzen 5 7430U | 6c/12t Zen 3 mobile. Adequate for 3B-param LLM at Q4. |
| RAM | 16 GB DDR4 | Headroom for KV cache + perception threads + OS. |
| Storage | 512 GB SSD | Model files (~2 GB each) + local build/runtime artifacts. |
| iGPU | RX Vega 7 | OpenVINO target for vision/hearing ONNX inference. Keeps CPU free for LLM. |

Plug-and-play constraint (see [constraints.md](constraints.md)) means everything runs locally. No GPU expansion planned.

---

## Sensors

| Part | Why |
|---|---|
| Full-HD webcam | RetinaFace input. 1080p is overkill but stock; will downscale. |
| ReSpeaker 4-Mic Array | DoA support out of the box + 16kHz PCM. Avoids DIY beamforming. |

---

## Actuation

| Part | Why |
|---|---|
| Arduino (model TBD) | Servo PWM driver. Decouples real-time motor control from main CPU. |
| Servos | Eyelid, eyebrow, eye-pan/tilt, head-yaw. Count + spec TBD. |

Mechanical eye design reference: see [research.md](research.md).

---

## BOM Status
See [roadmap.md](roadmap.md) — only the PC is owned today.
