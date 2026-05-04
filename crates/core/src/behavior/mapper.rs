use crate::ports::{EyeController, VoiceEmitter};
use crate::types::SocialContext;
use crate::fusion::weights::{DISSONANCE_THRESHOLD, ENGAGEMENT_FLOOR};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BehaviorMode {
    Idle,
    AnalyticalStare,
    Search,
    MimicryPositive,
    MimicryNegative,
}

pub fn classify(ctx: &SocialContext) -> BehaviorMode {
    if !ctx.is_active {
        BehaviorMode::Idle
    } else if ctx.dissonance > DISSONANCE_THRESHOLD {
        BehaviorMode::AnalyticalStare
    } else if ctx.engagement < ENGAGEMENT_FLOOR {
        BehaviorMode::Search
    } else if ctx.fused_valence >= 0.0 {
        BehaviorMode::MimicryPositive
    } else {
        BehaviorMode::MimicryNegative
    }
}

pub fn apply(mode: BehaviorMode, eye: &mut impl EyeController, _voice: &mut impl VoiceEmitter) {
    match mode {
        BehaviorMode::Idle => eye.saccade(),
        BehaviorMode::AnalyticalStare => eye.look_at(0.0, 0.0),
        BehaviorMode::Search => eye.saccade(),
        BehaviorMode::MimicryPositive => {
            eye.look_at(0.0, 0.0);
            eye.blink();
        }
        BehaviorMode::MimicryNegative => eye.look_at(0.0, -0.3),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(is_active: bool, dissonance: f32, engagement: f32, fused_valence: f32) -> SocialContext {
        SocialContext {
            fused_valence,
            fused_arousal: 0.5,
            dissonance,
            engagement,
            is_active,
        }
    }

    #[test]
    fn inactive_is_idle() {
        assert_eq!(classify(&ctx(false, 0.0, 0.9, 0.5)), BehaviorMode::Idle);
    }

    #[test]
    fn high_dissonance_is_stare() {
        assert_eq!(
            classify(&ctx(true, 0.7, 0.9, 0.5)),
            BehaviorMode::AnalyticalStare
        );
    }

    #[test]
    fn low_engagement_is_search() {
        assert_eq!(classify(&ctx(true, 0.0, 0.1, 0.5)), BehaviorMode::Search);
    }

    #[test]
    fn positive_valence_is_mimicry_pos() {
        assert_eq!(
            classify(&ctx(true, 0.0, 0.5, 0.3)),
            BehaviorMode::MimicryPositive
        );
    }

    #[test]
    fn negative_valence_is_mimicry_neg() {
        assert_eq!(
            classify(&ctx(true, 0.0, 0.5, -0.3)),
            BehaviorMode::MimicryNegative
        );
    }
}
