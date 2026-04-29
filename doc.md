# Idea

Robot head with human expression and ai powered talk capabilities. It needs to behave like a human but to be visually a robot, with all its parts exposed. The idea isn't for it to look like a human. it is to cause conflicting feelings in whoever looks or talks to the robot.
The robot has to live in the uncanny valley, where interact with it becomes uncomfortable.

## Constrains

1. **Plug-and-play:** I don't want to be setting the robot up just to make it work. Once it's in a power supply, it needs to work out of the box. One minute boot time is fine.

2. **Human-like behavior:** It will have elements such eyes, eyelids and eyebrows that mimic human behavior. I need to express human emotion via facial expressions, speech and visual contact.

3. **Speak direction recognition:** It needs to be able to identify the direction in which a new speaker is communication with it and turn its head to make visual contact. As well as starting and maintaining a conversation.

4. **Low latency:** In order to maintain a communication, the robot must have low latency to don't be a bottleneck in the conversation.

5. **Speaking language:** The robot need to mostly speak portuguese, English support may be added in the future.

## Fields

There will be employed a handful of technologies in various fields to make the robot alive and aware, such as:

### Vision

Use InsightFace, being ReticaFace for detection and ArcFace for recognition. The implementation will be made with ort to run ONNX runtime. The processing will be offloaded to iGPU with OpenVINO with AMD support.

### Hearing

Similar vision, using ONNX to run SenseVoice-Small model. The model will also provide speech emotion recognition(SER) and audio event detection(AED) with low latency. Use a full duplex approach with a sliding window buffer for audio. The audio will be handled to the SLM wiht cpal crate.

### Thinking

Use llama.cpp to run Qwen 2.5 3B(Q4_K_M) model via llama-cpp-2. This allow to manage KV cache and don't starve the vision system. A 1.5B model is also considered to save KV cache.

### Speaking

Use Sherpa-ONNX with the ort setup to run Piper model.

### Programming language

Because of the fact that I hate python and C++, I was considering using both:
- Rust
- C

## Architecture

### Unified Person Identification



## Hardware

**PC:**
Ryzen 5 7430u
16GB RAM DDR4
512GB SSD
RX Vega 7(Integrated)

**Video:**
Full HD webcam

**Audio:**
ReSpeaker 4-Mic Array

### Arduino

Handling of low level components such servos and microphone.

## Simulation

### Godot



## Research

Animatronic eyes with impression file and instructions:
https://www.reddit.com/r/3Dprinting/comments/1jr7qvq/i_designed_animatronic_eye_mechanisms_files_below/

50s retro robots style

https://www.nationalgeographic.com/science/article/robot-humanoids-mechanical-engineering

Cool robot design parts:
https://www.youtube.com/@WillCogley/videos
