use crate::types::EmotionTint;

pub trait VoiceEmitter {
    fn speak(&mut self, text: &str, emotion_tint: EmotionTint);
}
