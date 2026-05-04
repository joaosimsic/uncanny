use uncanny_core::{ports::VoiceEmitter, types::EmotionTint};

pub struct StubVoice;

impl VoiceEmitter for StubVoice {
    fn speak(&mut self, text: &str, tint: EmotionTint) {
        eprintln!("[voice] speak({tint:?}) \"{text}\"");
    }
}
