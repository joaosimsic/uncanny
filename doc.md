# Idea

Robot head with human expression and ai powered talk capabilities. It needs to behave like a human but to be visually a robot, with all its parts exposed. The idea isn't for it to look like a human. it is to cause conflicting feelings in whoever looks or talks to the robot.
The robot has to live in the uncanny valley, where interact with it becomes uncomfortable.

## Constrains

1. **Plug-and-play:** I don't want to be setting the robot up just to make it work. Once it's in a power supply, it needs to work out of the box.

2. **Human-like behavior:** It will have elements such eyes, eyelids and eyebrows that mimic human behavior. I need to express human emotion via facial expressions, speech and visual contact.

3. **Speak direction recognition:** It needs to be able to identify the direction in which a new speaker is communication with it and turn its head to make visual contact. As well as starting and maintaining a conversation.

## Technology

There will be employed a handful of technologies to make the robot alive, such as:

### OpenCV (Computer Vision)

In order to it to communicate, it needs to distinguish who its talking to. In that sense, it will use a webcam to recognize people faces so it can address its communication.

### SenseVoice-Small (Speech Foundation Model)

Ability for the robot to transcribe arriving communication, as well as identifying emotion.

### Qwen 2.5 3B 4-bit (SLM)

Its going to be the brain that interprets the communication and generates a response based in the context.

### Piper / Kokoro (TTS)

For speaking what was generated on the SLM as the response.

### Programming language

Because of the fact that I hate python and C++, I was considering using both:
- Rust
- C

## Hardware

### PC

Most affordable option:

Mini PC:
- Ryzen 5 7430u
- 16GB RAM DDR4
- 512GB SSD
- RX Vega 7(Integrated)

### Audio

Use of a microphone array using direction of arrival to distinguish the source of speak.

### Arduino

Handling of low level components such servos and microphone.

## Validation

### Godot

## Research

Animatronic eyes with impression file and instructions:
https://www.reddit.com/r/3Dprinting/comments/1jr7qvq/i_designed_animatronic_eye_mechanisms_files_below/
