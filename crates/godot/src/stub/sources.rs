use std::sync::{Arc, Mutex};
use std::time::Instant;

use uncanny_core::{
    ports::{AcousticSource, SemanticSource, SpatialSource, VisualSource},
    types::PerceptionPacket,
};

#[derive(Clone)]
pub struct FixtureSource {
    inner: Arc<Mutex<PerceptionPacket>>,
    semantic_updated_at: Arc<Mutex<Option<Instant>>>,
}

impl FixtureSource {
    pub fn new(initial: PerceptionPacket) -> Self {
        Self {
            inner: Arc::new(Mutex::new(initial)),
            semantic_updated_at: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set(&self, packet: PerceptionPacket, now: Instant) {
        *self.inner.lock().unwrap() = packet;
        let has_semantic = packet.semantic_valence.is_some();
        *self.semantic_updated_at.lock().unwrap() = if has_semantic { Some(now) } else { None };
    }
}

impl AcousticSource for FixtureSource {
    fn latest_valence(&self) -> f32 {
        self.inner.lock().unwrap().acoustic_valence
    }
    fn latest_arousal(&self) -> f32 {
        self.inner.lock().unwrap().acoustic_arousal
    }
}

impl VisualSource for FixtureSource {
    fn latest_valence(&self) -> f32 {
        self.inner.lock().unwrap().visual_valence
    }
    fn latest_engagement(&self) -> f32 {
        self.inner.lock().unwrap().user_engagement
    }
}

impl SemanticSource for FixtureSource {
    fn latest_valence(&self) -> Option<f32> {
        self.inner.lock().unwrap().semantic_valence
    }
    fn last_update(&self) -> Option<Instant> {
        *self.semantic_updated_at.lock().unwrap()
    }
}

impl SpatialSource for FixtureSource {
    fn latest_bearing(&self) -> Option<f32> {
        self.inner.lock().unwrap().doa_bearing
    }
}
