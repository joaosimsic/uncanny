use crate::types::SocialContext;
use super::weights::DECAY_RATE;

pub fn step_factor(current: f32, dt_secs: f32) -> f32 {
    (current * (1.0 - DECAY_RATE * dt_secs)).max(0.0)
}

pub fn apply_factor(ctx: SocialContext, factor: f32) -> SocialContext {
    SocialContext {
        fused_valence: ctx.fused_valence * factor,
        fused_arousal: ctx.fused_arousal * factor,
        dissonance: ctx.dissonance * factor,
        engagement: ctx.engagement * factor,
        is_active: ctx.is_active,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factor_decays_per_rate() {
        let f = step_factor(1.0, 1.0);
        assert!((f - (1.0 - DECAY_RATE)).abs() < 1e-6);
    }

    #[test]
    fn factor_clamps_at_zero() {
        assert_eq!(step_factor(0.001, 1000.0), 0.0);
    }

    #[test]
    fn apply_factor_scales_all_fields() {
        let ctx = SocialContext {
            fused_valence: 0.8,
            fused_arousal: 0.6,
            dissonance: 0.4,
            engagement: 0.9,
            is_active: false,
        };
        let out = apply_factor(ctx, 0.5);
        assert!((out.fused_valence - 0.4).abs() < 1e-6);
        assert!((out.fused_arousal - 0.3).abs() < 1e-6);
        assert!(!out.is_active);
    }
}
