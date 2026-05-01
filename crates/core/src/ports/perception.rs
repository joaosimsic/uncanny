pub trait AcousticSource {
    fn latest_valence(&self) -> f32;
    fn latest_arousal(&self) -> f32;
}

pub trait VisualSource {
    fn latest_valence(&self) -> f32;
    fn latest_engagement(&self) -> f32;
}

pub trait SemanticSource {
    fn latest_text(&self) -> Option<String>;
}

pub trait SpatialSource {
    fn latest_bearing(&self) -> Option<f32>;
}
