use std::time::Instant;

use crate::behavior::{apply, classify, BehaviorMode};
use crate::fusion::FusionEngine;
use crate::perception::aggregator::PerceptionAggregator;
use crate::ports::{AcousticSource, EyeController, SemanticSource, SpatialSource, VisualSource, VoiceEmitter};

pub struct Runtime<A, V, S, P, E, Em>
where
    A: AcousticSource,
    V: VisualSource,
    S: SemanticSource,
    P: SpatialSource,
    E: EyeController,
    Em: VoiceEmitter,
{
    aggregator: PerceptionAggregator<A, V, S, P>,
    fusion: FusionEngine,
    eye: E,
    voice: Em,
    last_mode: Option<BehaviorMode>,
}

impl<A, V, S, P, E, Em> Runtime<A, V, S, P, E, Em>
where
    A: AcousticSource,
    V: VisualSource,
    S: SemanticSource,
    P: SpatialSource,
    E: EyeController,
    Em: VoiceEmitter,
{
    pub fn new(
        aggregator: PerceptionAggregator<A, V, S, P>,
        fusion: FusionEngine,
        eye: E,
        voice: Em,
    ) -> Self {
        Self {
            aggregator,
            fusion,
            eye,
            voice,
            last_mode: None,
        }
    }

    pub fn tick(&mut self, now: Instant) {
        let packet = self.aggregator.tick(now);
        let ctx = self.fusion.tick(&packet, now);
        let mode = classify(&ctx);
        apply(mode, &mut self.eye, &mut self.voice);
        self.last_mode = Some(mode);
    }

    pub fn last_mode(&self) -> Option<BehaviorMode> {
        self.last_mode
    }
}
