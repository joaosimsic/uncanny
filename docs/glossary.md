# Glossary

| Term | Meaning |
|---|---|
| **ADR** | Architecture Decision Record. See [decisions.md](decisions.md). |
| **AED** | Audio Event Detection — non-verbal cue identification (laughter, sigh, cry). |
| **ArcFace** | Face-recognition embedding model. Used here for user persistence across frames. |
| **Arousal** | Emotion-model axis: energy / intensity (0..1). Pairs with valence. |
| **DoA** | Direction of Arrival — angular bearing of a sound source from a mic array. |
| **FPS** | Frames per second. |
| **gguf** | Quantized model format used by `llama.cpp`. |
| **Hexagonal architecture** | Domain core + ports (traits) + adapters (impls). Hardware-agnostic core. |
| **iGPU** | Integrated GPU — here, the Vega 7 inside the Ryzen 5 7430U. |
| **KV cache** | Key/value attention cache in transformer LLMs. Memory cost grows with context length. |
| **MiniXception** | Lightweight CNN, candidate for visual emotion classification. |
| **ONNX** | Open Neural Network Exchange — portable model format. |
| **OpenVINO** | Intel's inference runtime. Provides AMD iGPU support via OpenCL backend. |
| **ort** | Rust crate wrapping ONNX Runtime. |
| **PCM** | Pulse-Code Modulation — raw audio samples. |
| **Q4_K_M** | 4-bit quantization variant in the gguf family. K-quant medium. |
| **ReSpeaker** | 4-mic array with on-board DoA processing. |
| **RetinaFace** | Face detection + landmark model. |
| **Saccade** | Rapid micro-movement of eye between fixation points. |
| **SenseVoice** | Multilingual ASR + emotion + AED model from FunAudioLLM. |
| **SER** | Speech Emotion Recognition. |
| **Sherpa-ONNX** | Inference framework around ONNX speech models. |
| **TTS** | Text-to-Speech. |
| **UPI** | Unified Person Identification. Binds DoA bearing + face embedding into one persistent user identity. |
| **Valence** | Emotion-model axis: positivity (-1..1). Pairs with arousal. |
