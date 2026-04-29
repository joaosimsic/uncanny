# Emotion Fusion Engine

This document defines the **Core Logic** responsible for synthesizing data from the multiple detection adapters into a singular behavioral state. In the hexagonal architecture, the Fusion Engine sits in the **Domain/Core** layer, isolated from specific hardware or model implementations.

---

## 1. Functional Overview
The Fusion Engine acts as a "Social Truth" processor. It takes the `PerceptionPacket` (Acoustic, Visual, and Semantic inputs) and calculates a unified `SocialContext`. Its primary goal is to identify **congruence**—the alignment between what a person says and how they act.

---

## 2. The Fusion Logic (Weighted Aggregation)
The engine does not treat all inputs equally. Weights are dynamic based on the confidence scores of the input adapters.

### A. Valence Calculation (Positivity vs. Negativity)
The engine calculates a weighted average to determine the user's apparent mood.
* **Semantic Weight (35%):** The literal meaning of the words.
* **Acoustic Weight (45%):** The emotional "truth" found in the tone.
* **Visual Weight (20%):** The physical micro-expression.

### B. Arousal Calculation (Energy/Intensity)
Determines the "tempo" of the robot's reaction.
* Derived primarily from **SenseVoice** (volume/speed) and **Facial Landmark** jitter (trembling or wide eyes).

---

## 3. The Uncanny Valley Logic: Incongruence
The engine's most critical "Uncanny" feature is the **Incongruence Detector**. This identifies when a human is masking their emotions.

| Scenario | Semantic (Text) | Acoustic (Tone) | Fusion Result | Robot Behavior |
| :--- | :--- | :--- | :--- | :--- |
| **Sarcasm** | Positive ("Great.") | Negative (Low/Flat) | **Conflict State** | Slow, tilting head stare. |
| **Suppressed Anger** | Neutral ("Fine.") | High Energy/Tense | **Conflict State** | Rapid blinking, slight eye narrowing. |
| **Genuine Joy** | Positive | Positive | **Aligned State** | Mechanical mimicry (delayed). |

### The Congruence Formula
The engine calculates a **Dissonance Factor ($D$)**:
$$D = \sqrt{(V_{acoustic} - V_{semantic})^2}$$
If $D > 0.6$, the engine triggers the **"Analytical Stare"** state, prioritizing the robot's visual observation over conversational flow.

---

## 4. State Persistence & Decay
To avoid "flickering" behavior, the engine implements an emotional buffer.
* **Hysteresis:** A change in detected emotion must persist for >300ms to trigger a motor change.
* **Mood Decay:** If the user leaves the frame or stops talking, the robot’s "perception" of their mood decays toward **Neutral** at a rate of 10% per second.

---

## 5. Domain Interface (The "Port")
The Fusion Engine exposes the following structure to the **Behavioral Layer**:

```rust
struct SocialContext {
    fused_valence: f32,    // Combined positivity
    fused_arousal: f32,    // Combined energy level
    dissonance: f32,       // Level of perceived "human lying"
    engagement: f32,       // Is the human looking at me?
    is_active: bool,       // Is a valid human target present?
}
```

---

## 6. Performance & Threads
* **Rate:** The Fusion Engine ticks at **20Hz** (every 50ms).
* **Isolation:** The engine runs in the main Core loop. It does not wait for I/O; it simply reads the most recent `PerceptionPacket` from an atomic memory location updated by the Detection adapters.
* **Determinism:** Given the same input packet, the Fusion Engine always produces the same `SocialContext`, making it easy to test in the **Godot Simulation**.
