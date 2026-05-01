use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct PerceptionPacket {
    pub acoustic_valence: f32,
    pub acoustic_arousal: f32,
    pub visual_valence: f32,
    pub user_engagement: f32,
    pub doa_bearing: Option<f32>,
    pub user_id: Option<u32>,
    pub timestamp_secs: f64,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SocialContext {
    pub fused_valence: f32,
    pub fused_arousal: f32,
    pub dissonance: f32,
    pub engagement: f32,
    pub is_active: bool,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum EmotionTint {
    Neutral,
    Happy,
    Sad,
    Angry,
    Thinking,
}
