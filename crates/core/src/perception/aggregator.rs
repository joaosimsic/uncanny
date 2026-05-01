use std::time::Instant;

use crate::ports::{AcousticSource, SemanticSource, SpatialSource, VisualSource};
use crate::types::PerceptionPacket;

pub struct PerceptionAggregator<A, V, S, P>
where
    A: AcousticSource,
    V: VisualSource,
    S: SemanticSource,
    P: SpatialSource,
{
    acoustic: A,
    visual: V,
    semantic: S,
    spatial: P,
    epoch: Instant,
}

impl<A, V, S, P> PerceptionAggregator<A, V, S, P>
where
    A: AcousticSource,
    V: VisualSource,
    S: SemanticSource,
    P: SpatialSource,
{
    pub fn new(acoustic: A, visual: V, semantic: S, spatial: P, epoch: Instant) -> Self {
        Self {
            acoustic,
            visual,
            semantic,
            spatial,
            epoch,
        }
    }

    pub fn tick(&self, now: Instant) -> PerceptionPacket {
        let semantic_valence = self.semantic.latest_valence();
        let semantic_age_ms = match self.semantic.last_update() {
            Some(t) => now
                .saturating_duration_since(t)
                .as_millis()
                .min(u32::MAX as u128) as u32,
            None => u32::MAX,
        };

        PerceptionPacket {
            acoustic_valence: self.acoustic.latest_valence(),
            acoustic_arousal: self.acoustic.latest_arousal(),
            visual_valence: self.visual.latest_valence(),
            user_engagement: self.visual.latest_engagement(),
            semantic_valence,
            semantic_age_ms,
            doa_bearing: self.spatial.latest_bearing(),
            user_id: None,
            timestamp_secs: now.saturating_duration_since(self.epoch).as_secs_f64(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::time::Duration;

    use super::*;

    struct Acoustic {
        valence: f32,
        arousal: f32,
    }
    impl AcousticSource for Acoustic {
        fn latest_valence(&self) -> f32 {
            self.valence
        }
        fn latest_arousal(&self) -> f32 {
            self.arousal
        }
    }

    struct Visual {
        valence: f32,
        engagement: f32,
    }
    impl VisualSource for Visual {
        fn latest_valence(&self) -> f32 {
            self.valence
        }
        fn latest_engagement(&self) -> f32 {
            self.engagement
        }
    }

    struct Semantic {
        valence: Cell<Option<f32>>,
        last: Cell<Option<Instant>>,
    }
    impl SemanticSource for Semantic {
        fn latest_valence(&self) -> Option<f32> {
            self.valence.get()
        }
        fn last_update(&self) -> Option<Instant> {
            self.last.get()
        }
    }

    struct Spatial(Option<f32>);
    impl SpatialSource for Spatial {
        fn latest_bearing(&self) -> Option<f32> {
            self.0
        }
    }

    fn agg(
        epoch: Instant,
        sem_at: Option<Instant>,
        sem_v: Option<f32>,
    ) -> PerceptionAggregator<Acoustic, Visual, Semantic, Spatial> {
        PerceptionAggregator::new(
            Acoustic {
                valence: 0.2,
                arousal: 0.4,
            },
            Visual {
                valence: -0.1,
                engagement: 0.7,
            },
            Semantic {
                valence: Cell::new(sem_v),
                last: Cell::new(sem_at),
            },
            Spatial(Some(1.5)),
            epoch,
        )
    }

    #[test]
    fn tick_copies_port_snapshots() {
        let epoch = Instant::now();
        let a = agg(epoch, Some(epoch), Some(0.5));
        let p = a.tick(epoch + Duration::from_millis(100));

        assert_eq!(p.acoustic_valence, 0.2);
        assert_eq!(p.acoustic_arousal, 0.4);
        assert_eq!(p.visual_valence, -0.1);
        assert_eq!(p.user_engagement, 0.7);
        assert_eq!(p.semantic_valence, Some(0.5));
        assert_eq!(p.doa_bearing, Some(1.5));
        assert!((p.timestamp_secs - 0.1).abs() < 1e-6);
    }

    #[test]
    fn semantic_age_is_clock_delta() {
        let epoch = Instant::now();
        let utterance_at = epoch + Duration::from_millis(200);
        let a = agg(epoch, Some(utterance_at), Some(0.0));
        let p = a.tick(utterance_at + Duration::from_millis(750));
        assert_eq!(p.semantic_age_ms, 750);
    }

    #[test]
    fn semantic_age_saturates_when_no_update() {
        let epoch = Instant::now();
        let a = agg(epoch, None, None);
        let p = a.tick(epoch + Duration::from_millis(50));
        assert_eq!(p.semantic_valence, None);
        assert_eq!(p.semantic_age_ms, u32::MAX);
    }
}
