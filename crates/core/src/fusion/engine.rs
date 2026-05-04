use std::time::{Duration, Instant};

use crate::types::{PerceptionPacket, SocialContext};
use super::decay;
use super::hysteresis::Hysteresis;
use super::weights::{SEMANTIC_STALE_MS, W_ACOUSTIC, W_SEMANTIC, W_VISUAL};

pub struct FusionEngine {
    hysteresis: Hysteresis<SocialContext>,
    last_tick: Option<Instant>,
    decay_factor: f32,
}

impl Default for FusionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl FusionEngine {
    pub fn new() -> Self {
        let neutral = SocialContext {
            fused_valence: 0.0,
            fused_arousal: 0.0,
            dissonance: 0.0,
            engagement: 0.0,
            is_active: false,
        };
        Self {
            hysteresis: Hysteresis::new(neutral, Duration::from_millis(300)),
            last_tick: None,
            decay_factor: 1.0,
        }
    }

    pub fn tick(&mut self, packet: &PerceptionPacket, now: Instant) -> SocialContext {
        let dt_secs = match self.last_tick {
            Some(prev) => now.saturating_duration_since(prev).as_secs_f32(),
            None => 0.0,
        };
        self.last_tick = Some(now);

        let raw = compute_raw(packet);
        let committed = self.hysteresis.update(raw, now);

        if committed.is_active {
            self.decay_factor = 1.0;
            committed
        } else {
            self.decay_factor = decay::step_factor(self.decay_factor, dt_secs);
            decay::apply_factor(committed, self.decay_factor)
        }
    }
}

fn compute_raw(packet: &PerceptionPacket) -> SocialContext {
    let semantic_fresh = packet.semantic_valence.is_some()
        && packet.semantic_age_ms <= SEMANTIC_STALE_MS;

    let fused_valence = if semantic_fresh {
        let sv = packet.semantic_valence.unwrap();
        W_ACOUSTIC * packet.acoustic_valence + W_SEMANTIC * sv + W_VISUAL * packet.visual_valence
    } else {
        let w_a = W_ACOUSTIC / (W_ACOUSTIC + W_VISUAL);
        let w_v = W_VISUAL / (W_ACOUSTIC + W_VISUAL);
        w_a * packet.acoustic_valence + w_v * packet.visual_valence
    };

    let dissonance = if semantic_fresh {
        (packet.acoustic_valence - packet.semantic_valence.unwrap()).abs()
    } else {
        0.0
    };

    let engagement = packet.user_engagement;

    SocialContext {
        fused_valence: fused_valence.clamp(-1.0, 1.0),
        fused_arousal: packet.acoustic_arousal.clamp(0.0, 1.0),
        dissonance: dissonance.clamp(0.0, 1.0),
        engagement,
        is_active: engagement > 0.1,
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::fusion::weights::DISSONANCE_THRESHOLD;

    fn packet(av: f32, aa: f32, vv: f32, ue: f32, sv: Option<f32>, sa: u32) -> PerceptionPacket {
        PerceptionPacket {
            acoustic_valence: av,
            acoustic_arousal: aa,
            visual_valence: vv,
            user_engagement: ue,
            semantic_valence: sv,
            semantic_age_ms: sa,
            doa_bearing: None,
            user_id: None,
            timestamp_secs: 0.0,
        }
    }

    #[test]
    fn sarcasm_exceeds_dissonance_threshold_after_hold() {
        let mut eng = FusionEngine::new();
        let epoch = Instant::now();
        let p = packet(-0.6, 0.3, -0.1, 0.8, Some(0.7), 0);
        let mut ctx = SocialContext {
            fused_valence: 0.0,
            fused_arousal: 0.0,
            dissonance: 0.0,
            engagement: 0.0,
            is_active: false,
        };
        for i in 0u64..7 {
            ctx = eng.tick(&p, epoch + Duration::from_millis(50 * i));
        }
        assert!(ctx.dissonance > DISSONANCE_THRESHOLD, "dissonance={}", ctx.dissonance);
        assert!(ctx.is_active);
    }

    #[test]
    fn stale_semantic_gives_zero_dissonance() {
        let mut eng = FusionEngine::new();
        let epoch = Instant::now();
        let p = packet(0.4, 0.2, 0.2, 0.8, Some(0.9), 4000);
        let ctx = eng.tick(&p, epoch);
        assert!((ctx.dissonance).abs() < 1e-6, "expected dissonance=0, got {}", ctx.dissonance);
    }

    #[test]
    fn inactive_ticks_decay_output() {
        let mut eng = FusionEngine::new();
        let epoch = Instant::now();

        let active_p = packet(0.8, 0.6, 0.5, 0.9, None, u32::MAX);
        for i in 0u64..8 {
            eng.tick(&active_p, epoch + Duration::from_millis(50 * i));
        }

        let inactive_p = packet(0.0, 0.0, 0.0, 0.0, None, u32::MAX);
        let base = epoch + Duration::from_millis(400);
        let mut ctx = eng.tick(&inactive_p, base);
        for i in 1u64..=200 {
            ctx = eng.tick(&inactive_p, base + Duration::from_millis(50 * i));
        }
        assert!(
            ctx.fused_arousal.abs() < 0.1,
            "arousal after 10s inactive: {}",
            ctx.fused_arousal
        );
    }
}
