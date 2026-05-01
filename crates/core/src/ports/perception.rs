use std::time::Instant;

pub trait AcousticSource {
    fn latest_valence(&self) -> f32;
    fn latest_arousal(&self) -> f32;
}

pub trait VisualSource {
    fn latest_valence(&self) -> f32;
    fn latest_engagement(&self) -> f32;
}

/// Semantic stream emits valence on a slow, irregular cadence —
/// only when the LLM finishes a "Thinking" pass on a transcribed
/// utterance. Adapters expose the most recent valence and the
/// monotonic `Instant` it was produced; the aggregator translates
/// that to `semantic_age_ms` so fusion can gate by staleness.
pub trait SemanticSource {
    fn latest_valence(&self) -> Option<f32>;
    fn last_update(&self) -> Option<Instant>;
}

pub trait SpatialSource {
    fn latest_bearing(&self) -> Option<f32>;
}