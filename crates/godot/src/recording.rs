use std::sync::{Arc, Mutex};

use uncanny_core::{ports::{EyeController, VoiceEmitter}, types::EmotionTint};

#[derive(Debug, Clone, PartialEq)]
pub enum EyeCommand {
    LookAt(f32, f32),
    Blink,
    Saccade,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VoiceCommand {
    pub text: String,
    pub emotion: EmotionTint,
}

pub struct RecordingEye {
    log: Arc<Mutex<Vec<EyeCommand>>>,
}

impl RecordingEye {
    pub fn new() -> Self {
        Self { log: Arc::new(Mutex::new(Vec::new())) }
    }

    pub fn log(&self) -> Arc<Mutex<Vec<EyeCommand>>> {
        Arc::clone(&self.log)
    }

    pub fn drain(&self) -> Vec<EyeCommand> {
        self.log.lock().unwrap().drain(..).collect()
    }
}

impl EyeController for RecordingEye {
    fn look_at(&mut self, x: f32, y: f32) {
        self.log.lock().unwrap().push(EyeCommand::LookAt(x, y));
    }

    fn blink(&mut self) {
        self.log.lock().unwrap().push(EyeCommand::Blink);
    }

    fn saccade(&mut self) {
        self.log.lock().unwrap().push(EyeCommand::Saccade);
    }
}

pub struct RecordingVoice {
    log: Arc<Mutex<Vec<VoiceCommand>>>,
}

impl RecordingVoice {
    pub fn new() -> Self {
        Self { log: Arc::new(Mutex::new(Vec::new())) }
    }

    pub fn log(&self) -> Arc<Mutex<Vec<VoiceCommand>>> {
        Arc::clone(&self.log)
    }

    pub fn drain(&self) -> Vec<VoiceCommand> {
        self.log.lock().unwrap().drain(..).collect()
    }
}

impl VoiceEmitter for RecordingVoice {
    fn speak(&mut self, text: &str, tint: EmotionTint) {
        self.log.lock().unwrap().push(VoiceCommand { text: text.to_owned(), emotion: tint });
    }
}
